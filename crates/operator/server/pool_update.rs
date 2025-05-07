use crate::MCPPoolSpec;
use axum::extract::{Path, State};
use axum::response::Response;
use axum::Json;
use std::sync::Arc;

use super::ServerState;

/// Handler for PUT /api/v1/pools/{name}
pub async fn pool_update(
    Path(_name): Path<String>,
    State(_state): State<Arc<ServerState>>,
    Json(_spec): Json<MCPPoolSpec>,
) -> Response {
    todo!();
}
