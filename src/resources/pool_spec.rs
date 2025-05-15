use crate::MCPPoolStatus;
use k8s_openapi::api::core::v1;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// McpPool custom resource definition
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq)]
#[kube(
    group = "nmcp.nwrx.io",
    version = "v1",
    kind = "MCPPool",
    singular = "mcppool",
    plural = "mcppools",
    shortname = "mcpp",
    namespaced,
    status = "MCPPoolStatus",
    printcolumn = r#"{"name":"In Use", "type":"integer", "jsonPath":".status.serverInUse"}"#,
    printcolumn = r#"{"name":"Waiting", "type":"integer", "jsonPath":".status.serverWaiting"}"#,
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct MCPPoolSpec {
    /// Maximum amount of MCPServer resources that can be managed by this MCPPool. After
    /// this limit is reached, the overflow servers will be marked as "ignored" and no Pod
    /// or Service resources will be created for them until older MCPServer resources are
    /// deleted.
    ///
    /// TODO: Deprecated in favor of `maxActiveServers`.
    #[serde(default = "default_max_servers")]
    pub max_servers_limit: u32,

    /// The maxcimum number of concurrent active servers that can be created in the pool.
    /// After this limit is reached, the overflow servers will be marked as "waiting" and
    /// no Pod or Service resources will be created for them until Pod and Service resources
    /// are deleted by the operator.
    #[serde(default = "default_max_servers")]
    pub max_servers_active: u32,

    /// The default resource requirements for each server in the pool. This will be used to
    /// determine the resource limits and requests for each server's pod. This is to
    /// ensure that each server has the necessary resources to run efficiently and
    /// effectively. This is also to prevent the pool from overwhelming the system with
    /// too many servers at once.
    #[serde(default)]
    pub default_resources: v1::ResourceRequirements,

    /// The default time in seconds that a server is allowed to run without receiving
    /// any requests before it's terminated. This helps to conserve resources by
    /// shutting down idle servers.
    #[serde(default = "default_idle_timeout")]
    pub default_idle_timeout: u32,
}

/// Default maximum servers
fn default_max_servers() -> u32 {
    100
}

/// Default idle timeout in seconds
fn default_idle_timeout() -> u32 {
    60 // 1 minutes
}

impl Default for MCPPoolSpec {
    fn default() -> Self {
        Self {
            max_servers_limit: default_max_servers(),
            max_servers_active: default_max_servers(),
            default_resources: v1::ResourceRequirements::default(),
            default_idle_timeout: default_idle_timeout(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kube::CustomResourceExt;

    #[test]
    fn test_mcp_pool_crd() {
        let crd = MCPPool::crd();
        assert_eq!(crd.spec.names.kind, "MCPPool");
        assert_eq!(crd.spec.names.plural, "mcppools");
        assert_eq!(crd.spec.names.singular, Some("mcppool".to_string()));
        assert_eq!(crd.spec.group, "nmcp.nwrx.io");
        assert_eq!(crd.spec.versions[0].name, "v1");
    }

    #[test]
    fn test_mcp_pool_spec_defaults() {
        let spec = MCPPoolSpec::default();
        assert_eq!(spec.max_servers_limit, 100);
        assert_eq!(spec.max_servers_active, 100);
        assert!(spec.default_resources.limits.is_none());
        assert!(spec.default_resources.requests.is_none());
        assert_eq!(spec.default_idle_timeout, 60);
    }

    #[test]
    fn test_mcp_pool_json_deserialization() {
        let json = r#"
        {
            "apiVersion": "nmcp.nwrx.io/v1",
            "kind": "MCPPool",
            "metadata": {
                "name": "test-pool",
                "namespace": "default"
            },
            "spec": {
                "maxServersLimit": 200,
                "maxServersActive": 150,
                "defaultResources": {
                    "limits": {
                        "cpu": "500m",
                        "memory": "512Mi"
                    },
                    "requests": {
                        "cpu": "100m",
                        "memory": "256Mi"
                    }
                },
                "defaultIdleTimeout": 120
            }
        }
        "#;

        let pool: MCPPool = serde_json::from_str(json).unwrap();
        assert_eq!(pool.spec.max_servers_limit, 200);
        assert_eq!(pool.spec.max_servers_active, 150);

        let limits = pool.spec.default_resources.limits.as_ref().unwrap();
        assert_eq!(limits.get("cpu").unwrap().0, "500m");
        assert_eq!(limits.get("memory").unwrap().0, "512Mi");

        let requests = pool.spec.default_resources.requests.as_ref().unwrap();
        assert_eq!(requests.get("cpu").unwrap().0, "100m");
        assert_eq!(requests.get("memory").unwrap().0, "256Mi");

        assert_eq!(pool.spec.default_idle_timeout, 120);
        assert_eq!(pool.metadata.name, Some("test-pool".to_string()));
        assert_eq!(pool.metadata.namespace, Some("default".to_string()));
    }

    #[test]
    fn test_mcp_pool_json_serialization() {
        use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
        use std::collections::BTreeMap;

        let mut limits = BTreeMap::new();
        limits.insert("cpu".to_string(), Quantity("500m".to_string()));
        limits.insert("memory".to_string(), Quantity("512Mi".to_string()));

        let mut requests = BTreeMap::new();
        requests.insert("cpu".to_string(), Quantity("100m".to_string()));
        requests.insert("memory".to_string(), Quantity("256Mi".to_string()));

        let pool = MCPPool {
            metadata: kube::core::ObjectMeta {
                name: Some("test-pool".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: MCPPoolSpec {
                max_servers_limit: 200,
                max_servers_active: 150,
                default_resources: v1::ResourceRequirements {
                    limits: Some(limits),
                    requests: Some(requests),
                    claims: None,
                },
                default_idle_timeout: 120,
            },
            status: None,
        };

        let json = serde_json::to_string(&pool).unwrap();
        assert!(json.contains("\"name\":\"test-pool\""));
        assert!(json.contains("\"namespace\":\"default\""));
        assert!(json.contains("\"maxServersLimit\":200"));
        assert!(json.contains("\"maxServersActive\":150"));
        assert!(json.contains("\"limits\":{\"cpu\":\"500m\",\"memory\":\"512Mi\"}"));
        assert!(json.contains("\"requests\":{\"cpu\":\"100m\",\"memory\":\"256Mi\"}"));
        assert!(json.contains("\"defaultIdleTimeout\":120"));
    }
}
