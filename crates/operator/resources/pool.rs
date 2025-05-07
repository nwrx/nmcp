use super::MCPServer;
use crate::MCPPool;
use k8s_openapi::api::core::v1;
use kube::ResourceExt;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

impl MCPPool {
    /// Returns the labels for the MCPPool.
    pub fn labels(&self) -> BTreeMap<String, String> {
        let uid = self.metadata.uid.clone().unwrap_or_default();
        let mut labels = BTreeMap::new();
        labels.insert("unmcp.dev/pool".to_string(), self.name_any());
        labels.insert("unmcp.dev/uid".to_string(), uid);
        labels
    }

    /// Return the URL for the MCPPool resource.
    pub fn url(&self) -> String {
        format!("/api/v1/pools/{}", self.name_any())
    }

    /// Returns the name of the MCPPool.
    pub fn into_response(&self, servers: Option<Vec<MCPServer>>) -> MCPPoolResponse {
        let status = self.status.clone().unwrap_or_default();
        MCPPoolResponse {
            id: self.metadata.uid.clone().unwrap_or_default(),
            name: self.name_any(),
            namespace: self.namespace().unwrap_or_default(),
            max_servers_limit: self.spec.max_servers_limit,
            max_servers_active: self.spec.max_servers_active,
            default_resources: self.spec.default_resources.clone(),
            default_idle_timeout: self.spec.default_idle_timeout,
            active_servers_count: status.active_servers_count,
            pending_servers_count: status.pending_servers_count,
            unmanaged_servers_count: status.unmanaged_servers_count,
            managed_servers_count: status.managed_servers_count,
            total_servers_count: status.total_servers_count,
            url: self.url(),
            servers: servers
                .unwrap_or_default()
                .into_iter()
                .map(|s| format!("/api/v1/servers/{}", s.name_any()))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MCPPoolResponse {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub max_servers_limit: u32,
    pub max_servers_active: u32,
    pub default_resources: v1::ResourceRequirements,
    pub default_idle_timeout: u32,
    pub active_servers_count: u32,
    pub pending_servers_count: u32,
    pub unmanaged_servers_count: u32,
    pub managed_servers_count: u32,
    pub total_servers_count: u32,
    pub url: String,
    pub servers: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MCPPoolStatus;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

    #[test]
    fn test_mcp_pool_labels() {
        let pool = MCPPool {
            metadata: ObjectMeta {
                name: Some("test-pool".to_string()),
                namespace: Some("default".to_string()),
                uid: Some("12345678-1234-1234-1234-123456789012".to_string()),
                ..Default::default()
            },
            spec: Default::default(),
            status: None,
        };

        let labels = pool.labels();
        assert_eq!(labels.len(), 2);
        assert_eq!(labels.get("unmcp.dev/pool"), Some(&"test-pool".to_string()));
        assert_eq!(
            labels.get("unmcp.dev/uid"),
            Some(&"12345678-1234-1234-1234-123456789012".to_string())
        );
    }

    #[test]
    fn test_mcp_pool_labels_no_uid() {
        let pool = MCPPool {
            metadata: ObjectMeta {
                name: Some("test-pool".to_string()),
                namespace: Some("default".to_string()),
                uid: None,
                ..Default::default()
            },
            spec: Default::default(),
            status: None,
        };

        let labels = pool.labels();
        assert_eq!(labels.len(), 2);
        assert_eq!(labels.get("unmcp.dev/pool"), Some(&"test-pool".to_string()));
        assert_eq!(labels.get("unmcp.dev/uid"), Some(&"".to_string()));
    }

    #[test]
    fn test_mcp_pool_response_from_pool_no_status() {
        let response = MCPPool {
            metadata: ObjectMeta {
                name: Some("test-pool".to_string()),
                namespace: Some("default".to_string()),
                uid: Some("12345678-1234-1234-1234-123456789012".to_string()),
                ..Default::default()
            },
            spec: Default::default(),
            status: None,
        }
        .into_response(None);
        assert_eq!(response.id, "12345678-1234-1234-1234-123456789012");
        assert_eq!(response.name, "test-pool");
        assert_eq!(response.namespace, "default");
        assert_eq!(response.active_servers_count, 0);
        assert_eq!(response.pending_servers_count, 0);
        assert_eq!(response.unmanaged_servers_count, 0);
        assert_eq!(response.managed_servers_count, 0);
        assert_eq!(response.total_servers_count, 0);
    }

    #[test]
    fn test_mcp_pool_response_from_pool() {
        let pool = MCPPool {
            metadata: ObjectMeta {
                name: Some("test-pool".to_string()),
                namespace: Some("default".to_string()),
                uid: Some("12345678-1234-1234-1234-123456789012".to_string()),
                ..Default::default()
            },
            spec: Default::default(),
            status: Some(MCPPoolStatus {
                active_servers_count: 5,
                pending_servers_count: 2,
                unmanaged_servers_count: 1,
                managed_servers_count: 7,
                total_servers_count: 8,
            }),
        };

        let servers = vec![
            MCPServer::new("s1", Default::default()),
            MCPServer::new("s2", Default::default()),
            MCPServer::new("s3", Default::default()),
        ];
        let response = pool.into_response(Some(servers));
        assert_eq!(response.id, "12345678-1234-1234-1234-123456789012");
        assert_eq!(response.name, "test-pool");
        assert_eq!(response.namespace, "default");
        assert_eq!(response.max_servers_limit, 100); // Default value
        assert_eq!(response.max_servers_active, 100); // Default value
        assert_eq!(response.default_idle_timeout, 60); // Default value
        assert_eq!(response.active_servers_count, 5);
        assert_eq!(response.pending_servers_count, 2);
        assert_eq!(response.unmanaged_servers_count, 1);
        assert_eq!(response.managed_servers_count, 7);
        assert_eq!(response.total_servers_count, 8);
        assert_eq!(response.url, "/api/v1/pools/test-pool");
        assert_eq!(response.servers.len(), 3);
        assert_eq!(response.servers[0], "/api/v1/servers/s1");
        assert_eq!(response.servers[1], "/api/v1/servers/s2");
        assert_eq!(response.servers[2], "/api/v1/servers/s3");
    }
}
