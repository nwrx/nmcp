use super::ServerState;
use crate::{MCPPool, MCPPoolSpec, MCPPoolStatus};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use kube::api::ObjectMeta;
use serde_json::json;
use std::sync::Arc;

/// Handler for GET /api/v1/pools/{name}
pub async fn pool_get(
    Path(name): Path<String>,
    State(_state): State<Arc<ServerState>>,
) -> Response {
    // In a real implementation, we would fetch the pool from K8s
    // For now, return sample data for the "default" pool
    if name == "default" {
        let pool = MCPPool {
            metadata: ObjectMeta::default(),
            spec: MCPPoolSpec::default(),
            status: Option::<MCPPoolStatus>::default(),
        };
        (StatusCode::OK, Json(pool)).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Pool {} not found", name) })),
        )
            .into_response()
    }
}
