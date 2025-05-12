use super::{Controller, MCP_SERVER_FINALIZER};
use crate::{Error, MCPServer, MCPServerPhase, Result};
use chrono::Utc;
use futures::StreamExt;
use k8s_openapi::api::core::v1;
use kube::runtime::controller::Action;
use kube::runtime::finalizer;
use kube::runtime::finalizer::{Error as FinalizerError, Event};
use kube::runtime::{watcher::Config, Controller as RuntimeController};
use kube::{Api, ResourceExt};
use std::result::Result as StdResult;
use std::sync::Arc;
use std::time::Duration;
use tracing::error;

impl Controller {
    /// Determine if a server should be up based on pool limits and idle timeout
    pub async fn should_server_be_running(&self, server: &MCPServer) -> Result<bool> {
        let pool = self.get_server_pool(server).await?;

        // --- Check pool limits
        if let Some(pool_status) = &pool.status {
            // If managed_servers_count > max_servers_limit, server should not be up
            if pool_status.managed_servers_count > pool.spec.max_servers_limit {
                return Ok(false);
            }

            // If active_servers_count >= max_servers_active, server should not be up
            if pool_status.active_servers_count >= pool.spec.max_servers_active {
                return Ok(false);
            }
        }

        // --- Check idle timeout
        if let Some(server_status) = &server.status {
            if server_status.phase == MCPServerPhase::Running {
                if let Some(last_request) = &server_status.last_request_at {
                    // Get the relevant idle timeout (server's value or pool's default)
                    let idle_timeout = if server.spec.idle_timeout > 0 {
                        server.spec.idle_timeout
                    } else {
                        pool.spec.default_idle_timeout
                    };

                    // If elapsed time is greater than the idle timeout, server should not be up
                    let now = Utc::now();
                    let elapsed = now.signed_duration_since(*last_request);
                    let elapsed_secs = elapsed.num_seconds();
                    if elapsed_secs > idle_timeout as i64 {
                        return Ok(false);
                    }
                }
            }
        }

        // --- Check if the server is in the "Requested" phase
        let should_be_running = matches!(
            server.status.clone().unwrap_or_default().phase,
            MCPServerPhase::Pending
                | MCPServerPhase::Requested
                | MCPServerPhase::Running
                | MCPServerPhase::Starting
        );
        Ok(should_be_running)
    }

    /// Reconcile the MCPServer resource by checking its status and updating it accordingly.
    async fn reconcile_server(&self, server: &MCPServer) -> Result<Action> {
        let api = Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace());
        let obj = Arc::new(server.clone());

        // --- Handle the reconciliation process using finalizers to ensure
        // --- that the cleanup process is completed before the resource is deleted.
        let result: StdResult<Action, FinalizerError<Error>> =
            finalizer(&api, MCP_SERVER_FINALIZER, obj, {
                let server = server.clone();
                let controller = self.clone();
                move |event| async move {
                    match event {
                        Event::Cleanup { .. } => {
                            self.delete_server_pod(&server).await?;
                            self.delete_server_service(&server).await?;
                            Ok(Action::await_change())
                        }
                        Event::Apply { .. } => {
                            // --- Delete the server if unused or threshold reached.
                            if !controller.should_server_be_running(&server).await? {
                                self.delete_server_pod(&server).await?;
                                self.delete_server_service(&server).await?;
                            }
                            // --- Set the status to "Requested" if not already set.
                            else {
                                self.start_server_pod(&server).await?;
                                self.start_server_service(&server).await?;
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
                error!("[{}] {}", server.name_any(), error);
                Ok(Action::requeue(Duration::from_secs(5)))
            }
        }
    }

    /// Start the operator for managing MCPServer resources.
    pub async fn start_server_operator(&self) -> Result<()> {
        let ns = self.get_namespace();
        let wc = Config::default();

        // --- Create API clients for MCPServer, Pod, and Service.
        let api = Api::<MCPServer>::namespaced(self.get_client(), &ns);
        let api_pod = Api::<v1::Pod>::namespaced(self.get_client(), &ns);
        let api_services = Api::<v1::Service>::namespaced(self.get_client(), &ns);

        // --- Start the controller for MCPServer resources.
        RuntimeController::new(api, wc.clone())
            .owns(api_pod, wc.clone())
            .owns(api_services, wc.clone())
            .run(
                |server, controller| async move { controller.reconcile_server(&server).await },
                |_, _, _| Action::requeue(Duration::from_secs(5)),
                Arc::new(self.clone()),
            )
            .for_each(|result| async move {
                result.is_err().then(|| {
                    error!("Error in controller: {:?}", result);
                });
            })
            .await;

        Ok(())
    }
}
