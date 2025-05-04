use crate::{MCPServer, MCPServerSpec};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use kube::api::ObjectMeta;
use std::sync::Arc;

use super::ServerState;

/// Handler for POST /api/v1/servers
pub async fn server_create(
    State(_state): State<Arc<ServerState>>,
    Json(spec): Json<MCPServerSpec>,
) -> Response {
    // In a real implementation, we would create the server in K8s
    // For now, just return a server we would have created
    let server = MCPServer {
        metadata: ObjectMeta {
            name: Some(format!("new-server-{}", uuid::Uuid::new_v4())),
            namespace: Some("default".to_string()),
            uid: Some(uuid::Uuid::new_v4().to_string()),
            ..Default::default()
        },
        spec,
        status: None,
    };

    (StatusCode::CREATED, Json(server)).into_response()
}
