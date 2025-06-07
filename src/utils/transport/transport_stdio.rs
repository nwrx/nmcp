use super::TransportPeer;
use crate::{Error, MCPServer, Result, MCP_SERVER_CONTAINER_NAME};
use crate::{IntoResource, DEFAULT_POD_BUFFER_SIZE};
use k8s_openapi::api::core::v1;
use kube::api::{AttachParams, AttachedProcess};
use kube::{Api, Client};
use rmcp::model::{ClientJsonRpcMessage, JsonRpcMessage};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;

/// A transport for communicating with a process via stdin/stdout
pub struct TransportAttachedProcess {
    client: Client,
    server: MCPServer,
    peers: Arc<RwLock<HashMap<String, TransportPeer>>>,

    stdin_rx: broadcast::Receiver<ClientJsonRpcMessage>,
    stdin_tx: broadcast::Sender<ClientJsonRpcMessage>,
    stdout_tx: broadcast::Sender<JsonRpcMessage>,
    stdout_rx: broadcast::Receiver<JsonRpcMessage>,

    task_attach_stdin: Option<JoinHandle<Result<()>>>,
    task_attach_stdout: Option<JoinHandle<Result<()>>>,
    task_attach_stderr: Option<JoinHandle<Result<()>>>,
}

impl Debug for TransportAttachedProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransportAttachedProcess")
            .field("server", &self.server)
            .field("task", &self.task_attach_stdout)
            .field("peers", &self.peers.blocking_read().len())
            .finish()
    }
}

impl TransportAttachedProcess {
    pub fn new(client: &Client, server: &MCPServer) -> Self {
        let (stdin_tx, stdin_rx) = broadcast::channel(DEFAULT_POD_BUFFER_SIZE);
        let (stdout_tx, stdout_rx) = broadcast::channel(DEFAULT_POD_BUFFER_SIZE);
        Self {
            client: client.clone(),
            server: server.clone(),
            peers: Arc::new(RwLock::new(HashMap::new())),
            stdin_tx,
            stdin_rx,
            stdout_tx,
            stdout_rx,
            task_attach_stdin: None,
            task_attach_stdout: None,
            task_attach_stderr: None,
        }
    }

    /// Attach to the process stdout.
    async fn attach_stdout<T>(&mut self, mut stdout: T) -> JoinHandle<Result<()>>
    where
        T: AsyncReadExt + Send + Unpin + 'static,
    {
        let stdout_tx = self.stdout_tx.clone();
        tokio::spawn(async move {
            let mut buffer = vec![0u8; DEFAULT_POD_BUFFER_SIZE];
            loop {
                match stdout.read(&mut buffer).await {
                    Ok(0) => {}
                    Ok(size) => {
                        let data = buffer.get(..size).expect("Failed to get data from buffer");
                        let data = String::from_utf8_lossy(data).to_string();
                        for line in data.lines() {
                            if !line.trim().is_empty() {
                                let message: JsonRpcMessage = match serde_json::from_str(line) {
                                    Ok(message) => message,
                                    Err(error) => {
                                        tracing::error!(
                                            "[STDOUT/ERROR]: Failed to parse message: {}",
                                            error
                                        );
                                        continue;
                                    }
                                };
                                tracing::info!("[STDOUT]: {:?}", message.clone());
                                let _ = stdout_tx.send(message).map_err(Error::from)?;
                            }
                        }
                    }
                    Err(error) => {
                        let error = Error::from(error).trace();
                        return Err(error);
                    }
                }
            }
        })
    }

    /// Attach to the process stderr.
    async fn attach_stderr<T>(&mut self, mut stderr: T) -> JoinHandle<Result<()>>
    where
        T: AsyncReadExt + Send + Unpin + 'static,
    {
        let stdout_tx = self.stdout_tx.clone();
        tokio::spawn(async move {
            let mut buffer = vec![0u8; DEFAULT_POD_BUFFER_SIZE];
            loop {
                match stderr.read(&mut buffer).await {
                    Ok(0) => {}
                    Ok(size) => {
                        let data = buffer.get(..size).expect("Failed to get data from buffer");
                        let data = String::from_utf8_lossy(data).to_string();
                        for line in data.lines() {
                            if !line.trim().is_empty() {
                                let message: JsonRpcMessage = serde_json::from_str(line)?;
                                let _ = stdout_tx.send(message).map_err(Error::from)?;
                            }
                        }
                    }
                    Err(error) => {
                        let error = Error::from(error).trace();
                        return Err(error);
                    }
                }
            }
        })
    }

