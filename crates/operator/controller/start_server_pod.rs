use super::Controller;
use crate::{Error, MCPServer, Result};
use k8s_openapi::api::core::v1;
use kube::api::PatchParams;
use kube::Api;

impl Controller {
    /// Create the `v1::Pod` resource for the `MCPServer`.
    ///
    /// This function creates a new Pod in the Kubernetes cluster for the MCPServer.
    /// It uses the `server_create_pod_patch` method to prepare the Pod specification.
    ///
    /// # Arguments
    /// * `server` - A reference to the MCPServer object.
    ///
    /// # Returns
    /// * `Result<v1::Pod>` - The created Pod.
    ///
    /// # Errors
    /// * `Error::ServerPodTemplateError` - If there is an error creating the Pod.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// let pod = controller.start_server_pod(&server).await?;
    /// ```
    pub async fn start_server_pod(&self, server: &MCPServer) -> Result<v1::Pod> {
        let pod = v1::Pod::default();
        let patch = self.create_server_pod_patch(server, pod).await;
        let pp = PatchParams::apply("mcp-server");
        let client = self.get_client().await;
        Api::namespaced(client, &self.namespace)
            .patch(&server.name_pod(), &pp, &patch)
            .await
            .map_err(Error::ServerPodTemplateError)
    }
}
