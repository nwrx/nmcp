use super::ServerState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;

/// Handler for GET /api/v1/servers/{name}
pub async fn server_get(
    Path(name): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Response {
    match state.controller().get_server(&name).await {
        Ok(server) => {
            let server = server.into_response();
            (StatusCode::OK, Json(server)).into_response()
        }
        Err(error) => {
            tracing::error!("Failed to get server: {}", error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error.to_string())).into_response()
        }
    }
}
