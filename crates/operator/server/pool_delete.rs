use super::ServerState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;

/// Handler for DELETE /api/v1/pools/{name}
pub async fn pool_delete(
    Path(name): Path<String>,
    State(_state): State<Arc<ServerState>>,
) -> Response {
    // In a real implementation, we would delete the pool from K8s
    // For now, just return a success response
    (
        StatusCode::OK,
        Json(json!({ "message": format!("Pool {} deleted", name) })),
    )
        .into_response()
}
