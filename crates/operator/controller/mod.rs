use crate::{get_kube_client, MCPServerTransportStdio, Result};
use kube::Client;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use structopt::StructOpt;
use tokio::sync::RwLock;

mod manager;
mod operator;
mod status;

/// The name of the Kubernetes operator manager. Used to identify the operator in the Kubernetes API.
pub const MCP_SERVER_OPERATOR_MANAGER: &str = "mcpserver.unmcp.dev/operator";

/// The finalizer name for MCPServer resources. This is used to ensure that the operator cleans up
/// resources before deleting the MCPServer and it's associated resources.
pub const MCP_SERVER_FINALIZER: &str = "mcpserver.unmcp.dev/finalizer";

/// Configuration for the Kubernetes operator
#[derive(Debug, Clone, StructOpt, Default)]
pub struct ControllerOptions {
    /// Log level (debug, info, warn, error)
    #[structopt(short, long, default_value = "info", env = "LOG_LEVEL")]
    pub log_level: String,

    /// Namespace to watch (default: all namespaces)
    #[structopt(short, long, default_value = "default", env = "KUBECTL_NAMESPACE")]
    pub namespace: String,

    /// Path to kubeconfig file (uses in-cluster config if not specified)
    #[structopt(short, long, env = "KUBECONFIG")]
    pub kubeconfig: Option<PathBuf>,
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
