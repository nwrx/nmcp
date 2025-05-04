use crate::MCPServer;
use crate::TestContext;
use anyhow::Result;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::{Api, DeleteParams};
use kube::CustomResourceExt;
use tokio::time::sleep;

impl TestContext {
    /// Deletes the MCPServer CRD from the Kubernetes cluster
    ///
    /// # Returns
    /// A Result that is Ok if the CRD was deleted successfully, or contains an error otherwise
    ///
    /// # Details
    /// This function deletes the MCPServer CRD from the Kubernetes cluster if it exists.
    /// If the CRD doesn't exist, it logs a warning but still returns Ok.
    ///
    /// # Example
    /// ```
    /// let context = TestContext::new();
    /// context.delete_crd_servers().await?;
    /// ```
    pub async fn delete_crd_servers(&self) -> Result<()> {
        // Get client
        let client = self.get_client().await?;
        let crds_api: Api<CustomResourceDefinition> = Api::all(client);
        let crd_name = MCPServer::crd_name();

        // Check if CRD exists
        match crds_api.get(crd_name).await {
            Ok(_) => {
                crds_api.delete(crd_name, &DeleteParams::default()).await?;
                sleep(std::time::Duration::from_millis(10)).await;
                Ok(())
            }
            Err(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
    use kube::{api::Api, CustomResourceExt};

    #[tokio::test]
    async fn test_delete_crd_servers() {
        let context = TestContext::new().unwrap();

        // --- Create the CRD first to ensure it exists
        let create_result = context.create_crd_servers().await;
        assert!(
            create_result.is_ok(),
            "Failed to create MCPServer CRD for deletion test"
        );

        // --- Delete the CRD
        let delete_result = context.delete_crd_servers().await;
        assert!(
            delete_result.is_ok(),
            "Failed to delete MCPServer CRD: {:?}",
            delete_result.err()
        );

        // --- Verify CRD no longer exists
        let client = context.get_client().await.unwrap();
        let crds_api: Api<CustomResourceDefinition> = Api::all(client);
        let crd_name = MCPServer::crd_name();
        let crd = crds_api.get(crd_name).await;
        assert!(crd.is_err(), "CRD was not deleted successfully");
    }
}
