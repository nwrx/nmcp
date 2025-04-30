use tokio::sync::RwLock;
use kube::client::Client;
use std::{sync::Arc, time::Duration};

/// Configuration for the MCPPool controller
#[derive(Clone)]
pub struct MCPPoolControllerConfig {
    /// Optional namespace to watch (if None, watch all namespaces)
    pub namespace: Option<String>,

    /// Interval between reconciliations
    pub reconciliation_interval: Duration,
}

impl Default for MCPPoolControllerConfig {
    /// Creates a default configuration for the MCPPool controller.
    ///
    /// # Returns
    /// A new `MCPPoolControllerConfig` with default values:
    /// - namespace: None (watch all namespaces)
    /// - reconciliation_interval: 10 seconds
    fn default() -> Self {
        Self {
            namespace: None,
            reconciliation_interval: Duration::from_secs(10),
        }
    }
}

#[derive(Clone)]
pub struct MCPPoolContext {
    /// The "Kube" Kubernetes client used to interact with the Kubernetes API
    /// and perform CRUD operations on resources.
    pub client: Client,
    
    /// Controller configuration used to customize the behavior of the controller.
    /// This includes settings such as the namespace to watch and the reconciliation interval.
    /// The configuration is passed to the controller at runtime and can be modified
    /// to change the controller's behavior without needing to restart the controller.
    pub config: MCPPoolControllerConfig,
}

/// MCPPool controller implementation
pub struct MCPPoolController {
    /// Controller context
    pub context: Arc<RwLock<MCPPoolContext>>,
}

impl MCPPoolController {
    /// Creates a new instance of `MCPPoolController`.
    ///
    /// # Parameters
    /// - `client`: An instance of `Client` that will be used to interact with the Kubernetes API.
    /// - `config`: An optional instance of `MCPPoolControllerConfig` containing configuration details for the pool controller.
    ///
    /// # Returns
    /// A new `MCPPoolController` instance.
    ///
    /// # Details
    /// This function initializes the pool controller by creating a shared context that holds the provided
    /// `client` and `config`. The context is wrapped in a `RwLock` to allow safe concurrent access, enabling
    /// multiple readers or a single writer at a time. The `RwLock` is further wrapped in an `Arc` (atomic reference
    /// counter) to allow the context to be shared across threads while maintaining thread safety.
    pub fn new(client: Client, config: Option<MCPPoolControllerConfig>) -> Self {
        let config = config.unwrap_or_default();
        let context_raw = MCPPoolContext { client, config };
        let context = Arc::new(RwLock::new(context_raw));
        Self { context }
    }

    /// Updates the MCPPool controller context with a new configuration.
    ///
    /// # Parameters
    /// - `config`: An instance of `MCPPoolControllerConfig` containing the new configuration details.
    ///
    /// # Details
    /// This function updates the existing context with the new configuration. It uses a write lock to ensure
    /// that no other threads are accessing the context while the update is being made. The `RwLock` allows
    /// multiple readers or a single writer at a time, ensuring thread safety during the update process.
    pub async fn update_context(&self, config: MCPPoolControllerConfig) {
        let mut context = self.context.write().await;
        context.config = config;
    }

    /// Retrieves the current configuration of the MCPPool controller.
    ///
    /// # Returns
    /// An instance of `MCPPoolControllerConfig` containing the current configuration details.
    ///
    /// # Details
    /// This function reads the current configuration from the context. It uses a read lock to ensure that
    /// no other threads are modifying the context while the read operation is being performed. The `RwLock`
    /// allows multiple readers or a single writer at a time, ensuring thread safety during the read operation.
    pub async fn get_context(&self) -> MCPPoolControllerConfig {
        let context = self.context.read().await;
        context.config.clone()
    }

    /// Retrieves the Kubernetes client from the MCPPool controller context.
    ///
    /// # Returns
    /// An instance of `Client` that can be used to interact with the Kubernetes API.
    ///
    /// # Details
    /// This function reads the client from the context. It uses a read lock to ensure that no other threads
    /// are modifying the context while the read operation is being performed. The `RwLock` allows multiple
    /// readers or a single writer at a time, ensuring thread safety during the read operation.
    pub async fn get_client(&self) -> Client {
        let context = self.context.read().await;
        context.client.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestContext;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_mcp_pool_controller_config_default() {
        let config = MCPPoolControllerConfig::default();
        assert_eq!(config.namespace, None);
        assert_eq!(config.reconciliation_interval, Duration::from_secs(10));
    }

    #[test]
    fn test_mcp_pool_controller_config_custom() {
        let config = MCPPoolControllerConfig {
            namespace: Some("test-namespace".to_string()),
            reconciliation_interval: Duration::from_secs(20),
        };
        assert_eq!(config.namespace, Some("test-namespace".to_string()));
        assert_eq!(config.reconciliation_interval, Duration::from_secs(20));
    }

    #[tokio::test]
    async fn test_mcp_pool_controller_new() {
        let context = TestContext::new().unwrap();
        let client = context.get_client().await.unwrap();
        let config: MCPPoolControllerConfig = MCPPoolControllerConfig::default();
        let controller = MCPPoolController::new(client.clone(), Some(config.clone()));
        let context = controller.context.read().await;
        assert_eq!(context.config.namespace, config.namespace);
        assert_eq!(context.config.reconciliation_interval, config.reconciliation_interval);
    }

    #[tokio::test]
    async fn test_mcp_pool_controller_update_context() {
        let context = TestContext::new().unwrap();
        let client = context.get_client().await.unwrap();
        let config: MCPPoolControllerConfig = MCPPoolControllerConfig::default();
        let controller = MCPPoolController::new(client.clone(), Some(config.clone()));
        let new_config = MCPPoolControllerConfig {
            namespace: Some("new-namespace".to_string()),
            reconciliation_interval: Duration::from_secs(30),
        };
        controller.update_context(new_config.clone()).await;
        let context = controller.context.read().await;
        assert_eq!(context.config.namespace, new_config.namespace);
        assert_eq!(context.config.reconciliation_interval, new_config.reconciliation_interval);
    }

    #[tokio::test]
    async fn test_mcp_pool_controller_get_context() {
        let context = TestContext::new().unwrap();
        let client = context.get_client().await.unwrap();
        let config: MCPPoolControllerConfig = MCPPoolControllerConfig::default();
        let controller = MCPPoolController::new(client.clone(), Some(config.clone()));
        let context = controller.get_context().await;
        assert_eq!(context.namespace, config.namespace);
        assert_eq!(context.reconciliation_interval, config.reconciliation_interval);
    }

    #[tokio::test]
    async fn test_mcp_pool_controller_get_client() {
        let context = TestContext::new().unwrap();
        let client = context.get_client().await.unwrap();
        let config: MCPPoolControllerConfig = MCPPoolControllerConfig::default();
        let controller = MCPPoolController::new(client.clone(), Some(config.clone()));
        let client_from_controller = controller.get_client().await;
        assert_eq!(
            client.apiserver_version().await.unwrap(),
            client_from_controller.apiserver_version().await.unwrap()
        );
    }
}
