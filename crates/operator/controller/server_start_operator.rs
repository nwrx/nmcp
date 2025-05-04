use crate::{Controller, MCPServer, Result};
use futures::StreamExt;
use k8s_openapi::api::core::v1;
use kube::runtime::controller::Action;
use kube::runtime::{watcher::Config, Controller as RuntimeController};
use kube::Api;
use std::sync::Arc;
use std::time::Duration;

impl Controller {
    /// Start the operator for managing MCPServer resources.
    ///
    /// This function initializes the operator, sets up the necessary API clients,
    /// and starts the reconciliation loop for MCPServer resources.
    ///
    /// # Returns
    /// * `Result<()>` - A result indicating success or failure.
    ///
    /// # Errors
    /// * `Error::ServerPodTemplateError` - If there is an error creating the Pod.
    /// * `Error::ServerServiceTemplateError` - If there is an error creating the Service.
    /// * `Error::ServerReconcileError` - If there is an error during reconciliation.
    pub async fn start_operator(&self) -> Result<()> {
        let client = self.get_client().await;
        let ns = self.namespace.clone();
        let wc = Config::default();

        // --- Create API clients for MCPServer, Pod, and Service.
        let api = Api::<MCPServer>::namespaced(client.clone(), &ns);
        let api_pod = Api::<v1::Pod>::namespaced(client.clone(), &ns);
        let api_services = Api::<v1::Service>::namespaced(client.clone(), &ns);

        // Start the controller
        RuntimeController::new(api, wc.clone())
            .owns(api_pod, wc.clone())
            .owns(api_services, wc.clone())
            .run(
                |server, controller| async move { controller.server_reconciler(&server).await },
                |_, _, _context| Action::requeue(Duration::from_secs(5)),
                Arc::new(self.clone()),
            )
            .for_each(|result| async move {
                match result {
                    Ok(action) => print!("Reconciled MCPServer: {action:?}"),
                    Err(error) => eprintln!("Reconciliation error: {error}"),
                }
            })
            .await;

        Ok(())
    }
}
