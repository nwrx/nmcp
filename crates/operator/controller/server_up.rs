use super::Controller;
use crate::{MCPServer, Result};

impl Controller {
    /// Start the `MCPServer` by creating its associated resources.
    ///
    /// This function creates both the Pod and Service resources for the MCPServer.
    ///
    /// # Arguments
    /// * `server` - A reference to the MCPServer object.
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the resources were created successfully.
    ///
    /// # Errors
    /// * Returns an error if there's an issue creating either resource.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// controller.server_up(&server).await?;
    /// ```
    pub async fn server_up(&self, server: &MCPServer) -> Result<()> {
        self.server_start_pod(server).await?;
        self.server_start_service(server).await?;
        Ok(())
    }
}
