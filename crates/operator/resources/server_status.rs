use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// MCPServer status
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MCPServerStatus {
    /// Time when the server was created
    pub created_at: DateTime<Utc>,

    /// Time when the server was started
    pub started_at: Option<DateTime<Utc>>,

    /// Time when the server was stopped
    pub stopped_at: Option<DateTime<Utc>>,

    /// Time of the last received request
    pub last_request_at: Option<DateTime<Utc>>,

    /// Is the server currently running
    pub is_running: bool,

    /// Is the server currently idle
    pub is_idle: bool,

    /// Total number of requests processed by the server
    pub total_requests: u32,

    /// Number of current connections to the server
    pub current_connections: u32,
}
