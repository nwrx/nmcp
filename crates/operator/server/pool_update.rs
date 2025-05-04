use crate::{MCPPool, MCPPoolSpec, MCPPoolStatus};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;

use super::ServerState;

/// Handler for PUT /api/v1/pools/{name}
pub async fn pool_update(
    Path(name): Path<String>,
    State(_state): State<Arc<ServerState>>,
    Json(request): Json<MCPPoolSpec>,
) -> Response {
    // In a real implementation, we would update the pool in K8s
    // For now, just return the pool we would have updated
    let pool = MCPPool {
        metadata: kube::api::ObjectMeta {
            name: Some(name),
            namespace: Some("default".to_string()),
            ..Default::default()
        },
        spec: request,
        status: Some(MCPPoolStatus::default()),
    };
    (StatusCode::OK, Json(pool)).into_response()
}
