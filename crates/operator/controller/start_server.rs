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
    /// controller.start_server(&server).await?;
    /// ```
    pub async fn start_server(&self, server: &MCPServer) -> Result<()> {
        self.start_server_pod(server).await?;
        self.start_server_service(server).await?;
        Ok(())
    }
}
