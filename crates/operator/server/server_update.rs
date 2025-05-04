use super::ServerState;
use crate::{MCPServer, MCPServerSpec, MCPServerStatus};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;

/// Handler for PUT /api/v1/servers/{uid}
pub async fn server_update(
    Path(uid): Path<String>,
    State(_state): State<Arc<ServerState>>,
    Json(spec): Json<MCPServerSpec>,
) -> Response {
    let server = MCPServer {
        metadata: kube::api::ObjectMeta {
            name: Some("existing-server".to_string()),
            namespace: Some("default".to_string()),
            uid: Some(uid),
            ..Default::default()
        },
        spec,
        status: Some(MCPServerStatus {
            is_running: true,
            is_idle: false,
            total_requests: 42,
            current_connections: 3,
            ..Default::default()
        }),
    };
    (StatusCode::OK, Json(server)).into_response()
}
