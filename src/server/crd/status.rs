use schemars::JsonSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// MCPServer status
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MCPServerStatus {
    pub start_time: Option<DateTime<Utc>>,
    pub server_endpoint: Option<String>,
    pub server_uuid: Option<String>,
    pub metrics: Option<MCPServerMetrics>,
}

/// Server metrics
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MCPServerMetrics {
    pub request_count: Option<i64>,
    pub active_connections: Option<i32>,
    pub cpu_usage: Option<String>,
    pub memory_usage: Option<String>,
}
