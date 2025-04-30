use chrono::Utc;
use serde_json::json;
use kube::api::{Api, Patch, PatchParams};
use kube::client::Client;
use kube::ResourceExt;
use tokio::sync::RwLock;
use std::sync::Arc;

use crate::utils::{Error, Result};
use crate::MCPServer;
use crate::pool::pool_controller::MCPPoolContext;
use super::{MCPPool, MCPPoolController, Condition};

impl MCPPoolController {
    /// Update the status of a MCPPool resource
    pub(crate) async fn update_pool_status(
        context: Arc<RwLock<MCPPoolContext>>, 
        pool: &MCPPool
    ) -> Result<()> {
        // Get the client from the controller context
        let ctx = context.read().await;
        let client = ctx.client.clone();
        
        // Extract namespace and name from the pool resource
        let namespace = pool.namespace().ok_or_else(|| Error::Generic("MCPPool has no namespace".to_string()))?;
        let name = pool.name_any();
        
        // Fetch all servers that belong to this pool
        let servers = Self::get_servers_for_pool(&client, &namespace, &name).await?;
        
        // Create API for MCPPool resources
        let pools_api: Api<MCPPool> = Api::namespaced(client, &namespace);
        
        // Count servers by phase
        let mut available_servers = 0;
        let mut in_use_servers = 0;
        let mut pending_servers = 0;
        
        for server in &servers {
            if let Some(status) = &server.status {
                if let Some(phase) = &status.phase {
                    match phase.as_str() {
                        "Available" => available_servers += 1,
                        "InUse" => in_use_servers += 1,
                        "Pending" => pending_servers += 1,
                        _ => {}
                    }
                }
            } else {
                pending_servers += 1;
            }
        }
        
        // Prepare conditions
        let mut conditions = Vec::new();
        
        // Add Ready condition
        let is_ready = available_servers >= pool.spec.min_servers;
        conditions.push(Condition {
            type_: "Ready".to_string(),
            status: if is_ready { "True".to_string() } else { "False".to_string() },
            reason: Some(if is_ready { "MinServersAvailable".to_string() } else { "NotEnoughServers".to_string() }),
            message: Some(format!(
                "Pool has {}/{} minimum required available servers",
                available_servers, pool.spec.min_servers
            )),
            last_transition_time: Some(Utc::now()),
        });
        
        // Add Scaled condition
        let total_servers = servers.len() as i32;
        let is_scaled = total_servers >= pool.spec.min_servers && total_servers <= pool.spec.max_servers;
        conditions.push(Condition {
            type_: "Scaled".to_string(),
            status: if is_scaled { "True".to_string() } else { "False".to_string() },
            reason: Some(if total_servers < pool.spec.min_servers {
                "BelowMinimum".to_string()
            } else if total_servers > pool.spec.max_servers {
                "AboveMaximum".to_string()
            } else {
                "OptimalSize".to_string()
            }),
            message: Some(format!(
                "Pool has {} servers (min: {}, max: {})",
                total_servers, pool.spec.min_servers, pool.spec.max_servers
            )),
            last_transition_time: Some(Utc::now()),
        });
        
        // Calculate metrics if autoscaling is enabled
        let metrics = if pool.spec.autoscaling.enabled && !servers.is_empty() {
            let mut total_cpu = 0.0;
            let mut total_memory = 0.0;
            let mut total_requests = 0;
            let mut active_connections = 0;
            let mut servers_with_metrics = 0;
            
            for server in &servers {
                if let Some(status) = &server.status {
                    if let Some(server_metrics) = &status.metrics {
                        servers_with_metrics += 1;
                        
                        if let Some(cpu) = &server_metrics.cpu_usage {
                            if let Ok(cpu_value) = cpu.trim_end_matches('%').parse::<f64>() {
                                total_cpu += cpu_value;
                            }
                        }
                        
                        if let Some(memory) = &server_metrics.memory_usage {
                            if let Ok(memory_value) = memory.trim_end_matches('%').parse::<f64>() {
                                total_memory += memory_value;
                            }
                        }
                        
                        if let Some(requests) = server_metrics.request_count {
                            total_requests += requests;
                        }
                        
                        if let Some(connections) = server_metrics.active_connections {
                            active_connections += connections;
                        }
                    }
                }
            }
            
            if servers_with_metrics > 0 {
                let avg_cpu = total_cpu / servers_with_metrics as f64;
                let avg_memory = total_memory / servers_with_metrics as f64;
                
                json!({
                    "averageCpuUtilization": format!("{:.2}%", avg_cpu),
                    "averageMemoryUtilization": format!("{:.2}%", avg_memory),
                    "totalRequests": total_requests,
                    "activeConnections": active_connections
                })
            } else {
                json!(null)
            }
        } else {
            json!(null)
        };
        
        // Create status patch
        let status_patch = json!({
            "status": {
                "availableServers": available_servers,
                "inUseServers": in_use_servers,
                "pendingServers": pending_servers,
                "conditions": conditions,
                "metrics": metrics
            }
        });
        
        // Apply status patch
        let patch_params = PatchParams::apply("unmcp-operator");
        pools_api.patch_status(&name, &patch_params, &Patch::Merge(status_patch))
            .await
            .map_err(Error::KubeError)?;
        
        Ok(())
    }
    
    /// Get all servers that belong to this pool
    async fn get_servers_for_pool(client: &Client, namespace: &str, pool_name: &str) -> Result<Vec<MCPServer>> {
        // Create API for MCPServer resources
        let servers_api: Api<MCPServer> = Api::namespaced(client.clone(), namespace);
        
        // Get all servers
        let servers = servers_api.list(&Default::default())
            .await
            .map_err(Error::KubeError)?;
            
        // Filter servers that belong to this pool
        let pool_servers = servers.items
            .into_iter()
            .filter(|server| {
                if let Some(pool) = Some(&server.spec.pool) {
                    pool == pool_name
                } else {
                    false
                }
            })
            .collect();
            
        Ok(pool_servers)
    }
}