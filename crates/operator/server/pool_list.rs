use super::ServerState;
use crate::{MCPPool, MCPPoolSpec, MCPPoolStatus};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use std::sync::Arc;

/// Pool list response
#[derive(Serialize)]
struct PoolListResponse {
    items: Vec<MCPPool>,
}

/// Handler for GET /api/v1/pools
pub async fn pool_list(State(_state): State<Arc<ServerState>>) -> Response {
    // In a real implementation, we would fetch pools from K8s
    // For now, return sample data
    let pools = vec![
        MCPPool {
            metadata: kube::api::ObjectMeta {
                name: Some("large".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: MCPPoolSpec::default(),
            status: Some(MCPPoolStatus::default()),
        },
        MCPPool {
            metadata: kube::api::ObjectMeta {
                name: Some("large".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: MCPPoolSpec::default(),
            status: Some(MCPPoolStatus::default()),
        },
    ];
    let response = PoolListResponse { items: pools };
    (StatusCode::OK, Json(response)).into_response()
}
