use crate::{get_kube_client, Kubeconfig, MCPServerTransportStdio, Result};
use clap::Parser;
use kube::Client;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

mod manager;
mod operator;
mod status;

/// The name of the Kubernetes operator manager. Used to identify the operator in the Kubernetes API.
pub const MCP_SERVER_OPERATOR_MANAGER: &str = "mcpserver.nmcp.nwrx.io/operator";

/// The finalizer name for MCPServer resources. This is used to ensure that the operator cleans up
/// resources before deleting the MCPServer and it's associated resources.
pub const MCP_SERVER_FINALIZER: &str = "mcpserver.nmcp.nwrx.io/finalizer";

/// Configuration for the Kubernetes operator
#[derive(Debug, Clone, Parser, Default)]
pub struct ControllerOptions {
    /// Namespace to operate in.
    #[arg(short, long, default_value = "default")]
    pub namespace: String,

    /// Path to Kubernetes config file.
    #[arg(short, long)]
    pub kubeconfig: Kubeconfig,
}

#[derive(Clone)]
pub struct Controller {
    client: Client,
    namespace: String,
    transports: Arc<RwLock<HashMap<String, Arc<RwLock<MCPServerTransportStdio>>>>>,
}

impl Controller {
    /// Create a new instance of the Controller.
    pub async fn new(options: &ControllerOptions) -> Result<Self> {
        Ok(Self {
            namespace: options.namespace.clone(),
            client: get_kube_client(options.kubeconfig.clone()).await?,
            transports: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get the Kubernetes client.
    pub fn get_client(&self) -> Client {
        self.client.clone()
    }

    /// Get the namespace of the controller.
    pub fn get_namespace(&self) -> String {
        self.namespace.clone()
    }
}
