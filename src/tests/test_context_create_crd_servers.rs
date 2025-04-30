use anyhow::Result;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{api::{Api, PostParams}, CustomResourceExt};
use tracing::{info, warn};

use crate::MCPServer;
use super::TestContext;

#[cfg(test)]
impl TestContext {
    /// Creates the MCPServer CRD in the Kubernetes cluster
    ///
    /// # Returns
    /// A Result that is Ok if the CRD was created successfully, or contains an error otherwise
    ///
    /// # Details
    /// This function creates the MCPServer CRD in the Kubernetes cluster if it doesn't already exist.
    /// If the CRD already exists, it logs a warning but still returns Ok.
    ///
    /// # Example
    /// ```
    /// let context = TestContext::new().await?;
    /// context.create_crd_servers().await?;
    /// ```
    pub async fn create_crd_servers(&self) -> Result<()> {
        info!("Creating MCPServer CRD if it doesn't exist");
        
        // Get client
        let client = self.get_client().await?;
        let crd: CustomResourceDefinition = MCPServer::crd();
        let crds_api: Api<CustomResourceDefinition> = Api::all(client);
        let crd_name = MCPServer::crd_name();
        
        // Check if CRD already exists
        match crds_api.get(crd_name).await {
            Ok(_) => {
                warn!("MCPServer CRD '{}' already exists in the cluster", crd_name);
                Ok(())
            },
            Err(_) => {
                // Create the CRD
                info!("Creating MCPServer CRD '{}'", crd_name);
                crds_api.create(&PostParams::default(), &crd).await?;
                info!("MCPServer CRD created successfully");
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
    async fn test_create_crd_servers() {
        
        // --- Create the CRD
        let context = TestContext::new().unwrap();
        let result = context.create_crd_servers().await;
        assert!(result.is_ok(), "Failed to create MCPServer CRD: {:?}", result.err());
        
        // --- Verify CRD exists
        let client = context.get_client().await.unwrap();
        let crds_api: Api<CustomResourceDefinition> = Api::all(client);
        let crd_name = MCPServer::crd_name();
        let crd = crds_api.get(crd_name).await;
        assert!(crd.is_ok(), "CRD was not created successfully");
        
        // Clean up - delete CRD after test
        let _ = context.delete_crd_servers().await;
    }
}
