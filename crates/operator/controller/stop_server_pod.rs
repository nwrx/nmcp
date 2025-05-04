use super::Controller;
use crate::{Error, MCPServer, Result};
use k8s_openapi::api::core::v1;
use kube::{Api, Error as KubeError};

impl Controller {
    /// Delete the `v1::Pod` resource for the `MCPServer`.
    ///
    /// This function deletes the Pod associated with the MCPServer.
    /// If the Pod doesn't exist (404 error), it will return Ok.
    ///
    /// # Arguments
    /// * `server` - A reference to the MCPServer object.
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the Pod was deleted successfully or didn't exist.
    ///
    /// # Errors
    /// * `Error::ServerPodDeleteError` - If there is an error deleting the Pod.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// controller.stop_server_pod(&server).await?;
    /// ```
    pub async fn stop_server_pod(&self, server: &MCPServer) -> Result<()> {
        let client = self.get_client().await;
        let api: Api<v1::Pod> = Api::namespaced(client, &self.namespace);
        match api.delete(&server.name_pod(), &Default::default()).await {
            Ok(_) => Ok(()),
            Err(error) => {
                if let KubeError::Api(error) = &error {
                    if error.code == 404 {
                        return Ok(());
                    };
                }
                Err(Error::ServerPodDeleteError(error))
            }
        }
    }
}
