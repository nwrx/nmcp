use crate::{Error, MCPServer, MCPServerTransport, Result};
use kube::Client;
use std::fmt::Debug;
use std::sync::Arc;
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
pub struct Transport(Arc<RwLock<TransportInner>>);

impl Transport {
    pub fn new(client: &Client, server: &MCPServer) -> Result<Self> {
        match server.clone().spec.transport {
            // --- Create a new transport that will proxy the pod's TTY to a BroadcastStream.
            // --- This will allow us to send and receive messages from the pod via SSE.
            MCPServerTransport::Stdio => {
                let transport = TransportAttachedProcess::new(client, server);
                let transport = TransportInner::AttachedProcess(transport);
                let transport = Arc::new(RwLock::new(transport));
                let transport = Self(transport);
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

    pub async fn subscribe(&self) -> Result<TransportPeer> {
        match &mut *self.0.write().await {
            TransportInner::AttachedProcess(transport) => transport.subscribe().await,
        }
    }

    pub async fn get_peer(&self, id: String) -> Result<TransportPeer> {
        match &*self.0.read().await {
            TransportInner::AttachedProcess(transport) => transport.get_peer(&id).await,
        }
    }
}
