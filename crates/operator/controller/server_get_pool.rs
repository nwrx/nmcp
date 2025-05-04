use super::Controller;
use crate::{Error, MCPPool, MCPServer, Result};
use kube::Api;

impl Controller {
    /// Get the `MCPPool` associated with the `MCPServer`.
    ///
    /// This function retrieves the MCPPool that this MCPServer belongs to.
    ///
    /// # Arguments
    /// * `server` - A reference to the MCPServer object.
    ///
    /// # Returns
    /// * `Result<MCPPool>` - The MCPPool associated with the MCPServer.
    ///
    /// # Errors
    /// * `Error::ServerPoolNotFoundError` - If the MCPPool is not found.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// let pool = controller.server_get_pool(&server).await?;
    /// ```
    pub async fn server_get_pool(&self, server: &MCPServer) -> Result<MCPPool> {
        let client = self.get_client().await;
        Api::namespaced(client, &self.namespace)
            .get(&server.spec.pool)
            .await
            .map_err(Error::ServerPoolNotFoundError)
    }
}
