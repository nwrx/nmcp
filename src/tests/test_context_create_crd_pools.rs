use anyhow::Result;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{api::{Api, PostParams}, CustomResourceExt};
use tracing::{info, warn};

use crate::MCPPool;
use super::TestContext;

#[cfg(test)]
impl TestContext {
    /// Creates the MCPPool CRD in the Kubernetes cluster
    ///
    /// # Returns
    /// A Result that is Ok if the CRD was created successfully, or contains an error otherwise
    ///
    /// # Details
    /// This function creates the MCPPool CRD in the Kubernetes cluster if it doesn't already exist.
    /// If the CRD already exists, it logs a warning but still returns Ok.
    ///
    /// # Example
    /// ```
    /// let context = TestContext::new().await?;
    /// context.create_crd_pools().await?;
    /// ```
    pub async fn create_crd_pools(&self) -> Result<()> {
        info!("Creating MCPPool CRD if it doesn't exist");
        
        // --- Get the CRD API from the client.
        let client = self.get_client().await?;
        let crd: CustomResourceDefinition = MCPPool::crd();
        let crds_api: Api<CustomResourceDefinition> = Api::all(client);
        let crd_name = MCPPool::crd_name();
        
        // Check if CRD already exists
        match crds_api.get(crd_name).await {
            Ok(_) => {
                warn!("MCPPool CRD '{}' already exists in the cluster", crd_name);
                Ok(())
            },
            Err(_) => {
                // Create the CRD
                info!("Creating MCPPool CRD '{}'", crd_name);
                crds_api.create(&PostParams::default(), &crd).await?;
                info!("MCPPool CRD created successfully");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kube::api::Api;
    use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;

    #[tokio::test]
    async fn test_create_crd_pools() {
        let context = TestContext::new().unwrap();
        
        // --- Create the CRD
        let result = context.create_crd_pools().await;
        assert!(result.is_ok(), "Failed to create MCPPool CRD: {:?}", result.err());
        
        // --- Verify CRD exists
        let client = context.get_client().await.unwrap();
        let crds_api: Api<CustomResourceDefinition> = Api::all(client);
        let crd_name = MCPPool::crd_name();
        let crd = crds_api.get(crd_name).await;
        assert!(crd.is_ok(), "CRD was not created successfully");
        
        // Clean up - delete CRD after test
        let _ = context.delete_crd_pools().await;
    }
}