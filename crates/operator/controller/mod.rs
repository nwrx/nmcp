use crate::{get_kube_client, Result};
use kube::Client;
use std::path::PathBuf;
use structopt::StructOpt;
use tracing_subscriber::fmt::format::FmtSpan;

mod getter;
mod manager;
mod operator;
mod stream;

/// Re-export the necessary modules for external use.
pub use stream::MCPServerStreamEvent;

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
    namespace: String,
    client: Client,
}

impl Controller {
    /// Create a new instance of the Controller.
    pub async fn new(options: &ControllerOptions) -> Result<Self> {
        let client = get_kube_client(options.kubeconfig.clone()).await?;
        Ok(Self {
            namespace: options.namespace.clone(),
            client: client.clone(),
        })
    }

    /// Start the tracing subscriber for logging.
    pub fn start_tracing(&self) {
        let format = tracing_subscriber::fmt::format()
            .with_level(true)
            .with_target(false)
            .without_time()
            .with_file(true)
            .with_line_number(true)
            .compact();

        // Create a filter that excludes reconciler retry messages
        let fmt_fields = tracing_subscriber::fmt::format::debug_fn(|writer, _, value| {
            write!(writer, "\n\t{value:?}")
        });

        tracing_subscriber::fmt()
            .with_line_number(true)
            .with_span_events(FmtSpan::NONE)
            .with_level(true)
            .fmt_fields(fmt_fields)
            .event_format(format)
            .init();
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
