use chrono::Utc;
use serde_json::json;
use k8s_openapi::api::core::v1::{Pod, Service};
use kube::api::{Api, Patch, PatchParams};
use kube::client::Client;

use crate::utils::{Error, Result};
use crate::server::server_crd::{MCPServer, Condition};
use crate::server::server_controller::MCPServerController;

impl MCPServerController {
    /// Update the status of a MCPServer resource
    pub(crate) async fn update_server_status(
        client: Client,
        server: &MCPServer,
        namespace: &str,
        name: &str,
        pod: Option<&Pod>,
        service: Option<&Service>,
    ) -> Result<()> {
        // Create API for MCPServer resources
        let servers_api: Api<MCPServer> = Api::namespaced(client, namespace);
        
        // Determine server phase based on Pod status
        let phase = if let Some(pod) = pod {
            if let Some(pod_status) = &pod.status {
                if let Some(phase) = &pod_status.phase {
                    match phase.as_str() {
                        "Running" => "Available",
                        "Pending" => "Pending",
                        "Failed" => "Failed",
                        _ => "Unknown",
                    }
                } else {
                    "Unknown"
                }
            } else {
                "Unknown"
            }
        } else {
            "Pending"
        };
        
        // Determine server endpoint URL
        let server_endpoint = if let Some(service) = service {
            if let Some(svc_status) = &service.status {
                if let Some(ingress) = &svc_status.load_balancer {
                    if let Some(ingress_items) = &ingress.ingress {
                        if !ingress_items.is_empty() {
                            let ip = ingress_items[0].ip.clone().unwrap_or_default();
                            if !ip.is_empty() {
                                Some(format!(
                                    "{}://{}:{}",
                                    server.spec.networking.protocol.to_lowercase(),
                                    ip,
                                    server.spec.networking.port
                                ))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        // Prepare conditions
        let mut conditions = Vec::new();
        
        // Add Ready condition
        conditions.push(Condition {
            type_: "Ready".to_string(),
            status: if phase == "Available" { "True".to_string() } else { "False".to_string() },
            reason: Some(phase.to_string()),
            message: Some(format!("Server is in {} state", phase)),
            last_transition_time: Some(Utc::now()),
        });
        
        // Create status patch
        let mut patch = json!({
            "status": {
                "phase": phase,
                "conditions": conditions,
                "startTime": Utc::now(),
            }
        });
        
        // Add server endpoint and UUID if available
        if let Some(endpoint) = server_endpoint {
            patch["status"]["serverEndpoint"] = json!(endpoint);
        }
        
        if let Some(pod) = pod {
            if let Some(env) = pod.spec.as_ref().and_then(|s| s.containers.first()).and_then(|c| c.env.as_ref()) {
                for var in env {
                    if var.name == "MCP_SERVER_UUID" {
                        if let Some(uuid) = &var.value {
                            patch["status"]["serverUuid"] = json!(uuid);
                        }
                    }
                }
            }
        }
        
        // Apply status patch
        let patch_params = PatchParams::apply("unmcp-operator");
        servers_api.patch_status(name, &patch_params, &Patch::Merge(patch))
            .await
            .map_err(Error::KubeError)?;
        
        Ok(())
    }
}