use crate::{get_kube_client, Result};
use kube::Client;
use std::{path::PathBuf, sync::Arc};
use structopt::StructOpt;
use tokio::sync::RwLock;
use tracing::info;

mod get_server_service;
mod list_servers;
mod server_create_pod_patch;
mod server_create_service_patch;
mod server_down;
mod server_get_pod;
mod server_get_pool;
mod server_is_up;
mod server_reconciler;
mod server_should_be_up;
mod server_start_operator;
mod server_start_pod;
mod server_start_service;
mod server_stop_pod;
mod server_stop_service;
mod server_up;

/// Configuration for the Kubernetes operator
#[derive(Debug, Clone, StructOpt)]
pub struct ControllerOptions {
    /// Log level (debug, info, warn, error)
    #[structopt(long, default_value = "info", env = "LOG_LEVEL")]
    pub log_level: String,

    /// Namespace to watch (default: all namespaces)
    #[structopt(long, default_value = "default", env = "KUBECTL_NAMESPACE")]
    pub namespace: String,

    /// Path to kubeconfig file (uses in-cluster config if not specified)
    #[structopt(long, env = "KUBECONFIG")]
    pub kubeconfig: Option<PathBuf>,
}

#[derive(Clone)]
pub struct Controller {
    namespace: String,
    client: Arc<RwLock<Client>>,
}

impl Controller {
    pub async fn new(options: &ControllerOptions) -> Result<Self> {
        tracing_subscriber::fmt::init();
        info!("Starting UNMCP Operator");

        let client = get_kube_client(options.kubeconfig.clone()).await?;
        Ok(Self {
            namespace: options.namespace.clone(),
            client: Arc::new(RwLock::new(client)),
        })
    }

    async fn get_client(&self) -> Client {
        self.client.read().await.clone()
    }
}
