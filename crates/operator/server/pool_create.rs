use crate::{MCPPoolSpec, ServerState};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

/// Request body for creating a new pool
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePoolRequest {
    /// Name of the pool to be created. This is a required field
    /// and should be unique within the namespace.
    pub name: String,

    // Include all fields from MCPPoolSpec directly
    #[serde(flatten)]
    pub spec: MCPPoolSpec,
}

/// Handler for POST /api/v1/pools
pub async fn pool_create(
    State(state): State<Arc<ServerState>>,
    Json(body): Json<CreatePoolRequest>,
) -> Response {
    match state.controller().create_pool(&body.name, body.spec).await {
        Ok(created_pool) => (StatusCode::CREATED, Json(created_pool)).into_response(),
        Err(error) => {
            tracing::error!("Failed to create pool: {}", error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error.to_string())).into_response()
        }
    }
}
