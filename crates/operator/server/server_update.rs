use super::ServerState;
use crate::MCPServerSpec;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;

/// Handler for PUT /api/v1/servers/{name}
pub async fn server_update(
    Path(name): Path<String>,
    State(state): State<Arc<ServerState>>,
    Json(spec): Json<MCPServerSpec>,
) -> Response {
    match state.controller().patch_server_spec(&name, &spec).await {
        Ok(updated_server) => {
            tracing::info!("Server updated successfully: {:?}", updated_server);
            (StatusCode::OK, Json(updated_server)).into_response()
        }
        Err(error) => {
            tracing::error!("Failed to update server: {}", error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error.to_string())).into_response()
        }
    }
}
