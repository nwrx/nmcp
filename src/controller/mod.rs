use crate::{get_kube_client, Kubeconfig, Result};
use clap::Parser;
use kube::Client;
use std::fmt::Debug;

mod into_pod;
mod into_resource;
mod into_service;
mod manager;
mod operator;
mod status;

pub use into_resource::*;
pub use manager::*;

/// The name of the Kubernetes operator manager. Used to identify the operator in the Kubernetes API.
pub const MCP_SERVER_OPERATOR_MANAGER: &str = "mcpserver.nmcp.nwrx.io/operator";

/// The finalizer name for `MCPServer` resources. This is used to ensure that the operator cleans up
/// resources before deleting the `MCPServer` and it's associated resources.
pub const MCP_SERVER_FINALIZER: &str = "mcpserver.nmcp.nwrx.io/finalizer";

/// Configuration for the Kubernetes operator
#[derive(Debug, Clone, Parser, Default)]
pub struct ControllerOptions {
    /// Namespace to operate in.
    #[arg(short, long, default_value = "default", env = "KUBECTL_NAMESPACE")]
    pub namespace: String,

    /// Path to Kubernetes config file.
    #[arg(short, long, env = "KUBECONFIG")]
    pub kubeconfig: Kubeconfig,
}

#[derive(Clone)]
pub struct Controller {
    client: Client,
    namespace: String,
}

impl Debug for Controller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Controller")
            .field("namespace", &self.namespace)
            .field("client", &"Client(...)")
            .finish()
    }
}

impl Controller {
    pub async fn new(options: &ControllerOptions) -> Result<Self> {
        Ok(Self {
            namespace: options.namespace.clone(),
            client: get_kube_client(options.kubeconfig.clone()).await?,
        })
    }

    pub fn get_client(&self) -> Client {
        self.client.clone()
    }

    pub fn get_namespace(&self) -> String {
        self.namespace.clone()
    }
}
