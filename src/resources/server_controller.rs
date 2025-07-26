use super::{
    IntoResource, MCPPool, MCPServer, MCPServerCondition as Condition, MCPServerPhase as Phase,
    MCPServerPodScheduledReason as PodScheduledReason, MCPServerRequestedReason as RequestReqson,
    MCPServerStaleReason as StaleReason, ResourceManager,
};
use crate::{Error, ErrorInner, Result, MCP_SERVER_CONTAINER_NAME};
use axum::http::StatusCode;
use chrono::Utc;
use futures::AsyncBufRead;
use k8s_openapi::api::core::v1;
use k8s_openapi::apimachinery::pkg::apis::meta;
use kube::api::LogParams;
use kube::api::ObjectMeta;
use kube::{Api, Client};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MCPServerPodStatus {
    Running,
    Pending,
    Succeeded,
    Failed {
        message: Option<String>,
        reason: Option<String>,
    },
    Unknown,
    NotFound,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MCPServerContainerStatus {
    Waiting(v1::ContainerStateWaiting),
    Running(v1::ContainerStateRunning),
    Terminated(v1::ContainerStateTerminated),
    ContainerNotFound,
    PodNotFound,
}

impl ResourceManager for MCPServer {
    fn new(name: &str, spec: Self::Spec) -> Self {
        Self {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                ..Default::default()
            },
            spec,
            status: Default::default(),
        }
    }
}

impl MCPServer {
    /***********************************************************************/
    /* Phase & Conditions                                                  */
    /***********************************************************************/

    /// Push or update a condition in the `MCPServer` status.
    pub async fn push_condition(&self, client: &Client, condition: Condition) -> Result<()> {
        let mut status = self.get_status(client).await?;
        let mut condition: meta::v1::Condition = condition.into();

        // --- Check if the condition already exists, if so, replace it,
        // --- update the observed generation and push the new condition.
        status.conditions.retain(|c| c.type_ != condition.type_);
        condition.observed_generation = self.metadata.generation;
        status.conditions.push(condition);

        // --- Apply the updated status to the resource.
        let _ = self.patch_status(client, status).await?;
        Ok(())
    }

    /// Clear all conditions from the `MCPServer` status.
    pub async fn clear_conditions(&self, client: &Client) -> Result<()> {
        let mut status = self.get_status(client).await?;
        if !status.conditions.is_empty() {
            status.conditions.clear();
            let _ = self.patch_status(client, status).await?;
        }
        Ok(())
    }

    /// Set the phase of the `MCPServer` to the given `Phase`.
    pub async fn set_phase(&self, client: &Client, phase: Phase) -> Result<()> {
        let mut status = self.get_status(client).await?;
        if status.phase != phase {
            status.phase = phase;
            let _ = self.patch_status(client, status).await?;
        }
        Ok(())
    }

    /***********************************************************************/
    /* Notifications                                                       */
    /***********************************************************************/

    /// Register that an `MCPServer` resource has been requested.
    pub async fn notify_request(&self, client: &Client) -> Result<()> {
        let mut status = self.get_status(client).await?;
        status.last_request_at = Some(Utc::now());
        status.total_requests += 1;
        let _ = self.patch_status(client, status).await?;
        Ok(())
    }

    /// Register that an active connection has been established.
    pub async fn notify_connect(&self, client: &Client) -> Result<()> {
        let mut status = self.get_status(client).await?;
        status.current_connections += 1;
        let _ = self.patch_status(client, status).await?;
        Ok(())
    }

    /// Register that an active connection has been closed.
    pub async fn notify_disconnect(&self, client: &Client) -> Result<()> {
        let mut status = self.get_status(client).await?;
        status.current_connections -= 1;
        let _ = self.patch_status(client, status).await?;
        Ok(())
    }

    /// Register the datetime when the server was started.
    pub async fn notify_started(&self, client: &Client) -> Result<()> {
        let mut status = self.get_status(client).await?;
        status.started_at = Some(Utc::now());
        let _ = self.patch_status(client, status).await?;
        Ok(())
    }

