use crate::{get_kube_client, Result};
use kube::Client;
use std::{path::PathBuf, sync::Arc};
use structopt::StructOpt;
use tokio::sync::RwLock;
use tracing::info;

mod create_server_pod_patch;
mod create_server_service_patch;
mod get_server_pod;
mod get_server_pool;
mod get_server_service;
mod is_server_up;
mod list_pools;
mod list_servers;
mod reconcile_server;
mod should_server_be_up;
mod start_server;
mod start_server_operator;
mod start_server_pod;
mod start_server_service;
mod stop_server;
mod stop_server_pod;
mod stop_server_service;

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
