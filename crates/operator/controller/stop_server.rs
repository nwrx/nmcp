use super::Controller;
use crate::{MCPServer, Result};

impl Controller {
    /// Stop the `MCPServer` by deleting its associated resources.
    ///
    /// This function deletes both the Pod and Service resources for the MCPServer.
    ///
    /// # Arguments
    /// * `server` - A reference to the MCPServer object.
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the resources were deleted successfully.
    ///
    /// # Errors
    /// * Returns an error if there's an issue deleting either resource.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// controller.stop_server(&server).await?;
    /// ```
    pub async fn stop_server(&self, server: &MCPServer) -> Result<()> {
        self.stop_server_pod(server).await?;
        self.stop_server_service(server).await?;
        Ok(())
    }
}
