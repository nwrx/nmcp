use tracing::{error, info};
use futures::future::join_all;
use crate::{MCPPoolController, MCPPoolControllerConfig, MCPServerController, MCPServerControllerConfig, Result, Program};

impl Program {
    /// Run the UNMCP operator with the configured parameters
    pub async fn operator_start(&self) -> Result<()> {

        info!("Starting UNMCP controller");
        
        // Get Kubernetes client using the extracted method
        let client = self.get_client().await?;

        // Configure controllers
        let pool_config: MCPPoolControllerConfig = MCPPoolControllerConfig {
            reconciliation_interval: self.reconciliation_interval_pool,
            namespace: self.namespace.clone(),
        };
        
        let server_config = MCPServerControllerConfig {
            reconciliation_interval: self.reconciliation_interval_server,
            namespace: self.namespace.clone(),
        };
        
        // Create controllers
        let pool_controller = MCPPoolController::new(client.clone(), Some(pool_config));
        let server_controller = MCPServerController::new(client.clone(), Some(server_config));

        // --- Assert that the CRDs are installed.
        info!("Asserting that CRDs are installed");
        server_controller.assert_crd_exists().await?;
        pool_controller.assert_crd_exists().await?;
        
        // Start controllers in separate tasks
        let pool_task = tokio::spawn(async move {
            if let Err(e) = pool_controller.start().await {
                error!("Pool controller error: {}", e);
            }
        });
        
        let server_task = tokio::spawn(async move {
            if let Err(e) = server_controller.start_operator().await {
                error!("Server controller error: {}", e);
            }
        });
        
        // Wait for all controllers to complete
        // (They should run indefinitely unless there's an error)
        let results = join_all(vec![pool_task, server_task]).await;
        for result in results {
            if let Err(e) = result {
                error!("Controller task error: {}", e);
            }
        }
        
        info!("UNMCP controller shutting down");
        Ok(())
    }
}
