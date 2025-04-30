use kube::ResourceExt;
use std::collections::BTreeMap;
use uuid::Uuid;
use k8s_openapi::api::core::v1::{Container, Pod, PodSpec, ContainerPort, EnvVar, ResourceRequirements, SecurityContext};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;

use crate::utils::Result;
use crate::server::server_crd::MCPServer;
use crate::server::server_controller::MCPServerController;

impl MCPServerController {
    /// Create a Pod for a MCPServer
    pub fn create_pod(server: &MCPServer, pod_name: &str, namespace: &str) -> Result<Pod> {
        
        // --- Create labels
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), pod_name.to_string());
        labels.insert("unmcp.dev/server".to_string(), server.name_any());
        labels.insert("unmcp.dev/pool".to_string(), server.spec.pool.clone());
        
        // Convert resource requirements
        let resources = if let Some(res) = &server.spec.resources.requests {
            let mut requests = BTreeMap::new();
            if let Some(cpu) = &res.cpu {
                requests.insert("cpu".to_string(), Quantity(cpu.clone()));
            }
            if let Some(memory) = &res.memory {
                requests.insert("memory".to_string(), Quantity(memory.clone()));
            }
            
            let mut limits = BTreeMap::new();
            if let Some(limit_res) = &server.spec.resources.limits {
                if let Some(cpu) = &limit_res.cpu {
                    limits.insert("cpu".to_string(), Quantity(cpu.clone()));
                }
                if let Some(memory) = &limit_res.memory {
                    limits.insert("memory".to_string(), Quantity(memory.clone()));
                }
            }
            
            Some(ResourceRequirements {
                requests: if !requests.is_empty() { Some(requests) } else { None },
                limits: if !limits.is_empty() { Some(limits) } else { None },
                ..ResourceRequirements::default()
            })
        } else {
            None
        };
        
        // Convert security context
        let security_context = if let Some(sc) = &server.spec.security_context {
            let k8s_sc = SecurityContext {
                run_as_non_root: Some(sc.run_as_non_root),
                run_as_user: sc.run_as_user,
                allow_privilege_escalation: Some(sc.allow_privilege_escalation),
                ..Default::default()
            };
            
            Some(k8s_sc)
        } else {
            None
        };
        
        // Create environment variables
        let mut env_vars = Vec::new();
        for (key, value) in &server.spec.server.env {
            env_vars.push(EnvVar {
                name: key.clone(),
                value: Some(value.clone()),
                value_from: None,
            });
        }
        
        // --- Add server metadata as environment variables
        if let Some(server_type) = &server.spec.metadata.server_type {
            env_vars.push(EnvVar {
                name: "MCP_SERVER_TYPE".to_string(),
                value: Some(server_type.clone()),
                value_from: None,
            });
        }
        
        // --- Add a unique server ID
        let server_uuid = Uuid::new_v4().to_string();
        env_vars.push(EnvVar {
            name: "MCP_SERVER_UUID".to_string(),
            value: Some(server_uuid),
            value_from: None,
        });
        
        // Create container ports
        let container_ports = vec![ContainerPort {
            container_port: server.spec.networking.port,
            name: Some("http".to_string()),
            ..ContainerPort::default()
        }];
        
        // Create container
        let container = Container {
            name: "server".to_string(),
            image: Some(server.spec.image.clone()),
            command: Some(vec![server.spec.server.command.clone()]),
            args: Some(server.spec.server.args.clone()),
            env: Some(env_vars),
            ports: Some(container_ports),
            resources,
            security_context,
            ..Container::default()
        };
        
        // Create pod
        Ok(Pod {
            metadata: kube::api::ObjectMeta {
                name: Some(pod_name.to_string()),
                namespace: Some(namespace.to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: vec![container],
                restart_policy: Some("Always".to_string()),
                ..PodSpec::default()
            }),
            ..Pod::default()
        })
    }
}
