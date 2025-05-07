use super::ServerState;
use crate::MCPServerSpec;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

/// Request body for creating a new pool
#[derive(Debug, Clone, Deserialize)]
pub struct CreateServerRequest {
    /// Name of the pool to be created. This is a required field
    /// and should be unique within the namespace.
    pub name: String,

    // Include all fields from MCPServerSpec directly
    #[serde(flatten)]
    pub spec: MCPServerSpec,
}

/// Handler for POST /api/v1/servers
pub async fn server_create(
    State(state): State<Arc<ServerState>>,
    Json(body): Json<CreateServerRequest>,
) -> Response {
    match state
        .controller()
        .create_server(&body.name, body.spec)
        .await
    {
        Ok(created_server) => (StatusCode::CREATED, Json(created_server)).into_response(),
        Err(error) => {
            tracing::error!("Failed to create server: {}", error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error.to_string())).into_response()
        }
    }
}
