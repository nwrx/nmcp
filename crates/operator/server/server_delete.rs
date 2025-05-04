use super::ServerState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;

/// Handler for DELETE /api/v1/servers/{uid}
pub async fn server_delete(
    Path(uid): Path<String>,
    State(_state): State<Arc<ServerState>>,
) -> Response {
    // In a real implementation, we would delete the server from K8s
    // For now, just return a success response
    (
        StatusCode::OK,
        Json(json!({ "message": format!("Server {} deleted", uid) })),
    )
        .into_response()
}
