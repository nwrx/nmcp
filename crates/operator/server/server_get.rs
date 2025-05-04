use super::ServerState;
use crate::{MCPServer, MCPServerSpec, MCPServerStatus};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;

/// Handler for GET /api/v1/servers/{uid}
pub async fn server_get(
    Path(uid): Path<String>,
    State(_state): State<Arc<ServerState>>,
) -> Response {
    // In a real implementation, we would fetch the server from K8s
    // For this example, return data for a specific UID
    if uid == "123e4567-e89b-12d3-a456-426614174000" {
        let server = MCPServer {
            metadata: kube::api::ObjectMeta {
                name: Some("time-service".to_string()),
                namespace: Some("default".to_string()),
                uid: Some(uid),
                ..Default::default()
            },
            spec: MCPServerSpec {
                pool: "default".to_string(),
                ..Default::default()
            },
            status: Some(MCPServerStatus {
                is_running: true,
                is_idle: false,
                total_requests: 42,
                current_connections: 3,
                last_request_at: Some(Utc::now()),
                ..Default::default()
            }),
        };

        (StatusCode::OK, Json(server)).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Server {} not found", uid) })),
        )
            .into_response()
    }
}
