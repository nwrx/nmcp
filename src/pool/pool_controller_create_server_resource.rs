use kube::api::ResourceExt;

use crate::server::{MCPServerSpec, ServerConfig, ServerMetadata};
use crate::utils::Result;
use crate::pool::pool_crd::MCPPool;
use crate::pool::pool_controller::MCPPoolController;
use crate::MCPServer;

impl MCPPoolController {
    /// Create a MCPServer resource for an MCPPool
    pub fn create_server_resource(pool: &MCPPool, server_name: &str, namespace: &str) -> Result<MCPServer> {
        
        // Define server metadata
        let metadata = kube::api::ObjectMeta {
            name: Some(server_name.to_string()),
            namespace: Some(namespace.to_string()),
            labels: Some({
                let mut labels = std::collections::BTreeMap::new();
                labels.insert("unmcp.dev/pool".to_string(), pool.name_any());
                labels.insert("app".to_string(), "mcp-server".to_string());
                labels
            }),
            ..Default::default()
        };

        // Create server spec using pool defaults
        let mut server_spec = MCPServerSpec::default();
        
        // Apply pool's server defaults if available
        if let Some(image) = &pool.spec.server_defaults.image {
            server_spec.image = image.clone();
        }

        if let Some(resources) = &pool.spec.server_defaults.resources {
            server_spec.resources = resources.clone();
        }

        if let Some(security_context) = &pool.spec.server_defaults.security_context {
            server_spec.security_context = Some(security_context.clone());
        }

        if let Some(networking) = &pool.spec.server_defaults.networking {
            server_spec.networking = networking.clone();
        }

        if let Some(liveness_probe) = &pool.spec.server_defaults.liveness_probe {
            server_spec.liveness_probe = Some(liveness_probe.clone());
        }

        // Set required fields
        server_spec.pool = pool.name_any();
        server_spec.server = ServerConfig {
            command: "mcp-server".to_string(),
            args: vec!["--pool".to_string(), pool.name_any()],
            env: {
                let mut env = std::collections::BTreeMap::new();
                env.insert("MCP_POOL".to_string(), pool.name_any());
                env
            },
        };

        // Set metadata 
        server_spec.metadata = ServerMetadata {
            server_type: Some("pool-managed".to_string()),
            capabilities: vec!["text".to_string(), "image".to_string()],
        };

        // Create the MCPServer with the applied configuration
        let server = MCPServer {
            metadata,
            spec: server_spec,
            status: None,
        };

        Ok(server)
    }
}