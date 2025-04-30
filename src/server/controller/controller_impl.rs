use tokio::sync::RwLock;
use kube::client::Client;
use std::{sync::Arc, time::Duration};

/// Configuration for the MCPServer controller
#[derive(Clone)]
pub struct MCPServerControllerConfig {
    
    /// Optional namespace to watch (if None, watch all namespaces)
    pub namespace: Option<String>,

    /// Interval between reconciliations
    pub reconciliation_interval: Duration,
}

impl Default for MCPServerControllerConfig {
    fn default() -> Self {
        Self {
            namespace: None,
            reconciliation_interval: Duration::from_secs(30), // Default 30 seconds
        }
    }
}

/// MCPServer controller context
#[derive(Clone)]
pub struct MCPServerContext {
    /// Kubernetes client
    pub client: Client,
    
    /// Controller configuration
    pub config: MCPServerControllerConfig,
}

/// MCPServer controller implementation
pub struct MCPServerController {
    /// Controller context
    pub context: Arc<RwLock<MCPServerContext>>,
}

impl MCPServerController {

    /// Creates a new instance of `MCPServerController`.
    ///
    /// # Parameters
    /// - `client`: An instance of `Client` that will be used to interact with the server.
    /// - `config`: An optional instance of `MCPServerControllerConfig` containing configuration details 
    ///   for the server controller. If not provided, default configuration will be used.
    ///
    /// # Returns
    /// A new `MCPServerController` instance.
    ///
    /// # Details
    /// This function initializes the server controller by creating a shared context that holds the provided
    /// `client` and `config`. The context is wrapped in a `RwLock` to allow safe concurrent access, enabling
    /// multiple readers or a single writer at a time. The `RwLock` is further wrapped in an `Arc` (atomic reference
    /// counter) to allow the context to be shared across threads while maintaining thread safety.
    ///
    /// The use of `RwLock` ensures that the context can be read by multiple threads simultaneously, but only one
    /// thread can modify it at a time, preventing data races and ensuring consistency.
    pub fn new(client: Client, config: Option<MCPServerControllerConfig>) -> Self {
        let config = config.unwrap_or_default();
        let context_raw = MCPServerContext { client: client.clone(), config };
        let context_lock = RwLock::new(context_raw);
        let context = Arc::new(context_lock);
        Self { context }
    }
}
