use anyhow::Result;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{api::{Api, DeleteParams}, CustomResourceExt};
use tokio::time::sleep;
use tracing::{info, warn};

use crate::MCPPool;
use super::TestContext;

#[cfg(test)]
impl TestContext {
    /// Deletes the MCPPool CRD from the Kubernetes cluster
    ///
    /// # Returns
    /// A Result that is Ok if the CRD was deleted successfully, or contains an error otherwise
    ///
    /// # Details
    /// This function deletes the MCPPool CRD from the Kubernetes cluster if it exists.
    /// If the CRD doesn't exist, it logs a warning but still returns Ok.
    ///
    /// # Example
    /// ```
    /// let context = TestContext::new().await?;
    /// context.delete_crd_pools().await?;
    /// ```
    pub async fn delete_crd_pools(&self) -> Result<()> {
        info!("Deleting MCPPool CRD if it exists");
        
        // --- Get the CRD API from the client.
        let client = self.get_client().await?;
        let crds_api: Api<CustomResourceDefinition> = Api::all(client);
        let crd_name = MCPPool::crd_name();
        
        // Check if CRD exists
        match crds_api.get(crd_name).await {
            Ok(_) => {
                info!("Deleting MCPPool CRD '{}'", crd_name);
                crds_api.delete(crd_name, &DeleteParams::default()).await?;
                info!("MCPPool CRD deleted successfully");

                // --- Sleep for a short duration to allow the deletion to propagate.
                sleep(std::time::Duration::from_millis(10)).await;
                Ok(())
            },
            Err(_) => {
                warn!("MCPPool CRD '{}' does not exist in the cluster", crd_name);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
    use kube::api::Api;

    #[tokio::test]
    async fn test_delete_crd_pools() {
        let context = TestContext::new().unwrap();
        
        // --- Create the CRD first to ensure it exists
        let create_result = context.create_crd_pools().await;
        assert!(create_result.is_ok(), "Failed to create MCPPool CRD for deletion test");
        
        // --- Delete the CRD
        let delete_result = context.delete_crd_pools().await;
        assert!(delete_result.is_ok(), "Failed to delete MCPPool CRD: {:?}", delete_result.err());
        
        // --- Verify CRD no longer exists
        let client = context.get_client().await.unwrap();
        let crds_api: Api<CustomResourceDefinition> = Api::all(client);
        let crd_name = MCPPool::crd_name();
        let crd = crds_api.get(crd_name).await;
        assert!(crd.is_err(), "CRD was not deleted successfully");
    }
}
