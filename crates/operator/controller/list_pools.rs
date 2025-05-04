use super::Controller;
use crate::{Error, MCPPool, Result};
use kube::{api::ListParams, Api};

impl Controller {
    /// List all MCPPool objects in the Kubernetes cluster.
    ///
    /// This function retrieves all MCPPool objects in the specified namespace.
    /// It uses the `list` method of the MCPPool object to get the list of pools.
    ///
    /// # Returns
    /// * `Result<Vec<MCPPool>>` - A vector of MCPPool objects.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// let pools = controller.list_pools().await?;
    /// ```
    pub async fn list_pools(&self) -> Result<Vec<MCPPool>> {
        let client = self.get_client().await;
        let pools = Api::<MCPPool>::namespaced(client, &self.namespace)
            .list(&ListParams::default())
            .await
            .map_err(Error::KubeError)?;
        Ok(pools.items)
    }
}
