use super::Controller;
use crate::{MCPServer, Result};
use futures::Stream;
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, AttachParams, AttachedProcess};
use std::fmt::{Display, Formatter};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;
use tracing::info;

/// The event type for SSE messages from the TTY stream
pub enum MCPServerStreamEvent {
    /// A log line from the container stdout
    Message(String),
    /// Signal that the stream has ended
    End,
    /// Signal that user input should be sent to the pod
    Input(String),
}

impl Display for MCPServerStreamEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MCPServerStreamEvent::Message(data) => write!(f, "event: message\ndata: {data}"),
            _ => write!(f, "Other event"),
        }
    }
}

impl Controller {
    pub async fn attach_to_server(&self, server: &MCPServer) -> Result<AttachedProcess> {
        let pod_name = server.name_pod();
        let client = self.get_client();
        let api: Api<Pod> = Api::namespaced(client, &self.namespace);
        let ap = AttachParams::interactive_tty().container("server");
        let attached = api.attach(&pod_name, &ap).await?;
        info!("Attached to pod: {}", pod_name);
        Ok(attached)
    }

    /// Attaches to a pod's TTY and returns a stream of events for SSE.
    ///
    /// This function connects to the TTY of the pod associated with the MCPServer
    /// and converts the output into a Stream of MCPServerStreamEvent items, which can be used
    /// to create a Server-Sent Events (SSE) endpoint.
    ///
    /// # Arguments
    /// * `server` - A reference to the MCPServer object.
    ///
    /// # Returns
    /// * `Result<impl Stream<Item = MCPServerStreamEvent>>` - A stream of events from the TTY.
    ///
    /// # Errors
    /// * `Error::ServerPodNotFoundError` - If the pod is not found.
    /// * `Error::ServerTtyError` - If there's an error attaching to the TTY.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// let event_stream = controller.get_server_stream(&server).await?;
    /// ```
    pub async fn get_server_stream(
        &self,
        server: &MCPServer,
    ) -> Result<impl Stream<Item = MCPServerStreamEvent>> {
        let mut attached = self.attach_to_server(server).await?;
        let mut stdout = attached.stdout().unwrap();
        let mut stdin = attached.stdin().unwrap();

        // Using RMCP, we can create a transport for the stdin and stdout streams.
        // let stdin = stdin.into_transport().await?;

        // Create channels for the streams
        let (tx, rx) = tokio::sync::mpsc::channel(1024);

        // Spawn a task to read from stdout
        let tx_stdout = tx.clone();
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            loop {
                let bytes = stdout.read(&mut buffer).await;
                match bytes {
                    Ok(n) if n > 0 => {
                        let output = String::from_utf8_lossy(&buffer[..n]).to_string();
                        if tx_stdout
                            .send(MCPServerStreamEvent::Message(output))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Ok(_) => {
                        sleep(Duration::from_secs(1)).await;
                    }
                    Err(_) => {
                        // Error reading
                        let _ = tx_stdout.send(MCPServerStreamEvent::End).await;
                        break;
                    }
                }
            }
        });

        // Create a separate writer task that will handle writing to stdin
        let tx_writer = tx.clone();
        tokio::spawn(async move {
            // For now, we're not using the input channel as we have a dedicated API for this
            // This could be enhanced in future to support bidirectional communication
            let (_, mut input_rx) = tokio::sync::mpsc::channel::<String>(32);

            // Process incoming stdin messages
            while let Some(input) = input_rx.recv().await {
                match stdin.write_all(input.as_bytes()).await {
                    Ok(_) => {
                        info!("Successfully wrote input to stdin: {}", input);
                    }
                    Err(e) => {
                        info!("Error writing to stdin: {:?}", e);
                        break;
                    }
                }
            }

            // If the input channel closed, signal end of stream
            let _ = tx_writer.send(MCPServerStreamEvent::End).await;
        });

        // Send the initial message to stdin
        // let payload: JsonRpcRequest = JsonRpcRequest {
        //     id: NumberOrString::Number(1),
        //     request: Request {
        //         method: "hello".to_string(),
        //         params: None,
        //     },
        //     jsonrpc: JsonRpcVersion2_0,
        // };
        // let payload_str = serde_json::to_string(&payload).unwrap();
        // let _ = stdin.write_all(payload_str.as_bytes()).await;

        // Return the stream
        Ok(ReceiverStream::new(rx))
    }

    /// Sends input to an active pod stream
    ///
    /// This function sends the provided input to the stdin of a pod.
    ///
    /// # Arguments
    /// * `server` - A reference to the MCPServer object.
    /// * `input` - The input string to send to the pod's stdin.
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the input was sent successfully.
    ///
    /// # Errors
    /// * `Error::ServerPodNotFoundError` - If the pod is not found.
    /// * `Error::ServerTtyError` - If there's an error attaching to the TTY.
    pub async fn send_to_server_stdin(&self, server: &MCPServer, input: String) -> Result<()> {
        let pod_name = server.name_pod();
        info!("Sending input to pod: {}", pod_name);

        // Create API client for the Pod
        let client = self.get_client();
        let api: Api<Pod> = Api::namespaced(client, &self.namespace);
        let ap = AttachParams::interactive_tty().container("server");
        let mut attached = api.attach(&pod_name, &ap).await?;

        // Get the stdin stream and write to it
        if let Some(mut stdin) = attached.stdin() {
            match stdin.write_all(input.as_bytes()).await {
                Ok(_) => {
                    info!("Successfully sent input to pod: {}", pod_name);
                    Ok(())
                }
                Err(e) => {
                    info!("Error writing to stdin: {:?}", e);
                    Err(crate::Error::ServerStreamError(kube::Error::Api(
                        kube::error::ErrorResponse {
                            status: "Failed".to_string(),
                            message: format!("Failed to write to stdin: {e}"),
                            reason: "InternalError".to_string(),
                            code: 500,
                        },
                    )))
                }
            }
        } else {
            info!("No stdin stream available for pod: {}", pod_name);
            Err(crate::Error::ServerStreamError(kube::Error::Api(
                kube::error::ErrorResponse {
                    status: "Failed".to_string(),
                    message: "No stdin stream available".to_string(),
                    reason: "InternalError".to_string(),
                    code: 500,
                },
            )))
        }
    }
}
