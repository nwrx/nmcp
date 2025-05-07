use super::ServerState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;

/// Handler for DELETE /api/v1/pools/{name}
pub async fn pool_delete(
    Path(name): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Response {
    match state.controller().delete_pool(&name).await {
        Ok(_) => {
            tracing::info!("Pool deleted successfully: {}", name);
            (StatusCode::NO_CONTENT, Json(())).into_response()
        }
        Err(error) => {
            tracing::error!("Failed to delete pool: {}", error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error.to_string())).into_response()
        }
    }
}
