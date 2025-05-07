use super::ServerState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;

/// Handler for DELETE /api/v1/servers/{name}
pub async fn server_delete(
    Path(name): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Response {
    match state.controller().delete_server(&name).await {
        Ok(_) => {
            tracing::info!("Server deleted successfully: {}", name);
            (StatusCode::NO_CONTENT, Json(json!({}))).into_response()
        }
        Err(error) => {
            tracing::error!("Failed to delete server: {}", error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error.to_string())).into_response()
        }
    }
}
