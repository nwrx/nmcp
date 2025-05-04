use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Status of the MCPPool custom resource
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct MCPPoolStatus {
    /// Number of servers currently in use (active) in the pool. Meaning
    /// that the server is running and has a pod and service created.
    pub active_servers_count: i32,

    /// Number of servers waiting to be created in the pool. Meaning
    /// that the server is waiting for a pod and service to be created.
    pub pending_servers_count: i32,

    /// Number of servers that are currently unmanaged by the pool. Meaning
    /// that the they overflow the max_servers_limit and are not being managed
    /// by the MCPPool controller.
    pub unmanaged_servers_count: i32,

    /// Number of servers that are currently managed by the MCPPool controller.
    /// Meaning that the server that do not overflow the max_servers_limit
    /// and are being managed by the MCPPool controller.
    pub managed_servers_count: i32,

    /// Total number of servers in the pool. This is the sum of all servers
    /// that are currently in use, waiting, ignored and managed by the MCPPool
    /// controller.
    pub total_servers_count: i32,
}
