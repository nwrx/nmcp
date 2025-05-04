use super::Controller;
use crate::{Error, MCPServer, Result};
use kube::api::Api;
use kube::runtime::finalizer::{Error as FinalizerError, Event};
use kube::runtime::{controller::Action, finalizer};
use std::{result::Result as StdResult, sync::Arc, time::Duration};

/// The finalizer name for MCPServer resources.
pub const MCP_SERVER_FINALIZER: &str = "mcpserver.unmcp.dev/finalizer";

impl Controller {
    /// Reconcile the MCPServer resource by checking its status and updating it accordingly.
    ///
    /// This function handles the reconciliation process using finalizers to ensure
    /// that the cleanup process is completed before the resource is deleted.
    ///
    /// # Arguments
    /// * `server` - A reference to the MCPServer object.
    ///
    /// # Returns
    /// * `Result<Action>` - The next action to take for this reconciliation.
    ///
    /// # Errors
    /// * Returns an error if there's an issue during reconciliation.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// let action = controller.server_reconciler(&server).await?;
    /// ```
    pub async fn reconcile_server(&self, server: &MCPServer) -> Result<Action> {
        let client = self.get_client().await;
        let api = Api::<MCPServer>::namespaced(client.clone(), &self.namespace);
        let obj = Arc::new(server.clone());

        // Handle the reconciliation process using finalizers to ensure
        // that the cleanup process is completed before the resource is deleted.
        let result: StdResult<Action, FinalizerError<Error>> =
            finalizer(&api, MCP_SERVER_FINALIZER, obj, {
                let server = server.clone();
                let controller = self.clone();
                move |event| async move {
                    match event {
                        Event::Cleanup { .. } => {
                            controller.stop_server(&server).await?;
                            Ok(Action::await_change())
                        }
                        Event::Apply { .. } => {
                            if controller.should_server_be_up(&server).await? {
                                controller.start_server(&server).await?;
                            } else {
                                controller.stop_server(&server).await?;
                            }
                            Ok(Action::requeue(Duration::from_secs(60)))
                        }
                    }
                }
            })
            .await;

        match result {
            Ok(action) => Ok(action),
            Err(error) => {
                eprintln!("Reconciliation error: {error}");
                match error {
                    FinalizerError::AddFinalizer(_) => Ok(Action::requeue(Duration::from_secs(5))),
                    FinalizerError::ApplyFailed(_) => Ok(Action::requeue(Duration::from_secs(5))),
                    FinalizerError::CleanupFailed(_) => Ok(Action::requeue(Duration::from_secs(5))),
                    FinalizerError::InvalidFinalizer => Ok(Action::requeue(Duration::from_secs(5))),
                    FinalizerError::RemoveFinalizer(_) => {
                        Ok(Action::requeue(Duration::from_secs(5)))
                    }
                    FinalizerError::UnnamedObject => Ok(Action::requeue(Duration::from_secs(5))),
                }
            }
        }
    }
}
