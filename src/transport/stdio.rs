use super::MCPEvent;
use crate::{Error, MCPServer, Result, DEFAULT_POD_BUFFER_SIZE, DEFAULT_SSE_CHANNEL_CAPACITY};
use kube::api::AttachedProcess;
use rmcp::model::{ClientJsonRpcMessage, JsonRpcMessage, NumberOrString};
use serde_json as JSON;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::sync::RwLock;
use tokio::time;
use tokio_stream::wrappers::BroadcastStream;

pub struct MCPServerTransportStdio {
    pub process: Arc<RwLock<AttachedProcess>>,
    pub server: MCPServer,
    is_alive: Arc<RwLock<bool>>,
    is_listening: Arc<RwLock<bool>>,
    stdin_rx: Arc<RwLock<Receiver<ClientJsonRpcMessage>>>,
    stdin_tx: Arc<RwLock<Sender<ClientJsonRpcMessage>>>,
    stdout_tx: Arc<RwLock<Sender<MCPEvent>>>,
    stdout_rx: Arc<RwLock<Receiver<MCPEvent>>>,
}

impl MCPServerTransportStdio {
    /// Create a new transport for the MCP server.
    pub fn new(process: AttachedProcess, server: MCPServer) -> Self {
        // --- Create a new channel for stdin and stdout.
        let (stdin_tx, stdin_rx) = channel(DEFAULT_SSE_CHANNEL_CAPACITY);
        let (stdout_tx, stdout_rx) = channel(DEFAULT_SSE_CHANNEL_CAPACITY);

        // --- Create a new transport for the MCP server.
        Self {
            server,
            process: Arc::new(RwLock::new(process)),
            is_alive: Arc::new(RwLock::new(true)),
            is_listening: Arc::new(RwLock::new(false)),
            stdin_rx: Arc::new(RwLock::new(stdin_rx)),
            stdin_tx: Arc::new(RwLock::new(stdin_tx)),
            stdout_tx: Arc::new(RwLock::new(stdout_tx)),
            stdout_rx: Arc::new(RwLock::new(stdout_rx)),
        }
    }

    /// Listen for messages from the process stdout and send them to the receiver.
    pub async fn listen(&self) -> Result<()> {
        if *self.is_listening.read().await {
            return Ok(());
        }

        let is_alive_1 = self.is_alive.clone();
        let is_alive_2 = self.is_alive.clone();
        let stdin_rx = self.stdin_rx.clone();
        let stdout_tx_1 = self.stdout_tx.clone();
        let stdout_tx_2 = self.stdout_tx.clone();
        let mut stdin = self.process.clone().write().await.stdin().unwrap();
        let mut stdout = self.process.clone().write().await.stdout().unwrap();
        let mut buffer = vec![0u8; DEFAULT_POD_BUFFER_SIZE];

        // --- Read from the stdout of the process and send to the SSE stream.
        tokio::spawn(async move {
            loop {
                match stdout.read(&mut buffer).await {
                    Ok(0) => {
                        time::sleep(Duration::from_millis(100)).await;
                    }

                    // --- Read the process stdout and send to the SSE stream.
                    Ok(size) => {
                        // --- Process the buffer line by line
                        let data = &buffer[..size];
                        let data = String::from_utf8_lossy(data).to_string();
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
                            if let JsonRpcMessage::Response(..) = &message {
                                match stdout_tx_1.read().await.send(message.into()) {
                                    Ok(..) => {}
                                    Err(error) => {
                                        tracing::error!("{}", error);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(error) => {
                        tracing::error!("{}", error);
                        break;
                    }
                }
            }

            *is_alive_1.write().await = false;
        });

        // --- Spawn a task to read from the process stdout and send events to the SSE stream.
        tokio::spawn(async move {
            loop {
                match stdin_rx.write().await.recv().await {
                    Ok(request) => {
                        let id = match request.clone() {
                            JsonRpcMessage::Request(request) => request.id,
                            _ => rmcp::model::NumberOrString::Number(u32::MAX),
                        };
                        let message = JSON::to_string(&request).unwrap();
                        let message_str = format!("{message}\n");
                        let message = message_str.as_bytes();
                        match stdin.write_all(message).await {
                            Ok(_) => {}
                            Err(error) => {
                                stdout_tx_2.read().await.send((id, error).into()).unwrap();
                                *is_alive_2.write().await = false;
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error reading from stdin channel: {}", e);
                        break;
                    }
                }
            }
        });

        // --- Set the listening flag to true.
        *self.is_listening.write().await = true;
        Ok(())
    }

    /// Write a message to the stdin of the attached process and read the first message from stdout.
    pub async fn send(&self, request: ClientJsonRpcMessage) -> Result<Option<JsonRpcMessage>> {
        // --- Check if the message is a request.
        let id = match request.clone() {
            JsonRpcMessage::Request(request) => Some(request.id),
            _ => None,
        };

        // --- First, send the message to the stdin channel.
        let stdin_tx = self.stdin_tx.write().await;
        match stdin_tx.send(request.clone()) {
            Ok(..) => {}
            Err(error) => {
                let message = format!("Failed to send message to stdin channel: {error}");
                return Err(Error::Internal(message));
            }
        }

        // --- If the message is a request, wait for the response.
        if let Some(id) = id {
            let timeout = Duration::from_secs(1);
            match self.wait_for_response(id, timeout).await {
                Ok(result) => Ok(Some(result)),
                Err(error) => Err(error),
            }
        } else {
            Ok(None)
        }
    }

    /// Wait for a result from the process stdout with the given ID.
    pub async fn wait_for_response(
        &self,
        id: NumberOrString,
        timeout: Duration,
    ) -> Result<JsonRpcMessage> {
        let mut stdout_rx = self.stdout_rx.write().await;

        // --- Wait for a message from the process stdout.
        let future = async {
            loop {
                match stdout_rx.recv().await {
                    Ok(MCPEvent::Message(message)) => match message.clone() {
                        JsonRpcMessage::Response(response) => {
                            if response.id == id {
                                return Ok(message);
                            }
                        }
                        JsonRpcMessage::Error(response) => {
                            if response.id == id {
                                return Ok(message);
                            }
                        }
                        _ => {}
                    },
                    Err(error) => {
                        let message = format!("Error reading from stdout channel: {error}");
                        tracing::error!("{}", message);
                    }
                    _ => {}
                }
            }
        };

        // --- Wait for the future to complete or timeout.
        match time::timeout(timeout, future).await {
            Ok(result) => result,
            Err(_) => Err(Error::Internal("Timeout waiting for result".into())),
        }
    }

    /// Create a stream of SSE events from the process stdout.
    pub async fn subscribe(&self, endpoint: String) -> BroadcastStream<MCPEvent> {
        let rx = self.stdout_rx.read().await.resubscribe();
        let tx = self.stdout_tx.read().await.clone();

        // --- Write the endpoint to the SSE stream.
        let event = MCPEvent::Endpoint(endpoint);
        let _ = tx.send(event);

        // --- When the stream is closed, send an error event.
        BroadcastStream::new(rx)
    }

    /// Teardown the transport and close the process.
    pub async fn is_alive(&self) -> bool {
        *self.is_alive.read().await
    }
}
