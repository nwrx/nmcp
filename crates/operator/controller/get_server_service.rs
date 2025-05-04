use super::Controller;
use crate::{Error, MCPServer, Result};
use k8s_openapi::api::core::v1;
use kube::Api;

impl Controller {
    /// Get the `v1::Service` associated with the `MCPServer`.
    ///
    /// This function retrieves the Service associated with the MCPServer.
    ///
    /// # Arguments
    /// * `server` - A reference to the MCPServer object.
    ///
    /// # Returns
    /// * `Result<v1::Service>` - The Service associated with the MCPServer.
    ///
    /// # Errors
    /// * `Error::ServerServiceNotFoundError` - If the Service is not found.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// let service = controller.get_server_service(&server).await?;
    /// ```
    pub async fn get_server_service(&self, server: &MCPServer) -> Result<v1::Service> {
        let client = self.get_client().await;
        Api::namespaced(client, &self.namespace)
            .get(&server.name_service())
            .await
            .map_err(Error::ServerServiceNotFoundError)
    }
}
