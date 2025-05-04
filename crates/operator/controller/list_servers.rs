use super::Controller;
use crate::{Error, MCPServer, Result};
use kube::{api::ListParams, Api};

impl Controller {
    /// List all MCPServer objects in the Kubernetes cluster.
    ///
    /// This function retrieves all MCPServer objects in the specified namespace.
    /// It uses the `list` method of the MCPServer object to get the list of servers.
    ///
    /// # Returns
    /// * `Result<Vec<MCPServer>>` - A vector of MCPServer objects.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// let servers = controller.list_servers().await?;
    /// ```
    pub async fn list_servers(&self) -> Result<Vec<MCPServer>> {
        let client = self.get_client().await;
        let servers = Api::<MCPServer>::namespaced(client, &self.namespace)
            .list(&ListParams::default())
            .await
            .map_err(Error::KubeError)?;
        Ok(servers.items)
    }
}
