use chrono::{DateTime, Utc};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Condition;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// `MCPServerPhase` represents the current lifecycle phase of the server
#[derive(Debug, Copy, Clone, Default, Deserialize, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum MCPServerPhase {
    /// Server is not running and has no traffic or resources allocated.
    /// This is the initial state of the server when it is created.
    #[default]
    Idle,

    /// The server has been requested to be started but is not yet running
    /// (e.g., waiting for resources to be created).
    Requested,

    /// Server is starting up and not yet ready to process requests
    /// (e.g., waiting for resources to be created or initialized).
    Starting,

    /// Server is currently running and processing requests. Meaning it's
    /// Pod and Service are up and running.
    Ready,

    /// Server is shutting down and not processing requests
    /// (e.g., waiting for resources to be deleted or cleaned up).
    Stopping,

    /// Server is in an error state and not processing requests
    /// (e.g., due to a failure in the server or its resources).
    Degraded,
}

/// `MCPServer` status
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MCPServerStatus {
    /// Current phase of the server lifecycle
    pub phase: MCPServerPhase,

    /// Conditions observed on the server, following Kubernetes conditions pattern
    #[serde(default)]
    pub conditions: Vec<Condition>,

    /// The last time the server was started
    #[serde(default)]
    pub created_at: DateTime<Utc>,

    /// The last time the server was requested to start
    pub requested_at: Option<DateTime<Utc>>,

    /// The last time the server was stopped
    pub stopped_at: Option<DateTime<Utc>>,

    /// The last time the server was started
    pub started_at: Option<DateTime<Utc>>,

    /// Time of the last received request
    pub last_request_at: Option<DateTime<Utc>>,

    /// Total number of requests processed by the server
    pub total_requests: u32,

    /// Number of current connections to the server
    pub current_connections: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_mcp_server_status_default() {
        let status = MCPServerStatus::default();
        assert_eq!(status.started_at, None);
        assert_eq!(status.stopped_at, None);
        assert_eq!(status.last_request_at, None);
        assert_eq!(status.phase, MCPServerPhase::Idle);
        assert!(status.conditions.is_empty());
        assert_eq!(status.total_requests, 0);
        assert_eq!(status.current_connections, 0);
    }

    #[test]
    fn test_mcp_server_status_serialization() {
        let created_at = Utc.with_ymd_and_hms(2025, 5, 1, 10, 0, 0).unwrap();
        let started_at = Utc.with_ymd_and_hms(2025, 5, 1, 10, 0, 0).unwrap();
        let stopped_at = Utc.with_ymd_and_hms(2025, 5, 1, 11, 0, 0).unwrap();
        let last_request_at = Utc.with_ymd_and_hms(2025, 5, 1, 10, 30, 0).unwrap();

        let status = MCPServerStatus {
            created_at,
            started_at: Some(started_at),
            stopped_at: Some(stopped_at),
            requested_at: None,
            last_request_at: Some(last_request_at),
            phase: MCPServerPhase::Ready,
            conditions: vec![],
            total_requests: 10,
            current_connections: 2,
        };

        let json = serde_json::to_string(&status).unwrap();

        // Deserialize and verify the object matches the original
        let deserialized: MCPServerStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.created_at, status.created_at);
        assert_eq!(deserialized.started_at, status.started_at);
        assert_eq!(deserialized.stopped_at, status.stopped_at);
        assert_eq!(deserialized.last_request_at, status.last_request_at);
        assert_eq!(deserialized.phase, status.phase);
        assert_eq!(deserialized.conditions, status.conditions);
        assert_eq!(deserialized.total_requests, status.total_requests);
        assert_eq!(deserialized.current_connections, status.current_connections);
    }

    #[test]
    fn test_mcp_server_status_deserialization() {
        let started_at = Utc.with_ymd_and_hms(2025, 5, 1, 10, 0, 0).unwrap();
        let stopped_at = Utc.with_ymd_and_hms(2025, 5, 1, 11, 0, 0).unwrap();
        let last_request_at = Utc.with_ymd_and_hms(2025, 5, 1, 10, 30, 0).unwrap();

        let json = format!(
            r#"
        {{
            "startedAt": "{}",
            "stoppedAt": "{}",
            "lastRequestAt": "{}",
            "phase": "Running",
            "conditions": [],
            "totalRequests": 10,
            "currentConnections": 2
        }}
        "#,
            started_at.to_rfc3339(),
            stopped_at.to_rfc3339(),
            last_request_at.to_rfc3339()
        );

        let status: MCPServerStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status.started_at.unwrap(), started_at);
        assert_eq!(status.stopped_at.unwrap(), stopped_at);
        assert_eq!(status.last_request_at.unwrap(), last_request_at);
        assert_eq!(status.phase, MCPServerPhase::Ready);
        assert!(status.conditions.is_empty());
        assert_eq!(status.total_requests, 10);
        assert_eq!(status.current_connections, 2);
    }
}