    /// Register the datetime when the server was requested to start.
    pub async fn notify_requested(&self, client: &Client) -> Result<()> {
        let mut status = self.get_status(client).await?;
        status.requested_at = Some(Utc::now());
        let _ = self.patch_status(client, status).await?;
        Ok(())
    }

    /***********************************************************************/
    /* Pod                                                                 */
    /***********************************************************************/

    /// Get the pod status for the given `MCPServer`.
    pub async fn get_pod_status(&self, client: &Client) -> Result<MCPServerPodStatus> {
        let pod = <Self as IntoResource<v1::Pod>>::get_resource(self, client).await;
        match pod {
            Ok(pod) => {
                let status = pod.status.unwrap_or_default();
                let phase = status.phase.unwrap_or_default();
                match phase.as_str() {
                    "Running" => Ok(MCPServerPodStatus::Running),
                    "Pending" => Ok(MCPServerPodStatus::Pending),
                    "Succeeded" => Ok(MCPServerPodStatus::Succeeded),
                    "Failed" => {
                        let message = status.message.clone();
                        let reason = status.reason.clone();
                        Ok(MCPServerPodStatus::Failed { message, reason })
                    }
                    _ => Ok(MCPServerPodStatus::Unknown),
                }
            }
            Err(error) => match error.source() {
                ErrorInner::KubeError(kube::Error::Api(error)) if error.code == 404 => {
                    Ok(MCPServerPodStatus::NotFound)
                }
                _ => Err(error),
            },
        }
    }

    /// Get the container status for the given `MCPServer`.
    pub async fn get_pod_container_status(
        &self,
        client: &Client,
    ) -> Result<MCPServerContainerStatus> {
        let pod = <Self as IntoResource<v1::Pod>>::get_resource(self, client).await;
        match pod {
            Ok(pod) => {
                if let Some(status) = pod.status {
                    if let Some(container_statuses) = status.container_statuses {
                        for container in container_statuses {
                            if container.name == MCP_SERVER_CONTAINER_NAME {
                                if let Some(state) = container.state {
                                    if let Some(waiting) = state.waiting {
                                        return Ok(MCPServerContainerStatus::Waiting(waiting));
                                    }
                                    if let Some(running) = state.running {
                                        return Ok(MCPServerContainerStatus::Running(running));
                                    }
                                    if let Some(terminated) = state.terminated {
                                        return Ok(MCPServerContainerStatus::Terminated(
                                            terminated,
                                        ));
                                    }
                                }
                            }
                        }
                        Ok(MCPServerContainerStatus::ContainerNotFound)
                    } else {
                        Ok(MCPServerContainerStatus::ContainerNotFound)
                    }
                } else {
                    Ok(MCPServerContainerStatus::PodNotFound)
                }
            }
            Err(error) => match error.source() {
                ErrorInner::KubeError(kube::Error::Api(error)) if error.code == 404 => {
                    Ok(MCPServerContainerStatus::PodNotFound)
                }
                _ => Err(error),
            },
        }
    }

    /// Ensure that the `Pod` for the `MCPServer` is scheduled.
    pub async fn ensure_pod_is_scheduled(&self, client: &Client) -> Result<()> {
        match self.get_pod_status(client).await? {
            MCPServerPodStatus::Running | MCPServerPodStatus::Pending => {
                // Pod is Running or Pending, we can wait for it to be scheduled.
            }
            MCPServerPodStatus::NotFound => {
                tracing::info!("Pod not found, creating it for server");
                let state = PodScheduledReason::Scheduled;
                let condition = Condition::PodScheduled(state);
                self.push_condition(client, condition).await?;
                let _ = <Self as IntoResource<v1::Pod>>::patch_resource(self, client).await?;
            }
            MCPServerPodStatus::Failed { message, .. } => {
                let error = Error::generic(message.unwrap_or("Pod failed to start".to_string()))
                    .with_name("E_POD_FAILED")
                    .with_status(StatusCode::SERVICE_UNAVAILABLE);
                return Err(error);
            }
            MCPServerPodStatus::Succeeded => {
                let error = Error::generic("Pod succeeded unexpectedly")
                    .with_name("E_POD_SUCCEEDED")
                    .with_status(StatusCode::SERVICE_UNAVAILABLE);
                return Err(error);
            }
            MCPServerPodStatus::Unknown => {
                let error = Error::generic("Pod status is unknown")
                    .with_name("E_POD_UNKNOWN")
                    .with_status(StatusCode::SERVICE_UNAVAILABLE);
                return Err(error);
            }
        }
        Ok(())
    }

