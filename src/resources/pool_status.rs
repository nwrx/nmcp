use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Status of the `MCPPool` custom resource
#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct MCPPoolStatus {
    /// Number of servers currently in use (active) in the pool. Meaning
    /// that the server is running and has a pod and service created.
    pub active_servers_count: u32,

    /// Number of servers waiting to be created in the pool. Meaning
    /// that the server is waiting for a pod and service to be created.
    pub pending_servers_count: u32,

    /// Number of servers that are currently unmanaged by the pool. Meaning
    /// that the they overflow the `max_servers_limit` and are not being managed
    /// by the `MCPPool` controller.
    pub unmanaged_servers_count: u32,

    /// Number of servers that are currently managed by the `MCPPool` controller.
    /// Meaning that the server that do not overflow the `max_servers_limit`
    /// and are being managed by the `MCPPool` controller.
    pub managed_servers_count: u32,

    /// Total number of servers in the pool. This is the sum of all servers
    /// that are currently in use, waiting, ignored and managed by the `MCPPool`
    /// controller.
    pub total_servers_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_pool_status_default() {
        let status = MCPPoolStatus::default();
        assert_eq!(status.active_servers_count, 0);
        assert_eq!(status.pending_servers_count, 0);
        assert_eq!(status.unmanaged_servers_count, 0);
        assert_eq!(status.managed_servers_count, 0);
        assert_eq!(status.total_servers_count, 0);
    }

    #[test]
    fn test_mcp_pool_status_serialization() {
        let status = MCPPoolStatus {
            active_servers_count: 5,
            pending_servers_count: 2,
            unmanaged_servers_count: 1,
            managed_servers_count: 7,
            total_servers_count: 8,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"activeServersCount\":5"));
        assert!(json.contains("\"pendingServersCount\":2"));
        assert!(json.contains("\"unmanagedServersCount\":1"));
        assert!(json.contains("\"managedServersCount\":7"));
        assert!(json.contains("\"totalServersCount\":8"));
    }

    #[test]
    fn test_mcp_pool_status_deserialization() {
        let json = r#"
        {
            "activeServersCount": 5,
            "pendingServersCount": 2,
            "unmanagedServersCount": 1,
            "managedServersCount": 7,
            "totalServersCount": 8
        }
        "#;

        let status: MCPPoolStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.active_servers_count, 5);
        assert_eq!(status.pending_servers_count, 2);
        assert_eq!(status.unmanaged_servers_count, 1);
        assert_eq!(status.managed_servers_count, 7);
        assert_eq!(status.total_servers_count, 8);
    }
}
