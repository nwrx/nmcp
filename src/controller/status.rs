use super::{IntoResource, ResourceManager};
use crate::{ErrorInner, MCPServer, Result};
use crate::{MCPServerConditionType as Condition, MCPServerPhase as Phase};
use chrono::Utc;
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::apimachinery::pkg::apis::meta::v1;
use kube::Client;
use std::time::Duration;
use tokio::time;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PodStatus {
    Running,
    Pending,
    Succeeded,
    Failed,
    Unknown,
    NotFound,
}

impl MCPServer {
    /// Sets the phase of the `MCPServer` resource.
    pub async fn set_server_status(&self, client: &Client, condition: Condition) -> Result<Self> {
        let mut status = self.get_status(client).await?;

        // --- Abort early if the last condition is the same as the new one.
        if let Some(last_condition) = status.conditions.last() {
            if last_condition.type_ == condition.to_string() {
                return Ok(self.clone());
            }
        }

        // --- Update the phase in the status
        status.conditions.push(v1::Condition {
            type_: condition.to_string(),
            last_transition_time: v1::Time(Utc::now()),
            observed_generation: self.metadata.generation,
            reason: condition.to_message(),
            message: condition.to_message(),
            status: condition.to_status(),
        });

        // --- Update the "started_at" or "stopped_at" field based on the new phase.
        match condition {
            Condition::PodPending => status.started_at = Some(Utc::now()),
            Condition::PodTerminated => status.stopped_at = Some(Utc::now()),
            Condition::PodFailed(..) => status.stopped_at = Some(Utc::now()),
            Condition::ServiceFailed(..) => status.stopped_at = Some(Utc::now()),
            _ => {}
        }

        // --- Update the phase based on the appended condition.
        status.phase = match condition {
            Condition::Idle => Phase::Idle,
            Condition::Requested => Phase::Requested,

            // Starting
            Condition::PodRunning => Phase::Starting,
            Condition::PodPending => Phase::Starting,
            Condition::ServiceReady => Phase::Starting,
            Condition::ServiceStarting => Phase::Starting,

            // Error
            Condition::PodFailed(..) => Phase::Failed,
            Condition::ServiceFailed(..) => Phase::Failed,
            Condition::PodTerminationFailed(_) => Phase::Failed,

            // Stopping
            Condition::PodTerminating => Phase::Stopping,
            Condition::PodTerminated => Phase::Stopping,
            Condition::ServiceTerminating => Phase::Stopping,
            Condition::ServiceTerminated => Phase::Stopping,

            // Running
            Condition::Running => Phase::Running,
        };

        // --- Set the last_request_at field to None if the condition is "Ready".
        if condition == Condition::Running {
            status.last_request_at = Some(Utc::now());
        }

        // --- Patch the MCPServer resource with the new status
        self.patch_status(client, status).await
    }

    /// Cleanup the `conditions` field of the `MCPServer` resource.
    pub async fn cleanup_server_conditions(&self, client: &Client) -> Result<Self> {
        let mut status = self.get_status(client).await?;
        status.conditions.clear();
        self.patch_status(client, status).await
    }

    /// Register that an `MCPServer` resource has been requested.
    pub async fn register_server_request(&self, client: &Client) -> Result<Self> {
        let mut status = self.get_status(client).await?;
        status.last_request_at = Some(Utc::now());
        status.total_requests += 1;
        self.patch_status(client, status).await
    }

    /// Register that an active connection has been established.
    pub async fn register_server_connection(&self, client: &Client) -> Result<Self> {
        let mut status = self.get_status(client).await?;
        status.current_connections += 1;
        self.patch_status(client, status).await
    }

    /// Register that an active connection has been closed.
    pub async fn unregister_server_connection(&self, client: &Client) -> Result<Self> {
        let mut status = self.get_status(client).await?;
        status.current_connections -= 1;
        self.patch_status(client, status).await
    }

    /// Get the pod status for the given `MCPServer`.
    pub async fn get_server_pod_status(&self, client: &Client) -> Result<PodStatus> {
        let pod = <Self as IntoResource<Pod>>::get_resource(self, client).await;
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

    /// Requests the server to start.
    #[tracing::instrument(name = "RequestServerUp", skip_all)]
    pub async fn request_server_up(&self, client: &Client) -> Result<&Self> {
        loop {
            match self.get_server_pod_status(client).await {
                Ok(PodStatus::Running) => {
                    return Ok(self);
                }
                Ok(PodStatus::Pending) => {
                    time::sleep(Duration::from_secs(1)).await;
                }
                Ok(
                    PodStatus::NotFound
                    | PodStatus::Failed
                    | PodStatus::Succeeded
                    | PodStatus::Unknown,
                ) => {
                    let _ = self.set_server_status(client, Condition::Requested).await?;
                    time::sleep(Duration::from_secs(1)).await;
                }
                Err(error) => {
                    return Err(error);
                }
            }
        }
    }
}
