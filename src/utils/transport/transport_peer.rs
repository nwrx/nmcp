use crate::{Error, Result, DEFAULT_SSE_CHANNEL_CAPACITY};
use axum::response::sse::Event;
use axum::response::Sse;
use futures::{FutureExt, Stream, StreamExt};
use rmcp::model::{
    ClientJsonRpcMessage, ErrorCode, ErrorData, JsonRpcError, JsonRpcMessage, JsonRpcVersion2_0,
    NumberOrString,
};
use std::borrow::Cow;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::BroadcastStream;
use tracing::Instrument;
use uuid::Uuid;

#[derive(Debug)]
struct TransportPeerInner {
    pub from_client_tx: broadcast::Sender<ClientJsonRpcMessage>,
    pub from_client_rx: broadcast::Receiver<ClientJsonRpcMessage>,
    pub from_server_tx: broadcast::Sender<JsonRpcMessage>,
    pub from_server_rx: broadcast::Receiver<JsonRpcMessage>,
    drop_tx: broadcast::Sender<()>,
    drop_rx: broadcast::Receiver<()>,
    task_attach_input: Option<JoinHandle<()>>,
    task_attach_output: Option<JoinHandle<()>>,
}

impl Default for TransportPeerInner {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportPeerInner {
    pub fn new() -> Self {
        let (from_client_tx, from_client_rx) = broadcast::channel(DEFAULT_SSE_CHANNEL_CAPACITY);
        let (from_server_tx, from_server_rx) = broadcast::channel(DEFAULT_SSE_CHANNEL_CAPACITY);
        let (drop_tx, drop_rx) = broadcast::channel(1);
        Self {
            from_server_tx,
            from_server_rx,
            from_client_tx,
            from_client_rx,
            drop_tx,
            drop_rx,
            task_attach_input: None,
            task_attach_output: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransportPeer {
    pub id: String,
    inner: Arc<RwLock<TransportPeerInner>>,
}

impl Default for TransportPeer {
    fn default() -> Self {
        Self::new()
    }
}

/// A transport peer that manages bidirectional communication between a client and server
/// using JSON-RPC messages over various channels.
///
/// The `TransportPeer` acts as a bridge that:
/// - Receives messages from clients and forwards them to the server
/// - Receives messages from the server and forwards them to clients
/// - Manages background tasks for handling stdin, _output, and stderr streams
/// - Provides Server-Sent Events (SSE) streaming capabilities
/// - Handles request-response correlation for JSON-RPC calls
///
/// Each peer has a unique ID and maintains internal state protected by an `Arc<RwLock>`.
/// The peer can be attached to different I/O streams and supports graceful cleanup of
/// resources when closed.
///
/// # Examples
///
/// ```rust
/// let peer = TransportPeer::new();
///
/// // Attach to various streams
/// peer.attach_input(stdin_sender).await?;
/// peer.attach_output(_output_receiver).await?;
///
/// // Send a request and wait for response
/// let response = peer.send_request(request_message).await?;
///
/// // Clean up when done
/// peer.close().await?;
/// ```
impl TransportPeer {
    pub fn new() -> Self {
        let inner = TransportPeerInner::new();
        let inner = RwLock::new(inner);
        let inner = Arc::new(inner);
        let id = Uuid::new_v4().to_string();
        Self { id, inner }
    }

    /// Attach a `broadcast::Sender` to the transport input channel.
    #[tracing::instrument(name = "TransportPeer::AttachInput", skip(self))]
    pub async fn attach_input(&self, tx: broadcast::Sender<ClientJsonRpcMessage>) -> Result<()> {
        if self.inner.read().await.task_attach_input.is_some() {
            return Err(Error::generic(
                "Transport peer already has a task to bind to stdin",
            ));
        }
        let mut rx = { self.inner.read().await.from_client_rx.resubscribe() };
        let task = tokio::spawn(async move {
            loop {
                if let Ok(message) = rx.recv().await {
                    if let Err(error) = tx.send(message) {
                        let _ = Error::from(error).trace();
                    }
                }
            }
        });
        let mut inner = self.inner.write().await;
        inner.task_attach_input = Some(task);
        Ok(())
    }

    /// Attach a `broadcast::Receiver` to the transport output channel.
    #[tracing::instrument(name = "TransportPeer::AttachOutput", skip(self))]
    pub async fn attach_output(&self, mut rx: broadcast::Receiver<JsonRpcMessage>) -> Result<()> {
        if self.inner.read().await.task_attach_output.is_some() {
            return Err(Error::generic(
                "Transport peer already has a task to bind to server",
            ));
        }
        let tx = { self.inner.read().await.from_server_tx.clone() };
        let task = tokio::spawn(
            async move {
                loop {
                    match rx.recv().await {
                        Ok(message) => {
                            if let Err(error) = tx.send(message) {
                                let _ = Error::from(error).trace();
                            }
                        }
                        Err(error) => {
                            let _ = Error::from(error).trace();
                            return;
                        }
                    }
                }
            }
            .instrument(tracing::Span::current()),
        );

        self.inner.write().await.task_attach_output = Some(task);
        Ok(())
    }

    /// Send a message to the the transport.
    pub async fn send_message_to_server(&self, message: ClientJsonRpcMessage) -> Result<usize> {
        self.inner
            .read()
            .await
            .from_client_tx
            .send(message)
            .map_err(Error::from)
    }

    /// Receive a message from the transport.
    pub async fn receive_message_from_server(&self) -> Option<JsonRpcMessage> {
        match self
            .inner
            .read()
            .await
            .from_server_tx
            .subscribe()
            .recv()
            .await
        {
            Ok(message) => Some(message),
            Err(broadcast::error::RecvError::Closed) => None,
            Err(broadcast::error::RecvError::Lagged(_)) => None,
        }
    }

    /// Receive a result from the transport with the given request ID.
    pub async fn receive_result(&self, request_id: NumberOrString) -> JsonRpcMessage {
        let mut rx = self.inner.read().await.from_server_rx.resubscribe();
        tokio::spawn(async move {
            while let Ok(message) = rx.recv().await {
                if let Some((_, result_id)) = message.clone().into_response() {
                    if result_id == request_id {
                        return message;
                    }
                }
            }
            JsonRpcMessage::Error(JsonRpcError {
                id: request_id,
                jsonrpc: JsonRpcVersion2_0,
                error: ErrorData {
                    code: ErrorCode(-32603),
                    message: Cow::Borrowed("Channel closed or no response received"),
                    data: None,
                },
            })
        })
        .await
        .unwrap()
    }

    pub async fn send_request(
        &self,
        message: ClientJsonRpcMessage,
    ) -> Result<Option<JsonRpcMessage>> {
        match message.clone().into_request() {
            // --- Message is a request, note that we wait for the result
            // --- from the server before sending the request, ensuring
            // --- that we dont miss broadcast messages since we're `resubscribing`
            Some((_, request_id)) => {
                let future = self.receive_result(request_id);
                let _ = self.send_message_to_server(message).await?;
                Ok(Some(future.await))
            }

            // --- Message is not a request, skip early since we won't receive a response.
            None => Ok(None),
        }
    }

    /// Return the SSE stream for the peer.
    pub async fn sse(
        self,
        endpoint: String,
        on_close: impl FnOnce() -> JoinHandle<Result<()>> + Send + 'static,
    ) -> Sse<impl Stream<Item = core::result::Result<Event, Infallible>>> {
        let endpoint = format!("{endpoint}?sessionId={}", self.id);
        tracing::debug!("Creating SSE stream with id {}", self.id);

        // --- Create an initial "once" stream that will send and single payload
        // --- with the endpoint URL for the SSE stream. This is required by the
        // --- MCProtocol to establish the SSE connection with the server.
        let endpoint = ::futures::stream::once(::futures::future::ok::<_, Infallible>(
            Event::default().event("endpoint").data(endpoint),
        ));

        // --- Wrap the `drop` in a `BroadcastStream` so that we can append
        // --- it to the SSE stream and ensure that the stream is closed when
        // --- the peer is dropped from the server-side.
        let drop_rx = self.inner.read().await.drop_rx.resubscribe();
        let drop_stream = BroadcastStream::new(drop_rx).into_future().then(async |_| {
            tracing::info!("SSE stream for peer closed");
            let _ = on_close().await;
        });

        // --- Create a `BroadcastStream` from the peer's receiver that will send
        // --- every message as an SSE event. This will allow us to stream.
        let rx = self.inner.read().await.from_server_rx.resubscribe();
        let stream = BroadcastStream::new(rx)
            .map(|message| {
                Ok::<Event, Infallible>(
                    Event::default()
                        .event("message")
                        .data(serde_json::to_string(&message.unwrap()).unwrap()),
                )
            })
            .take_until(drop_stream);

        // --- Chain the two streams together, so that the first stream sends the
        // --- endpoint URL and the second stream sends the messages from the
        // --- STDOUT of the pod.
        let stream = endpoint.chain(stream);
        Sse::new(stream)
    }

    /// Close the transport peer, cleaning up resources and aborting any ongoing tasks.
    pub async fn close(&self) -> Result<()> {
        tracing::info!("Closing transport peer {}", self.id);
        let mut inner = self.inner.write().await;
        if let Some(task) = inner.task_attach_input.take() {
            task.abort();
        }
        if let Some(task) = inner.task_attach_output.take() {
            task.abort();
        }
        let _ = inner.drop_tx.send(());
        Ok(())
    }
}
