use super::Controller;
use super::ResourceManager;
use super::MCP_SERVER_FINALIZER;
use crate::Error;
use crate::MCPPool;
use crate::{ErrorInner, IntoResource, MCPServer, Result};
use crate::{MCPServerConditionType as Condition, MCPServerPhase as Phase};
use chrono::Utc;
use futures::StreamExt;
use k8s_openapi::api::core::v1;
use kube::runtime::controller::Action;
use kube::runtime::finalizer;
use kube::runtime::finalizer::Event;
use kube::runtime::{watcher::Config, Controller as RuntimeController};
use kube::{Api, ResourceExt};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PodStatus {
    Running,
    Pending,
    Succeeded,
    Failed,
    Unknown,
    NotFound,
}

#[derive(Debug)]
struct ReconcileReportError(Error);

impl std::fmt::Display for ReconcileReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ReconcileReportError {}

impl From<ReconcileReportError> for Error {
    fn from(e: ReconcileReportError) -> Self {
        e.0
    }
}

impl Controller {
    /// Get the pod status for the given `MCPServer`.
    pub async fn get_server_pod_status(&self, server: &MCPServer) -> Result<PodStatus> {
        let pod = <MCPServer as IntoResource<v1::Pod>>::get_resource(server, &self.client).await;
        match pod {
            Ok(pod) => {
                let phase = pod.status.unwrap_or_default().phase.unwrap_or_default();
                match phase.as_str() {
                    "Running" => Ok(PodStatus::Running),
                    "Pending" => Ok(PodStatus::Pending),
                    "Succeeded" => Ok(PodStatus::Succeeded),
                    "Failed" => Ok(PodStatus::Failed),
                    _ => Ok(PodStatus::Unknown),
                }
            }
            Err(error) => match error.source() {
                ErrorInner::KubeError(kube::Error::Api(error)) if error.code == 404 => {
                    Ok(PodStatus::NotFound)
                }
                _ => Err(error),
            },
        }
    }

    /// Start the server pod and service for the given MCPServer.
    #[tracing::instrument(name = "EnsureUp", skip_all)]
    pub async fn ensure_server_is_up(&self, server: &MCPServer) -> Result<()> {
        match self.get_server_pod_status(server).await? {
            PodStatus::Running => {}
            PodStatus::NotFound => {
                let _ = <MCPServer as IntoResource<v1::Pod>>::patch_resource(server, &self.client)
                    .await?;
            }
            PodStatus::Pending => {
                let condition = Condition::PodPending;
                let _ = server.set_server_status(&self.client, condition).await?;
            }
            PodStatus::Succeeded => {
                let condition = Condition::PodTerminated;
                let _ = server.set_server_status(&self.client, condition).await?;
            }
            PodStatus::Failed => {
                let condition = Condition::PodFailed("Pod is in an error state".to_string());
                let _ = server.set_server_status(&self.client, condition).await?;
            }
            PodStatus::Unknown => {
                let condition = Condition::PodFailed("Pod is in an unknown state".to_string());
                let _ = server.set_server_status(&self.client, condition).await?;
            }
        }
        // self.start_server_service(server).await?;
        server
            .set_server_status(&self.client, Condition::Running)
            .await
            .map(|_| ())
    }

    /// Stop the server pod and service for the given MCPServer.
    #[tracing::instrument(name = "EnsureDown", skip_all, err)]
    pub async fn ensure_server_is_down(&self, server: &MCPServer) -> Result<()> {
        let phase = server.get_status(&self.client).await?.phase;

        // --- Track the pod status and delete it if necessary.
        match self.get_server_pod_status(server).await? {
            PodStatus::NotFound => {
                if phase != Phase::Idle {
                    let condition = Condition::PodTerminated;
                    let _ = server.set_server_status(&self.client, condition).await?;
                }
            }
            _ => {
                <MCPServer as IntoResource<v1::Pod>>::delete_resource(server, &self.client).await?;
                let condition = Condition::PodTerminating;
                let _ = server.set_server_status(&self.client, condition).await?;
            }
        }

        // --- Finally, set the server status to "Idle" and clean up conditions.
        if phase != Phase::Idle {
            let condition = Condition::Idle;
            let _ = server.set_server_status(&self.client, condition).await?;
            let _ = server.cleanup_server_conditions(&self.client).await?;
        }
        Ok(())
    }

    /// Determine if the server should be started based on its status and pool limits.
    #[tracing::instrument(name = "CanServerdBeUp", skip_all)]
    pub async fn can_server_be_up(&self, server: &MCPServer) -> Result<bool> {
        let pool = MCPPool::get_by_name(&self.client, &server.spec.pool).await?;
        let pool_status = pool.status.unwrap_or_default();

        // --- If active_servers_count >= max_servers_active, server should not be up
        if pool_status.active_servers_count >= pool.spec.max_servers_active {
            tracing::info!(
                "Pool active servers limit reached: {} >= {}",
                pool_status.active_servers_count,
                pool.spec.max_servers_active
            );
            return Ok(false);
        }

        // --- If the server is not "Idle", it should be started
        Ok(true)
    }