    /// Ensure that the `Pod` for the `MCPServer` is terminated.
    pub async fn ensure_pod_is_terminated(&self, client: &Client) -> Result<()> {
        match self.get_pod_status(client).await? {
            MCPServerPodStatus::NotFound => {
                // Pod is already terminated/not found
            }
            _ => {
                let reason = PodScheduledReason::Terminating;
                let condition = Condition::PodScheduled(reason);
                self.push_condition(client, condition).await?;
                <Self as IntoResource<v1::Pod>>::delete_resource(self, client).await?;
            }
        }
        Ok(())
    }

    /// Update the `status` based on the current state of the associated `Pod`.
    pub async fn reconcile_status_with_pod(&self, client: &Client) -> Result<()> {
        match self.get_pod_status(client).await? {
            MCPServerPodStatus::Running => {
                self.set_phase(client, Phase::Ready).await?;
            }
            MCPServerPodStatus::NotFound => {
                self.set_phase(client, Phase::Idle).await?;
            }
            MCPServerPodStatus::Pending => {
                let reason = PodScheduledReason::Scheduled;
                let condition = Condition::PodScheduled(reason);
                self.push_condition(client, condition).await?;
                self.set_phase(client, Phase::Starting).await?;
            }
            MCPServerPodStatus::Failed { message, .. } => {
                let error = Error::generic(message.unwrap_or("Pod failed to start".to_string()))
                    .with_name("E_POD_FAILED")
                    .with_status(StatusCode::SERVICE_UNAVAILABLE);
                let reason = PodScheduledReason::Failed(error);
                let condition = Condition::PodScheduled(reason);
                self.push_condition(client, condition).await?;
                self.set_phase(client, Phase::Degraded).await?;
            }
            MCPServerPodStatus::Succeeded => {
                let reason = PodScheduledReason::Succeeded;
                let condition = Condition::PodScheduled(reason);
                self.push_condition(client, condition).await?;
                self.set_phase(client, Phase::Idle).await?;
            }
            MCPServerPodStatus::Unknown => {
                let error = Error::generic("Pod status is unknown")
                    .with_name("E_POD_UNKNOWN")
                    .with_status(StatusCode::SERVICE_UNAVAILABLE);
                let reason = PodScheduledReason::Failed(error);
                let condition = Condition::PodScheduled(reason);
                self.push_condition(client, condition).await?;
                self.set_phase(client, Phase::Degraded).await?;
            }
        }
        Ok(())
    }

    /***********************************************************************/
    /* Lifecycle                                                           */
    /***********************************************************************/

    /// Check if the pool can accept more servers based on its limits.
    pub async fn can_pool_accept_more_servers(&self, client: &Client) -> Result<bool> {
        let pool = MCPPool::get_by_name(client, &self.spec.pool).await?;
        let status = pool.status.unwrap_or_default();
        Ok(status.active_servers_count < pool.spec.max_servers_active)
    }

