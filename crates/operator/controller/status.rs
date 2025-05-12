use super::{Controller, MCP_SERVER_OPERATOR_MANAGER};
use crate::{Error, MCPServer, MCPServerConditionType, MCPServerPhase, Result};
use chrono::Utc;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{Condition, Time};
use kube::api::{Patch, PatchParams};
use kube::{Api, ResourceExt};
use serde_json::json;

impl Controller {
    /// Sets the phase of the MCPServer resource.
    pub async fn set_server_phase(&self, server: &MCPServer, phase: MCPServerPhase) -> Result<()> {
        // --- Check if the server is already in the desired phase
        if let Some(status) = &server.status {
            if status.phase == phase {
                return Ok(());
            }
        }

        // --- Update the "started_at" or "stopped_at" field based on the new phase.
        let mut status = server.status.clone().unwrap_or_default();
        match phase {
            MCPServerPhase::Starting => status.started_at = Some(Utc::now()),
            MCPServerPhase::Stopping => status.stopped_at = Some(Utc::now()),
            _ => {}
        }

        // --- Patch the MCPServer resource with the new status
        let _ = Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace())
            .patch_status(
                &server.name_any(),
                &PatchParams::apply(MCP_SERVER_OPERATOR_MANAGER).force(),
                &Patch::Merge(&json!({ "status": status })),
            )
            .await
            .map_err(Error::KubeError);

        Ok(())
    }

    /// Push a condition on the MCPServer resource.
    pub async fn set_server_condition(
        &self,
        server: &MCPServer,
        condition: MCPServerConditionType,
        message: Option<String>,
    ) -> Result<()> {
        let mut status = server.status.clone().unwrap_or_default();

        // --- Update the phase in the status
        status.conditions.push(Condition {
            type_: condition.to_string(),
            last_transition_time: Time(Utc::now()),
            observed_generation: server.metadata.generation,
            reason: condition.to_message(),
            message: message.unwrap_or_else(|| condition.to_message()),
            status: condition.to_status(),
        });

        // --- Patch the MCPServer resource with the new status
        let _ = Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace())
            .patch_status(
                &server.name_any(),
                &PatchParams::apply(MCP_SERVER_OPERATOR_MANAGER).force(),
                &Patch::Merge(&json!({ "status": status })),
            )
            .await
            .map_err(Error::KubeError);

        Ok(())
    }
}