    /// Determine if the server should be shutdown based on its status and idle timeout.
    #[tracing::instrument(name = "ShouldServerBeDown", skip_all)]
    pub async fn should_server_be_down(&self, server: &MCPServer) -> Result<bool> {
        let status = server.get_status(&self.client).await?;
        let pool = MCPPool::get_by_name(&self.client, &server.spec.pool).await?;

        // --- Check idle timeout
        if let Some(last_request) = &status.last_request_at {
            let tiemout = match server.spec.idle_timeout {
                0 => pool.spec.default_idle_timeout,
                _ => server.spec.idle_timeout,
            };

            // If elapsed time is greater than the idle timeout, server should not be up
            let now = Utc::now();
            let elapsed = now.signed_duration_since(*last_request).to_std().unwrap();
            let elapsed_secs = elapsed.as_secs() as i64;
            if elapsed_secs > tiemout as i64 {
                return Ok(true);
            }
        }

        // --- If all checks pass, the server should be up.
        Ok(false)
    }

    /// Reconcile the MCPServer resource by checking its status and updating it accordingly.
    #[tracing::instrument(name = "Reconcile", skip_all, fields(server = %server.name_any()))]
    async fn reconcile(
        &self,
        server: Arc<MCPServer>,
    ) -> core::result::Result<Action, finalizer::Error<ReconcileReportError>> {
        let api = Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace());

        // --- Handle the reconciliation process using finalizers to ensure
        // --- that the cleanup process is completed before the resource is deleted.
        finalizer(&api, MCP_SERVER_FINALIZER, server, {
            let controller = self.clone();
            move |event| async move {
                match event {
                    Event::Cleanup(server) => {
                        self.ensure_server_is_down(&server)
                            .await
                            .expect("Failed to ensure server is down");
                        Ok(Action::requeue(Duration::from_secs(5)))
                    }
                    Event::Apply(server) => async {
                        let phase = server.status.clone().unwrap_or_default().phase;
                        match phase {
                            Phase::Idle => self.ensure_server_is_down(&server).await?,
                            Phase::Failed => self.ensure_server_is_down(&server).await?,
                            Phase::Stopping => self.ensure_server_is_down(&server).await?,
                            Phase::Requested => {
                                tracing::info!(
                                    "Server is in requested phase, checking conditions for: {}",
                                    server.name_any()
                                );
                                if controller.can_server_be_up(&server).await? {
                                    self.ensure_server_is_up(&server).await?
                                } else {
                                    self.ensure_server_is_down(&server).await?
                                }
                            }
                            Phase::Starting => {
                                tracing::info!("Starting server: {}", server.name_any());
                                if controller.can_server_be_up(&server).await? {
                                    self.ensure_server_is_up(&server).await?
                                } else if controller.should_server_be_down(&server).await? {
                                    self.ensure_server_is_down(&server).await?
                                }
                            }
                            Phase::Running => {
                                tracing::info!("Server is running: {}", server.name_any());
                                if controller.should_server_be_down(&server).await? {
                                    self.ensure_server_is_down(&server).await?
                                } else {
                                    self.ensure_server_is_up(&server).await?
                                }
                            }
                        }
                        Result::Ok(Action::requeue(Duration::from_secs(5)))
                    }
                    // The `kube::runtime::finalizer` expects it's reconcile closure to return an error that
                    // implements `std::error::Error`, however, since we are using `error_stack::Report` and
                    // since that type does not implement `std::error::Error`, we need to temporarily wrap it
                    // in a custom error type that does implement `std::error::Error`.
                    .await
                    .map_err(ReconcileReportError),
                }
            }
        })
        .await
    }

    /// Handle an error during the reconciliation process.
    #[tracing::instrument(name = "ErrorPolicy", skip_all)]
    fn error_policy(
        &self,
        _server: &MCPServer,
        error: &finalizer::Error<ReconcileReportError>,
    ) -> Result<Action> {
        match error {
            finalizer::Error::ApplyFailed(e) => {
                let _ = e.0.clone().trace();
                Ok(Action::requeue(Duration::from_secs(5)))
            }
            _ => {
                tracing::error!("Unhandled error during MCPServer reconciliation: {}", error);
                // Requeue the action to retry later
                Ok(Action::requeue(Duration::from_secs(5)))
            }
        }
    }

    /// Start the operator for managing MCPServer resources.
    #[tracing::instrument(name = "Operator", skip_all, err)]
    pub async fn start_server_operator(&self) -> Result<()> {
        let ns = self.get_namespace();
        let wc = Config::default();

        // --- Create API clients for MCPServer, Pod, and Service.
        let api = Api::<MCPServer>::namespaced(self.get_client(), &ns);
        let api_pod = Api::<v1::Pod>::namespaced(self.get_client(), &ns);
        let api_services = Api::<v1::Service>::namespaced(self.get_client(), &ns);

        // --- Start the controller for MCPServer resources.
        tracing::info!("Starting MCPServer operator in namespace '{}'", ns);
        let stream = RuntimeController::new(api, wc.clone())
            .owns(api_pod, Default::default())
            .owns(api_services, Default::default())
            .run(
                |server, controller| async move {
                    tracing::debug!("Reconcile MCPServer: {}", server.name_any());
                    controller.reconcile(server).await
                },
                |server, error, controller| controller.error_policy(&server, error).unwrap(),
                Arc::new(self.clone()),
            );

        // --- Loop to handle the reconciliation stream.
        stream
            .for_each(|result| {
                match result {
                    Ok(action) => tracing::debug!("Reconciled MCPServer action: {:?}", action),
                    Err(error) => {
                        let _ = Error::from(error).trace();
                    }
                }
                futures::future::ready(())
            })
            .await;

        Ok(())
    }
}