    /// Check if the server was idle for too long and should be stopped.
    pub async fn is_server_stale(&self, client: &Client) -> Result<bool> {
        let pool = MCPPool::get_by_name(client, &self.spec.pool).await?;
        let status = self.get_status(client).await?;

        // --- Get the last request time. If it's None, fallback to
        // --- the `started_at` time, then to `created_at`.
        let last_request = status
            .last_request_at
            .or(status.started_at)
            .unwrap_or(status.created_at);

        // --- Get the idle timeout from the server spec or pool spec
        let idle_timeout = match self.spec.idle_timeout {
            0 => pool.spec.default_idle_timeout,
            _ => self.spec.idle_timeout,
        };

        // --- Calculate the elapsed time since the last request.
        let now = Utc::now();
        let elapsed = now.signed_duration_since(last_request).to_std().unwrap();
        let elapsed_secs = elapsed.as_secs() as i64;
        let is_stale = elapsed_secs > idle_timeout as i64;

        // --- Ensure we are tracking the `stale` condition in the server status.
        if is_stale {
            let reason = StaleReason::IdleTimeout;
            let condition = Condition::Stale(reason);
            self.push_condition(client, condition).await?;
        }

        Ok(is_stale)
    }

    /// Determine if the server should be started based on its status and pool limits.
    pub async fn should_server_be_up(&self, client: &Client) -> Result<bool> {
        let status = self.get_status(client).await?;
        Ok(match status.phase {
            Phase::Idle | Phase::Degraded | Phase::Stopping => false,
            Phase::Ready | Phase::Starting => !self.is_server_stale(client).await?,
            Phase::Requested => self.can_pool_accept_more_servers(client).await?,
        })
    }

    /// Determine if the server should be shutdown based on its status and idle timeout.
    pub async fn should_server_be_down(&self, client: &Client) -> Result<bool> {
        let status = self.get_status(client).await?;
        Ok(match status.phase {
            Phase::Degraded => false, // Don't stop degraded servers automatically, they may need manual intervention.
            Phase::Idle | Phase::Stopping => true,
            Phase::Ready | Phase::Starting => self.is_server_stale(client).await?,
            Phase::Requested => self.is_server_stale(client).await?,
        })
    }

    /// Start or stop the server based on its current status and conditions.
    pub async fn reconcile_server(&self, client: &Client) -> Result<()> {
        if self.should_server_be_up(client).await? {
            self.up(client).await?;
        } else if self.should_server_be_down(client).await? {
            self.down(client).await?;
        }

        // --- Reconcile the server status with the pod status.
        self.reconcile_status_with_pod(client).await?;

        Ok(())
    }

    /***********************************************************************/
    /* Actions                                                             */
    /***********************************************************************/

    /// Requests the server to start.
    pub async fn request(&self, client: &Client) -> Result<()> {
        match self.get_status(client).await?.phase {
            Phase::Ready | Phase::Requested | Phase::Starting => Ok(()),
            Phase::Idle => {
                let reason = RequestReqson::Connection;
                let condition = Condition::Requested(reason);
                self.clear_conditions(client).await?;
                self.push_condition(client, condition).await?;
                self.set_phase(client, Phase::Requested).await?;
                self.notify_requested(client).await?;
                Ok(())
            }
            _ => {
                let error = Error::generic("Server cannot be requested to start")
                    .with_name("E_SERVER_CANNOT_START")
                    .with_status(StatusCode::BAD_REQUEST);
                Err(error)
            }
        }
    }

    /// Start the server `Pod` and `Service`.
    pub async fn up(&self, client: &Client) -> Result<()> {
        self.ensure_pod_is_scheduled(client).await?;
        Ok(())
    }

    /// Shutdown the server `Pod` and `Service`.
    pub async fn down(&self, client: &Client) -> Result<()> {
        self.ensure_pod_is_terminated(client).await?;
        Ok(())
    }

    /// Return a log stream for the server pod.
    pub async fn get_logs(&self, client: &Client) -> Result<impl AsyncBufRead> {
        let status = self.get_pod_status(client).await?;
        if status == MCPServerPodStatus::NotFound {
            return Err(Error::generic("Server is not running".to_string()));
        }
        Api::<v1::Pod>::namespaced(client.clone(), client.default_namespace())
            .log_stream(
                &<Self as IntoResource<v1::Pod>>::resource_name(self),
                &LogParams {
                    container: Some(MCP_SERVER_CONTAINER_NAME.to_string()),
                    follow: true,
                    pretty: false,
                    previous: false,
                    timestamps: true,
                    ..Default::default()
                },
            )
            .await
            .map_err(Error::from)
    }
}
