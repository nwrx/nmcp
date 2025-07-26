use crate::{Error, MCPServer, MCPServerTransport, Result};
use kube::Client;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
mod transport_peer;
mod transport_stdio;

pub use transport_peer::*;
pub use transport_stdio::*;

pub enum TransportInner {
    AttachedProcess(TransportAttachedProcess),
}

impl Debug for TransportInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AttachedProcess(_) => write!(f, "TransportInner::AttachedProcess"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Transport {
    inner: Arc<RwLock<TransportInner>>,
    pub created_at: Instant,
    pub last_accessed: Arc<RwLock<Instant>>,
}

impl Transport {
    pub fn new(client: &Client, server: &MCPServer) -> Result<Self> {
        match server.clone().spec.transport {
            // --- Create a new transport that will proxy the pod's TTY to a BroadcastStream.
            // --- This will allow us to send and receive messages from the pod via SSE.
            MCPServerTransport::Stdio => {
                let transport = TransportAttachedProcess::new(client, server);
                let transport = TransportInner::AttachedProcess(transport);
                let transport = Arc::new(RwLock::new(transport));
                let transport = Self {
                    inner: transport,
                    created_at: Instant::now(),
                    last_accessed: Arc::new(RwLock::new(Instant::now())),
                };
                Ok(transport)
            }

            // --- Create a new transport that will proxy the pod's HTTP stream to a BroadcastStream.
            MCPServerTransport::Sse { .. } => {
                Err(Error::generic("SSE transport is not supported yet"))
            }
            MCPServerTransport::StreamableHttp { .. } => Err(Error::generic(
                "Streamable HTTP transport is not supported yet",
            )),
        }
    }

    /// Record the last access time for the transport. This allows us to track when the transport was last used,
    /// and clean up idle transports if necessary based on the configured idle timeout.
    pub async fn touch(&self) {
        let mut last_accessed = self.last_accessed.write().await;
        *last_accessed = Instant::now();
    }

    /// Get the age of the transport since it was created. This is useful for monitoring how long the transport
    /// has been active and may help in determining if it should be cleaned up or not.
    pub fn age(&self) -> Duration {
        Instant::now().duration_since(self.created_at)
    }

    /// Get the idle time of the transport since it was last accessed. This is useful for determining if the transport
    /// has been idle for too long and may need to be cleaned up.
    pub async fn idle_time(&self) -> Duration {
        let last_accessed = self.last_accessed.read().await;
        Instant::now().duration_since(*last_accessed)
    }

    pub async fn subscribe(&mut self) -> Result<TransportPeer> {
        self.touch().await;
        match &mut *self.inner.write().await {
            TransportInner::AttachedProcess(transport) => transport.subscribe().await,
        }
    }

    pub async fn get_peer(&self, id: String) -> Result<TransportPeer> {
        self.touch().await;
        match &*self.inner.read().await {
            TransportInner::AttachedProcess(transport) => transport.get_peer(&id).await,
        }
    }

    pub async fn close(&mut self) -> Result<()> {
        match &mut *self.inner.write().await {
            TransportInner::AttachedProcess(transport) => transport.close().await,
        }
    }
}
