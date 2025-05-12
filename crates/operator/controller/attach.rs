use super::Controller;
use crate::{Error, MCPServer, Result};
use axum::response::sse::Event;
use futures::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, AttachParams, AttachedProcess};
use kube::ResourceExt;
use rmcp::model::{
    ErrorCode, ErrorData, JsonRpcError, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest,
    JsonRpcResponse, JsonRpcVersion2_0, Notification, NumberOrString,
};
use serde_json as JSON;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};
use tokio_stream::wrappers::BroadcastStream;
use tracing::{debug, error, info};

// Configuration constants
const DEFAULT_SSE_CHANNEL_CAPACITY: usize = 100;
const DEFAULT_POD_BUFFER_SIZE: usize = 1024 * 256; //  256 KiB

///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum MCPEvent {
    Endpoint(String),
    Message(JsonRpcMessage),
    Error(String),
}

impl From<JsonRpcMessage> for MCPEvent {
    fn from(message: JsonRpcMessage) -> Self {
        MCPEvent::Message(message)
    }
}

impl From<MCPEvent> for Event {
    fn from(event: MCPEvent) -> Self {
        match event {
            MCPEvent::Endpoint(endpoint) => Event::default().event("endpoint").data(endpoint),
            MCPEvent::Error(error) => Event::default().event("error").data(error),
            MCPEvent::Message(message) => Event::default()
                .event("message")
                .data(JSON::to_string(&message).unwrap()),
        }
    }
}

///////////////////////////////////////////////////////////////////////

impl From<Error> for JsonRpcMessage {
    fn from(error: Error) -> Self {
        JsonRpcMessage::Error(JsonRpcError {
            error: ErrorData {
                code: ErrorCode(-32603),
                data: None,
                message: error.to_string().into(),
            },
            id: NumberOrString::Number(u32::MAX),
            jsonrpc: JsonRpcVersion2_0,
        })
    }
}

///////////////////////////////////////////////////////////////////////

pub struct MCPServerTransportStdio {
    pub process: Arc<RwLock<AttachedProcess>>,
    is_listening: Arc<RwLock<bool>>,
    stdin_rx: Arc<RwLock<Receiver<JsonRpcRequest>>>,
    stdin_tx: Arc<RwLock<Sender<JsonRpcRequest>>>,
    stdout_tx: Arc<RwLock<Sender<MCPEvent>>>,
    stdout_rx: Arc<RwLock<Receiver<MCPEvent>>>,

    /// The cached `initialize` response from the MCP server.
    initialize_response: Arc<RwLock<Option<JsonRpcResponse>>>,
}

