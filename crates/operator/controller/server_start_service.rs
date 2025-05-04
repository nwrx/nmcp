use super::Controller;
use crate::{Error, MCPServer, Result};
use k8s_openapi::api::core::v1;
use kube::api::PatchParams;
use kube::Api;

impl Controller {
    /// Create the v1::Service resource for the `MCPServer`.
    ///
    /// This function creates a new Service in the Kubernetes cluster for the MCPServer.
    /// It uses the `server_create_service_patch` method to prepare the Service specification.
    ///
    /// # Arguments
    /// * `server` - A reference to the MCPServer object.
    ///
    /// # Returns
    /// * `Result<v1::Service>` - The created Service.
    ///
    /// # Errors
    /// * `Error::ServerServiceTemplateError` - If there is an error creating the Service.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// let service = controller.server_start_service(&server).await?;
    /// ```
    pub async fn server_start_service(&self, server: &MCPServer) -> Result<v1::Service> {
        let service = v1::Service::default();
        let patch = self.server_create_service_patch(server, service).await;
        let pp = PatchParams::apply("mcp-server");
        Api::namespaced(self.get_client().await, &self.namespace)
            .patch(&server.name_service(), &pp, &patch)
            .await
            .map_err(Error::ServerServiceTemplateError)
    }
}
