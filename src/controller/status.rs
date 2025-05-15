use super::{Controller, MCP_SERVER_OPERATOR_MANAGER};
use crate::{Error, MCPServer, Result};
use crate::{MCPServerConditionType as Condition, MCPServerPhase as Phase};
use chrono::Utc;
use k8s_openapi::apimachinery::pkg::apis::meta::v1;
use kube::api::{Patch, PatchParams};
use kube::{Api, ResourceExt};
use serde_json::json;

impl Controller {
    /// Sets the phase of the MCPServer resource.
    pub async fn set_server_status(&self, server: &MCPServer, condition: Condition) -> Result<()> {
        let server = self.get_server_by_name(&server.name_any()).await?;
        let mut status = server.status.clone().unwrap_or_default();
        let old_phase = status.phase.clone();

        // --- Abort early if the last condition is the same as the new one.
        if let Some(last_condition) = status.conditions.last() {
            if last_condition.type_ == condition.to_string() {
                tracing::debug!(
                    "Skipping MCPServer status update: last condition is the same as new one: '{}'",
                    condition
                );
                return Ok(());
            }
        }

        // --- Update the phase in the status
        status.conditions.push(v1::Condition {
            type_: condition.to_string(),
            last_transition_time: v1::Time(Utc::now()),
            observed_generation: server.metadata.generation,
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

            // Stopping
            Condition::PodTerminating => Phase::Stopping,
            Condition::PodTerminated => Phase::Stopping,
            Condition::ServiceTerminating => Phase::Stopping,
            Condition::ServiceTerminated => Phase::Stopping,

            // Running
            Condition::Running => Phase::Running,
            _ => status.phase,
        };

        // --- Set the last_request_at field to None if the condition is "Ready".
        if condition == Condition::Running {
            status.last_request_at = Some(Utc::now());
        }

        // --- Patch the MCPServer resource with the new status
        tracing::info!(
            "Updating MCPServer status: {:?} -> {:?} ({:?})",
            old_phase,
            status.phase,
            condition,
        );
        Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace())
            .patch_status(
                &server.name_any(),
                &PatchParams::apply(MCP_SERVER_OPERATOR_MANAGER),
                &Patch::Merge(&json!({ "status": status })),
            )
            .await
            .map_err(Error::from)?;

        Ok(())
    }

    /// Cleanup the `conditions` field of the MCPServer resource.
    pub async fn cleanup_server_conditions(&self, server: &MCPServer) -> Result<()> {
        let server = self.get_server_by_name(&server.name_any()).await?;
        let mut status = server.status.clone().unwrap_or_default();
        status.conditions.clear();

        // --- Patch the MCPServer resource with the new status
        Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace())
            .patch_status(
                &server.name_any(),
                &PatchParams::apply(MCP_SERVER_OPERATOR_MANAGER),
                &Patch::Merge(&json!({ "status": status })),
            )
            .await
            .map_err(Error::from)?;

        Ok(())
    }

    /// Register that an MCPServer resource has been requested.
    pub async fn register_server_request(&self, server: &MCPServer) -> Result<()> {
        let server = self.get_server_by_name(&server.name_any()).await?;
        let mut status = server.status.clone().unwrap_or_default();
        status.last_request_at = Some(Utc::now());
        status.total_requests += 1;

        // --- Update the MCPServer resource with the new status
        Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace())
            .patch_status(
                &server.name_any(),
                &PatchParams::apply(MCP_SERVER_OPERATOR_MANAGER),
                &Patch::Merge(&json!({ "status": status })),
            )
            .await
            .map_err(Error::from)?;

        Ok(())
    }

    /// Register that an active connection has been established.
    pub async fn register_server_connection(&self, server: &MCPServer) -> Result<()> {
        let server = self.get_server_by_name(&server.name_any()).await?;
        let mut status = server.status.clone().unwrap_or_default();
        status.current_connections += 1;

        // --- Update the MCPServer resource with the new status
        Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace())
            .patch_status(
                &server.name_any(),
                &PatchParams::apply(MCP_SERVER_OPERATOR_MANAGER),
                &Patch::Merge(&json!({ "status": status })),
            )
            .await
            .map_err(Error::from)?;

        Ok(())
    }

    /// Register that an active connection has been closed.
    pub async fn unregister_server_connection(&self, server: &MCPServer) -> Result<()> {
        let server = self.get_server_by_name(&server.name_any()).await?;
        let mut status = server.status.clone().unwrap_or_default();
        status.current_connections -= 1;

        // --- Update the MCPServer resource with the new status
        Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace())
            .patch_status(
                &server.name_any(),
                &PatchParams::apply(MCP_SERVER_OPERATOR_MANAGER),
                &Patch::Merge(&json!({ "status": status })),
            )
            .await
            .map_err(Error::from)?;

        Ok(())
    }
}
