use kube::Resource;
use tracing::{info, warn};
use kube::api::{Api, ListParams};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;

use crate::utils::{Error, Result};
use crate::pool::pool_controller::MCPPoolController;
use crate::pool::pool_crd::MCPPool;

impl MCPPoolController {
    /// Asserts that the MCPPool CRD exists in the Kubernetes cluster
    ///
    /// # Returns
    /// A Result that is Ok if the CRD exists, or an Error if it doesn't exist or if there was an error checking
    ///
    /// # Details
    /// This function checks if the MCPPool CRD is registered in the Kubernetes cluster.
    /// If the CRD doesn't exist, it returns an error, which can be used to determine
    /// whether the controller should start or not.
    ///
    pub async fn assert_crd_exists(&self) -> Result<()> {
        info!("Checking if MCPPool CRD exists");
        
        // Get client from context
        let client = self.context.read().await.client.clone();
        
        // Get the CRD name based on MCPPool resource
        let crd_name = format!(
            "{}.{}",
            MCPPool::plural(&()).to_lowercase(),
            MCPPool::group(&()).to_lowercase()
        );
        
        // Create API for CRDs
        let crds_api: Api<CustomResourceDefinition> = Api::all(client);
        
        // Try to find our CRD
        let list_params = ListParams::default()
            .fields(&format!("metadata.name={}", crd_name));
        
        match crds_api.list(&list_params).await {
            Ok(list) => {
                if list.items.is_empty() {
                    warn!("MCPPool CRD not found in the cluster");
                    Err(Error::ReconciliationError(format!(
                        "MCPPool CRD '{}' not found in the cluster. Please install the CRD before starting the controller",
                        crd_name
                    )))
                } else {
                    info!("MCPPool CRD exists in the cluster");
                    Ok(())
                }
            },
            Err(err) => {
                warn!("Error checking for MCPPool CRD: {}", err);
                Err(Error::KubeError(err))
            }
        }
    }
}