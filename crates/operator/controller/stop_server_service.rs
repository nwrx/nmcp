use super::Controller;
use crate::{Error, MCPServer, Result};
use k8s_openapi::api::core::v1;
use kube::Api;

impl Controller {
    /// Delete the v1::Service resource for the `MCPServer`.
    ///
    /// This function deletes the Service associated with the MCPServer.
    /// If the Service doesn't exist (404 error), it will return Ok.
    ///
    /// # Arguments
    /// * `server` - A reference to the MCPServer object.
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the Service was deleted successfully or didn't exist.
    ///
    /// # Errors
    /// * `Error::ServerServiceDeleteError` - If there is an error deleting the Service.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// controller.stop_server_service(&server).await?;
    /// ```
    pub async fn stop_server_service(&self, server: &MCPServer) -> Result<()> {
        let client = self.get_client().await;
        let api: Api<v1::Service> = Api::namespaced(client, &self.namespace);
        match api
            .delete(&server.name_service(), &Default::default())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                if let kube::Error::Api(err) = &e {
                    if err.code == 404 {
                        return Ok(());
                    };
                }
                Err(Error::ServerServiceDeleteError(e))
            }
        }
    }
}
