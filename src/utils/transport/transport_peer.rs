use crate::{Error, Result, DEFAULT_SSE_CHANNEL_CAPACITY};
use axum::response::sse::Event;
use axum::response::Sse;
use futures::Stream;
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
use tokio_stream::StreamExt;
use uuid::Uuid;

#[derive(Debug)]
struct TransportPeerInner {
    pub from_client_tx: broadcast::Sender<ClientJsonRpcMessage>,
    pub from_client_rx: broadcast::Receiver<ClientJsonRpcMessage>,
    pub server_tx: broadcast::Sender<JsonRpcMessage>,
    pub from_server_rx: broadcast::Receiver<JsonRpcMessage>,

    task_bind_stdin: Option<JoinHandle<()>>,
    task_bind_stdout: Option<JoinHandle<()>>,
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
        Self {
            server_tx: from_server_tx,
            from_server_rx,
            from_client_tx,
            from_client_rx,
            task_bind_stdin: None,
            task_bind_stdout: None,
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

impl TransportPeer {
    pub fn new() -> Self {
        let inner = TransportPeerInner::new();
        let inner = RwLock::new(inner);
        let inner = Arc::new(inner);
        let id = Uuid::new_v4().to_string();
        Self { id, inner }
    }

    pub async fn attach_stdin(
        &self,
        stdin_tx: broadcast::Sender<ClientJsonRpcMessage>,
    ) -> Result<()> {
        if self.inner.read().await.task_bind_stdin.is_some() {
            return Err(Error::generic(
                "Transport peer already has a task to bind to stdin",
            ));
        }

        let mut rx = { self.inner.read().await.from_client_rx.resubscribe() };
        let task = tokio::spawn(async move {
            loop {
                if let Ok(message) = rx.recv().await {
                    if let Err(error) = stdin_tx.send(message) {
                        let _ = Error::from(error).trace();
                    }
                }
            }
        });

        let mut inner = self.inner.write().await;
        inner.task_bind_stdin = Some(task);
        Ok(())
    }

    /// Attach to the process stdout.
    #[tracing::instrument(name = "TransportPeer::AttachStdout", skip(self))]
    pub async fn attach_stdout(
        &self,
        mut stdout_rx: broadcast::Receiver<JsonRpcMessage>,
    ) -> Result<()> {
        if self.inner.read().await.task_bind_stdout.is_some() {
            return Err(Error::generic(
                "Transport peer already has a task to bind to server",
            ));
        }

        // --- Extract a reference to the server's broadcast channel sender.
        let tx = self.inner.read().await.server_tx.clone();

        // --- Start a task that will receive messages from the server's Broadcast channel
        // --- and send them to the peer's mpsc channel (`from_client_tx`). This allows for
        // --- distict handling of messages sent by the server to the peer.
        self.inner.write().await.task_bind_stdout = Some(tokio::spawn(async move {
            loop {
                match stdout_rx.recv().await {
                    Ok(message) => {
                        if let Err(error) = tx.send(message) {
                            let _ = Error::from(error).trace();
                        }
                    }
                    Err(error) => {
                        let _ = Error::from(error).trace();
                    }
                }
            }
        }));

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
        match self.inner.read().await.server_tx.subscribe().recv().await {
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
    ) -> Sse<impl Stream<Item = core::result::Result<Event, Infallible>>> {
        let endpoint = format!("{endpoint}?sessionId={}", self.id);

        // --- Create an initial "once" stream that will send and single payload
        // --- with the endpoint URL for the SSE stream. This is required by the
        // --- MCProtocol to establish the SSE connection with the server.
        let endpoint = ::futures::stream::once(::futures::future::ok::<_, Infallible>(
            Event::default().event("endpoint").data(endpoint),
        ));

        // --- Create a `BroadcastStream` from the peer's receiver that will send
        // --- every message send by the STDOUT of the pod to the SSE stream.
        let rx = self.inner.read().await.from_server_rx.resubscribe();
        let stream = BroadcastStream::new(rx).map(|message| {
            Ok::<Event, Infallible>(
                Event::default()
                    .event("message")
                    .data(serde_json::to_string(&message.unwrap()).unwrap()),
            )
        });

        // --- Chain the two streams together, so that the first stream sends the
        // --- endpoint URL and the second stream sends the messages from the
        // --- STDOUT of the pod.
        let stream = endpoint.chain(stream);
        Sse::new(stream)
    }
}