impl MCPServerTransportStdio {
    /// Listen for messages from the process stdout and send them to the receiver.
    pub async fn listen(&self) -> Result<()> {
        // --- If already listening, return early.
        let mut is_listening = self.is_listening.write().await;
        if *is_listening {
            return Ok(());
        }

        // --- Spawn a task to read from the process stdout and send events to the SSE stream.
        let stdout_tx = self.stdout_tx.clone();
        let stdin_rx = self.stdin_rx.clone();
        let mut stdin = self.process.clone().write().await.stdin().unwrap();
        let mut stdout = self.process.clone().write().await.stdout().unwrap();

        // --- Read from the stdin channel and write to the process stdin.
        tokio::spawn(async move {
            loop {
                info!("[LISTEN/STDIN] Reading from stdin channel");
                match stdin_rx.write().await.recv().await {
                    Ok(message) => {
                        let message = JSON::to_string(&message).unwrap();
                        let message = format!("{message}\n");
                        let message = message.as_bytes();
                        if let Err(e) = stdin.write_all(message).await {
                            error!("Failed to write to process stdin: {}", e);
                            break;
                        }
                        // if let Err(e) = stdin.flush().await {
                        //     error!("Failed to flush process stdin: {}", e);
                        //     break;
                        // }
                    }
                    Err(e) => {
                        error!("Error reading from stdin channel: {}", e);
                        break;
                    }
                }
            }
        });

        // --- Read from the stdout of the process and send to the SSE stream.
        tokio::spawn(async move {
            let mut buffer = vec![0u8; DEFAULT_POD_BUFFER_SIZE];
            loop {
                match stdout.read(&mut buffer).await {
                    Ok(0) => {
                        sleep(Duration::from_millis(100)).await;
                    }
                    Ok(size) => {
                        // --- Process the buffer line by line
                        let data = String::from_utf8_lossy(&buffer[..size]).to_string();
                        for line in data.lines() {
                            if line.trim().is_empty() {
                                continue;
                            }

                            // --- Parse each line as JSON
                            let message: JsonRpcMessage = match JSON::from_str(line) {
                                Ok(data) => data,
                                Err(error) => Error::from(error).into(),
                            };

                            // --- Broadcast the message to the SSE stream
                            if let JsonRpcMessage::Response(response) = &message {
                                let utf8 = JSON::to_string(&response).unwrap();
                                match stdout_tx.write().await.send(message.into()) {
                                    Ok(_) => info!("[LISTEN/STDOUT]: {}", utf8),
                                    Err(e) => {
                                        error!("Failed to send event to SSE stream: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("[LISTEN/STDOUT]: Error reading from process: {}", e);
                        break;
                    }
                }
            }

            info!("[LISTEN/STDOUT] Process has closed stdout stream");
        });

        // --- Set the listening flag to true.
        *is_listening = true;
        Ok(())
    }

    /// Write a message to the stdin of the attached process and read the first message from stdout.
    pub async fn send(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        // let mut stdout_rx = self.stdout_rx.write().await;
        let stdin_tx = self.stdin_tx.write().await;
        let mut stdout_rx = self.stdout_rx.write().await;

        // --- If the request is an initialize request, check if we have a cached response.
        // if request.request.method == "initialize" {
        //     let initialize_response = self.initialize_response.read().await;
        //     if let Some(response) = &*initialize_response {
        //         return Ok(response.clone());
        //     }
        // }

        // --- First, send the message to the stdin channel.
        match stdin_tx.send(request.clone()) {
            Ok(_) => info!("[MESSAGE/REQUEST]: {}", JSON::to_string(&request).unwrap()),
            Err(e) => {
                error!("[MESSAGE/REQUEST]: {}", e);
                return Err(Error::Internal(
                    "Failed to send message to stdin channel".into(),
                ));
            }
        }

        // Set a timeout for the response
        let future = async {
            loop {
                match stdout_rx.recv().await {
                    Ok(MCPEvent::Message(message)) => {
                        // --- Check if the message is the initialize response. If so, cache it
                        // --- so we can reuse it later if we get another initialize request.
                        if request.request.method == "initialize" {
                            if let JsonRpcMessage::Response(response) = &message {
                                if response.id == request.id {
                                    let mut initialize_response =
                                        self.initialize_response.write().await;
                                    *initialize_response = Some(response.clone());
                                }
                            }
                        }

                        // --- Check if the message is a response to the request.
                        if let JsonRpcMessage::Response(response) = message {
                            if response.id == request.id {
                                info!(
                                    "[MESSAGE/RESPONSE]: {}",
                                    JSON::to_string(&response).unwrap()
                                );
                                return Ok(response);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read from stdout channel: {}", e);
                        return Err(Error::Internal("Failed to read from stdout channel".into()));
                    }
                    _ => {
                        info!("Ignoring message");
                        debug!("Ignoring message");
                    }
                }
            }
        };

        // Apply the timeout
        let duration = Duration::from_secs(10);
        match timeout(duration, future).await {
            Ok(result) => result,
            Err(_) => Err(Error::Internal(format!(
                "Timed out waiting for response to request {}",
                request.id
            ))),
        }
    }

    /// Create a stream of SSE events from the process stdout.
    pub async fn subscribe(&self, endpoint: String) -> BroadcastStream<MCPEvent> {
        let rx = self.stdout_rx.read().await.resubscribe();
        let tx = self.stdout_tx.read().await;

        // --- Write the endpoint to the SSE stream.
        let event = MCPEvent::Endpoint(endpoint.to_owned());
        match tx.send(event) {
            Ok(_) => debug!("Sending endpoint to SSE stream: {}", endpoint),
            Err(e) => {
                error!("Failed to send endpoint to SSE stream: {}", e);
            }
        }

        // --- When the stream is closed, send an error event.
        BroadcastStream::new(rx)
    }
}

impl From<AttachedProcess> for MCPServerTransportStdio {
    fn from(process: AttachedProcess) -> Self {
        let (stdin_tx, stdin_rx) = channel::<JsonRpcRequest>(DEFAULT_SSE_CHANNEL_CAPACITY);
        let (stdout_tx, stdout_rx) = channel::<MCPEvent>(DEFAULT_SSE_CHANNEL_CAPACITY);
        MCPServerTransportStdio {
            initialize_response: Arc::new(RwLock::new(None)),
            is_listening: Arc::new(RwLock::new(false)),
            process: Arc::new(RwLock::new(process)),
            stdin_tx: Arc::new(RwLock::new(stdin_tx)),
            stdin_rx: Arc::new(RwLock::new(stdin_rx)),
            stdout_tx: Arc::new(RwLock::new(stdout_tx)),
            stdout_rx: Arc::new(RwLock::new(stdout_rx)),
        }
    }
}

impl Controller {
    /// Ensures that a pod's TTY is attached and returns true if successful
    pub async fn get_server_tty(&self, server: &MCPServer) -> Result<AttachedProcess> {
        Api::<Pod>::namespaced(self.get_client(), &self.get_namespace())
            .attach(
                &server.name_pod(),
                &AttachParams::interactive_tty()
                    .container("server")
                    .max_stdout_buf_size(DEFAULT_POD_BUFFER_SIZE)
                    .max_stdin_buf_size(DEFAULT_POD_BUFFER_SIZE),
            )
            .await
            .map_err(Error::KubeError)
    }

    /// Attaches to a pod's TTY and returns a stream of events for SSE.
    pub async fn get_server_sse(
        &self,
        server: &MCPServer,
    ) -> Result<Arc<RwLock<MCPServerTransportStdio>>> {
        let server_name = server.name_any();
        {
            let channels = self.channels.read().await;
            if let Some(existing_channels) = channels.get(&server_name) {
                return Ok(existing_channels.clone());
            }
        }

        // Create a new channel if none exists
        let process = self.get_server_tty(server).await?;
        let channels = Arc::new(RwLock::new(process.into()));

        // Store the channel in the hashmap
        {
            let mut channels_map = self.channels.write().await;
            channels_map.insert(server_name, channels.clone());
        }

        Ok(channels)
    }
}
