use super::{
    IntoResource, MCPPool, MCPServer, MCPServerCondition as Condition, MCPServerPhase as Phase,
    MCPServerPodScheduledState as PodScheduledState, MCPServerRequestedState as RequestedState,
    ResourceManager,
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
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PodStatus {
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
pub enum ContainerStatus {
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
        condition.observed_generation = self.metadata.generation;

        // --- Check if the condition already exists and is identical
        let mut conditions = status.conditions.iter();
        if let Some(existing) = conditions.find(|c| c.type_ == condition.type_) {
            if existing.status == condition.status
                && existing.reason == condition.reason
                && existing.message == condition.message
                && existing.observed_generation == condition.observed_generation
            {
                return Ok(()); // No change needed
            }
        }

        // --- Remove existing condition of the same type and add the new one
        status.conditions.retain(|c| c.type_ != condition.type_);
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

    /// Clear the connected clients count.
    pub async fn clear_connected_clients(&self, client: &Client) -> Result<()> {
        let mut status = self.get_status(client).await?;
        if status.current_connections > 0 {
            status.current_connections = 0;
            let _ = self.patch_status(client, status).await?;
        }
        Ok(())
    }

    /***********************************************************************/
    /* Pod                                                                 */
    /***********************************************************************/

    /// Get the pod status for the given `MCPServer`.
    pub async fn get_pod_status(&self, client: &Client) -> Result<PodStatus> {
        match <Self as IntoResource<v1::Pod>>::get_resource(self, client).await {
            Ok(pod) => {
                let status = pod.status.unwrap_or_default();
                let phase = status.phase.unwrap_or_default();
                match phase.as_str() {
                    "Running" => Ok(PodStatus::Running),
                    "Pending" => Ok(PodStatus::Pending),
                    "Succeeded" => Ok(PodStatus::Succeeded),
                    "Failed" => {
                        let message = status.message.clone();
                        let reason = status.reason.clone();
                        Ok(PodStatus::Failed { message, reason })
                    }
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

    /// Get the container status for the given `MCPServer`.
    pub async fn get_pod_container_status(&self, client: &Client) -> Result<ContainerStatus> {
        match <Self as IntoResource<v1::Pod>>::get_resource(self, client).await {
            Ok(pod) => {
                if let Some(status) = pod.status {
                    if let Some(container_statuses) = status.container_statuses {
                        for container in container_statuses {
                            if container.name == MCP_SERVER_CONTAINER_NAME {
                                if let Some(state) = container.state {
                                    if let Some(waiting) = state.waiting {
                                        return Ok(ContainerStatus::Waiting(waiting));
                                    }
                                    if let Some(running) = state.running {
                                        return Ok(ContainerStatus::Running(running));
                                    }
                                    if let Some(terminated) = state.terminated {
                                        return Ok(ContainerStatus::Terminated(terminated));
                                    }
                                }
                            }
                        }
                        Ok(ContainerStatus::ContainerNotFound)
                    } else {
                        Ok(ContainerStatus::ContainerNotFound)
                    }
                } else {
                    Ok(ContainerStatus::PodNotFound)
                }
            }
            Err(error) => match error.source() {
                ErrorInner::KubeError(kube::Error::Api(error)) if error.code == 404 => {
                    Ok(ContainerStatus::PodNotFound)
                }
                _ => Err(error),
            },
        }
    }

    /// Ensure that the `Pod` for the `MCPServer` is scheduled.
    pub async fn ensure_pod_is_scheduled(&self, client: &Client) -> Result<()> {
        if self.get_pod_status(client).await? == PodStatus::NotFound {
            tracing::info!("Pod not found, creating it for server");
            self.notify_started(client).await?;
            let _ = <Self as IntoResource<v1::Pod>>::patch_resource(self, client).await?;
        }
        Ok(())
    }

    /// Ensure that the `Pod` for the `MCPServer` is terminated.
    pub async fn ensure_pod_is_terminated(&self, client: &Client) -> Result<()> {
        if self.get_pod_status(client).await? != PodStatus::NotFound {
            <Self as IntoResource<v1::Pod>>::delete_resource(self, client).await?;
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

        // --- If stale, update the requested state condition.
        if is_stale {
            let reason = RequestedState::IdleTimeout;
            let condition = Condition::Requested(reason);
            self.push_condition(client, condition).await?;
            self.set_phase(client, Phase::Stopping).await?;
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

    /// Return a `Future` that will finish once the server is in the `Ready` phase.
    pub async fn wait_until_ready(&self, client: &Client, timeout: Option<Duration>) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        let start_time = std::time::Instant::now();

        loop {
            let status = self.get_status(client).await?;
            match status.phase {
                Phase::Ready => return Ok(()),
                Phase::Requested | Phase::Starting | Phase::Degraded => {
                    let _ = interval.tick().await;
                }
                Phase::Idle | Phase::Stopping => {
                    return Err(Error::generic("Server is idle")
                        .with_name("E_SERVER_IDLE_STALE")
                        .with_status(StatusCode::SERVICE_UNAVAILABLE));
                }
            }

            // --- Check for timeout.
            if let Some(timeout) = timeout {
                if start_time.elapsed() >= timeout {
                    return Err(Error::generic("Server did not become ready in time")
                        .with_name("E_SERVER_NOT_READY")
                        .with_status(StatusCode::REQUEST_TIMEOUT));
                }
            }
        }
    }
    /***********************************************************************/
    /* Reconciliation                                                      */
    /***********************************************************************/

    /// Update the `status` based on the current state of the associated `Pod`.
    pub async fn reconcile_status_with_pod(&self, client: &Client) -> Result<()> {
        let current_status = self.get_status(client).await?;
        let pod_status = self.get_pod_status(client).await?;

        match current_status.phase {
            Phase::Requested => match pod_status {
                PodStatus::NotFound => {
                    // Pod hasn't been created yet, keep waiting
                }
                PodStatus::Pending => {
                    let reason = PodScheduledState::Scheduled;
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Starting).await?;
                }
                PodStatus::Running => {
                    let reason = PodScheduledState::Running;
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Ready).await?;
                }
                PodStatus::Failed { message, .. } => {
                    let error =
                        Error::generic(message.unwrap_or("Pod failed to start".to_string()));
                    let reason = PodScheduledState::Failed(error);
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Degraded).await?;
                }
                PodStatus::Succeeded => {
                    let reason = PodScheduledState::Succeeded;
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Idle).await?;
                }
                PodStatus::Unknown => {
                    let error = Error::generic("Pod status is unknown");
                    let reason = PodScheduledState::Failed(error);
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Degraded).await?;
                }
            },
            Phase::Starting => match pod_status {
                PodStatus::Running => {
                    let reason = PodScheduledState::Running;
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Ready).await?;
                }
                PodStatus::Failed { message, .. } => {
                    let error =
                        Error::generic(message.unwrap_or("Pod failed to start".to_string()));
                    let reason = PodScheduledState::Failed(error);
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Degraded).await?;
                }
                PodStatus::NotFound => {
                    // Pod was deleted while starting, go back to requested
                    self.set_phase(client, Phase::Requested).await?;
                }
                PodStatus::Succeeded => {
                    let reason = PodScheduledState::Succeeded;
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Idle).await?;
                }
                PodStatus::Unknown => {
                    let error = Error::generic("Pod status is unknown");
                    let reason = PodScheduledState::Failed(error);
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Degraded).await?;
                }
                PodStatus::Pending => {
                    // Still starting, no action needed
                }
            },
            Phase::Ready => match pod_status {
                PodStatus::Running => {
                    // Everything is good, no action needed
                }
                PodStatus::Failed { message, .. } => {
                    let error =
                        Error::generic(message.unwrap_or("Pod failed while running".to_string()));
                    let reason = PodScheduledState::Failed(error);
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Degraded).await?;
                }
                PodStatus::NotFound => {
                    // Pod was deleted while ready, transition to stopping
                    self.set_phase(client, Phase::Stopping).await?;
                }
                PodStatus::Succeeded => {
                    let reason = PodScheduledState::Succeeded;
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Idle).await?;
                }
                PodStatus::Unknown => {
                    let error = Error::generic("Pod status is unknown");
                    let reason = PodScheduledState::Failed(error);
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Degraded).await?;
                }
                PodStatus::Pending => {
                    // Pod restarted, go back to starting
                    self.set_phase(client, Phase::Starting).await?;
                }
            },
            Phase::Stopping => match pod_status {
                PodStatus::NotFound => {
                    let reason = PodScheduledState::Succeeded;
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Idle).await?;
                    self.clear_connected_clients(client).await?;
                }
                PodStatus::Succeeded => {
                    let reason = PodScheduledState::Succeeded;
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Idle).await?;
                    self.clear_connected_clients(client).await?;
                }
                PodStatus::Running | PodStatus::Pending => {
                    let reason = PodScheduledState::Terminating;
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                }
                PodStatus::Failed { message, .. } => {
                    let error = Error::generic(
                        message.unwrap_or("Pod failed during termination".to_string()),
                    );
                    let reason = PodScheduledState::Failed(error);
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Degraded).await?;
                }
                PodStatus::Unknown => {
                    let error = Error::generic("Pod status is unknown during termination");
                    let reason = PodScheduledState::Failed(error);
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Degraded).await?;
                }
            },
            Phase::Idle => match pod_status {
                PodStatus::NotFound => {
                    let reason = PodScheduledState::Succeeded;
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                }
                PodStatus::Running | PodStatus::Pending => {
                    // Pod shouldn't be running while idle, transition to stopping
                    self.set_phase(client, Phase::Stopping).await?;
                }
                PodStatus::Succeeded => {
                    let reason = PodScheduledState::Succeeded;
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                }
                PodStatus::Failed { message, .. } => {
                    let error =
                        Error::generic(message.unwrap_or("Pod failed while idle".to_string()));
                    let reason = PodScheduledState::Failed(error);
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Degraded).await?;
                }
                PodStatus::Unknown => {
                    let error = Error::generic("Pod status is unknown while idle");
                    let reason = PodScheduledState::Failed(error);
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Degraded).await?;
                }
            },
            Phase::Degraded => match pod_status {
                PodStatus::Running => {
                    // Pod recovered, transition back to ready
                    let reason = PodScheduledState::Running;
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Ready).await?;
                }
                PodStatus::NotFound => {
                    // Pod was cleaned up, transition to idle
                    let reason = PodScheduledState::Succeeded;
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Idle).await?;
                    self.clear_connected_clients(client).await?;
                }
                PodStatus::Pending => {
                    // Pod is being recreated, transition to starting
                    let reason = PodScheduledState::Scheduled;
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Starting).await?;
                }
                PodStatus::Succeeded => {
                    // Pod succeeded, transition to idle
                    let reason = PodScheduledState::Succeeded;
                    let condition = Condition::PodScheduled(reason);
                    self.push_condition(client, condition).await?;
                    self.set_phase(client, Phase::Idle).await?;
                }
                PodStatus::Failed { .. } => {
                    // Stay in degraded state
                }
                PodStatus::Unknown => {
                    // Stay in degraded state
                }
            },
        }
        Ok(())
    }

    /// Start or stop the server based on its current status and conditions.
    pub async fn reconcile_server(&self, client: &Client) -> Result<()> {
        if self.should_server_be_up(client).await? {
            self.ensure_pod_is_scheduled(client).await?;
        } else if self.should_server_be_down(client).await? {
            self.ensure_pod_is_terminated(client).await?;
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
        self.notify_request(client).await?;
        match self.get_status(client).await?.phase {
            Phase::Ready | Phase::Requested | Phase::Starting => Ok(()),
            Phase::Idle | Phase::Degraded | Phase::Stopping => {
                self.set_phase(client, Phase::Requested).await?;
                self.notify_requested(client).await?;
                Ok(())
            }
        }
    }

    /// Request the server to stop.
    pub async fn shutdown(&self, client: &Client) -> Result<()> {
        match self.get_status(client).await?.phase {
            Phase::Idle | Phase::Stopping => Ok(()),
            Phase::Ready | Phase::Starting | Phase::Degraded => {
                self.set_phase(client, Phase::Stopping).await?;
                Ok(())
            }
            Phase::Requested => {
                self.set_phase(client, Phase::Idle).await?;
                Ok(())
            }
        }
    }

    /// Return a log stream for the server pod.
    pub async fn get_logs(&self, client: &Client) -> Result<impl AsyncBufRead> {
        Api::<v1::Pod>::namespaced(client.clone(), client.default_namespace())
            .log_stream(
                &<Self as IntoResource<v1::Pod>>::resource_name(self),
                &LogParams {
                    container: Some(MCP_SERVER_CONTAINER_NAME.to_string()),
                    follow: true,
                    pretty: true,
                    previous: false,
                    timestamps: true,
                    ..Default::default()
                },
            )
            .await
            .map_err(Error::from)
    }
}
