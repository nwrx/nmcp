use kube::CustomResourceExt;
use tracing::{info, warn};
use kube::api::Api;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;

use crate::utils::{Error, Result};
use super::{MCPServerController, MCPServer};

impl MCPServerController {
    /// Asserts that the MCPServer CRD exists in the Kubernetes cluster
    ///
    /// # Returns
    /// A Result that is Ok if the CRD exists, or an Error if it doesn't exist or if there was an error checking
    ///
    /// # Details
    /// This function checks if the MCPServer CRD is registered in the Kubernetes cluster.
    /// If the CRD doesn't exist, it returns an error, which can be used to determine
    /// whether the controller should start or not.
    ///
    pub async fn assert_crd_exists(&self) -> Result<()> {
        info!("Checking if MCPServer CRD exists");
        
        // --- Get CRD API from the client.
        let client = self.context.read().await.client.clone();
        let crds_api: Api<CustomResourceDefinition> = Api::all(client);
        let crd_name = MCPServer::crd_name();
        
        // --- Check if CRD already exists.
        match crds_api.get(crd_name).await {
            Ok(_) => {
                info!("MCPServer CRD exists in the cluster");
                Ok(())
            },
            Err(err) => {
                warn!("MCPServer CRD not found in the cluster: {}", err);
                Err(Error::ReconciliationError(format!(
                    "MCPServer CRD '{}' not found in the cluster. Please install the CRD before starting the controller",
                    crd_name
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::TestContext;
    use super::*;

    #[tokio::test]
    async fn test_assert_crd_exists_ok() {
        let context = TestContext::new().unwrap();
        let client = context.get_client().await.unwrap();
        let controller = MCPServerController::new(client, None);
        context.create_crd_servers().await.unwrap();
        let result = controller.assert_crd_exists().await;
        assert!(result.is_ok(), "Failed to assert MCPServer CRD exists: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_assert_crd_exists_err() {
        let context = TestContext::new().unwrap();
        let client = context.get_client().await.unwrap();
        let controller = MCPServerController::new(client, None);
        context.delete_crd_servers().await.unwrap();
        let result = controller.assert_crd_exists().await;
        assert!(result.is_err(), "Expected error when asserting MCPServer CRD exists, but got Ok");
    }
}