    /// Attach to the process stdin.
    async fn attach_stdin<T>(&mut self, mut stdin: T) -> JoinHandle<Result<()>>
    where
        T: AsyncWriteExt + Send + Unpin + 'static,
    {
        let mut stdin_rx = self.stdin_rx.resubscribe();
        tokio::spawn(async move {
            loop {
                match stdin_rx.recv().await {
                    Ok(message) => {
                        let data = serde_json::to_string(&message)?;
                        let data = format!("{data}\n");
                        let data = data.as_bytes();
                        if let Err(error) = stdin.write_all(data).await {
                            let error = Error::from(error).trace();
                            return Err(error);
                        }
                    }
                    Err(error) => match error {
                        broadcast::error::RecvError::Lagged(_) => {
                            tracing::warn!("Stdin receiver lagged, some messages may be lost");
                        }
                        broadcast::error::RecvError::Closed => {
                            tracing::info!("Stdin receiver closed, stopping stdin task");
                            return Ok(());
                        }
                    },
                }
            }
        })
    }

    #[tracing::instrument(name = "AttachToProcess", skip_all)]
    async fn attach_to_process(&mut self) -> Result<AttachedProcess> {
        Api::<v1::Pod>::namespaced(self.client.clone(), self.client.default_namespace())
            .attach(
                &<MCPServer as IntoResource<v1::Pod>>::resource_name(&self.server),
                &AttachParams::default()
                    .tty(false)
                    .stdin(true)
                    .stdout(true)
                    .stderr(true)
                    .container(MCP_SERVER_CONTAINER_NAME)
                    .max_stdin_buf_size(DEFAULT_POD_BUFFER_SIZE)
                    .max_stdout_buf_size(DEFAULT_POD_BUFFER_SIZE)
                    .max_stderr_buf_size(DEFAULT_POD_BUFFER_SIZE),
            )
            .await
            .map_err(Error::from)
    }

    #[tracing::instrument(name = "IsAttached", skip_all)]
    async fn is_attached(&self) -> bool {
        let is_stdout_attached = self.task_attach_stdout.is_some()
            && !self.task_attach_stdout.as_ref().unwrap().is_finished();
        let is_stdin_attached = self.task_attach_stdin.is_some()
            && !self.task_attach_stdin.as_ref().unwrap().is_finished();
        is_stdout_attached && is_stdin_attached
    }

    #[tracing::instrument(name = "BindStreams", skip_all)]
    async fn bind_streams(&mut self) -> Result<&mut Self> {
        if self.is_attached().await {
            return Ok(self);
        }
        if let Some(task) = self.task_attach_stdout.take() {
            task.abort();
        }
        if let Some(task) = self.task_attach_stdin.take() {
            task.abort();
        }
        if let Some(task) = self.task_attach_stderr.take() {
            task.abort();
        }

        let mut process = self.attach_to_process().await?;
        let stdin = process.stdin().unwrap();
        let stdout = process.stdout().unwrap();
        let stderr = process.stderr().unwrap();

        // --- Attach the stdout and stdin to the transport.
        self.task_attach_stdin = Some(self.attach_stdout(stdout).await);
        self.task_attach_stdout = Some(self.attach_stdin(stdin).await);
        self.task_attach_stderr = Some(self.attach_stderr(stderr).await);

        // --- Spawn the task to read from stdin.
        Ok(self)
    }

    /// Get a peer by ID.
    #[tracing::instrument(name = "GetPeer", skip_all)]
    pub async fn get_peer(&self, id: &String) -> Result<TransportPeer> {
        match self.peers.read().await.get(id) {
            Some(peer) => Ok(peer.clone()),
            None => Err(Error::generic(format!("Peer with ID {id} not found"))),
        }
    }

    /// Create a stream of SSE events from the process stdout.
    #[tracing::instrument(name = "Subscribe", skip_all)]
    pub async fn subscribe(&mut self) -> Result<TransportPeer> {
        let _ = self.bind_streams().await?;

        // --- Create a new peer for the transport.
        let peer = {
            let peer = TransportPeer::new();
            let id = peer.id.clone();
            let mut peers = self.peers.write().await;
            let _ = peers.insert(id, peer.clone());
            drop(peers);
            peer
        };

        // --- Connect the stdin and stdout channels to the peer.
        peer.attach_stdin(self.stdin_tx.clone()).await?;
        peer.attach_stdout(self.stdout_rx.resubscribe()).await?;

        Ok(peer)
    }
}
