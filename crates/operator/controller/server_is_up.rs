use super::Controller;
use crate::{MCPServer, Result};

impl Controller {
    /// Check if the server is currently running.
    ///
    /// This function checks if the Pod associated with the MCPServer is in the "Running" state.
    ///
    /// # Arguments
    /// * `server` - A reference to the MCPServer object.
    ///
    /// # Returns
    /// * `Result<bool>` - True if the server is running, false otherwise.
    ///
    /// # Errors
    /// * Returns an error if there's an issue retrieving the pod information.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// let is_running = controller.server_is_up(&server).await?;
    /// ```
    pub async fn server_is_up(&self, server: &MCPServer) -> Result<bool> {
        let pod = self.server_get_pod(server).await?;
        let status = pod.status.clone().unwrap_or_default();
        if let Some(phase) = status.phase {
            if phase == "Running" {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
