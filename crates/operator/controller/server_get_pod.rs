use super::Controller;
use crate::{Error, MCPServer, Result};
use k8s_openapi::api::core::v1;
use kube::Api;

impl Controller {
    /// Get the server pod from the MCPServer object.
    ///
    /// This function retrieves the pod associated with the MCPServer object.
    /// It uses the `name_pod` method of the MCPServer object to get the pod name.
    ///
    /// # Arguments
    /// * `server` - A reference to the MCPServer object.
    ///
    /// # Returns
    /// * `Result<v1::Pod>` - The pod associated with the MCPServer object.
    ///
    /// # Errors
    /// * `Error::ServerPodNotFoundError` - If the pod is not found in the Kubernetes cluster.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// let pod = controller.server_get_pod(&server).await?;
    /// ```
    pub async fn server_get_pod(&self, server: &MCPServer) -> Result<v1::Pod> {
        let client = self.get_client().await;
        Api::<v1::Pod>::namespaced(client, &self.namespace)
            .get(&server.name_pod())
            .await
            .map_err(Error::ServerPodNotFoundError)
    }
}
