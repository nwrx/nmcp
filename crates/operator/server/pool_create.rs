use crate::{MCPPool, MCPPoolSpec, ServerState};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;

/// Handler for POST /api/v1/pools
pub async fn pool_create(
    State(_state): State<Arc<ServerState>>,
    Json(spec): Json<MCPPoolSpec>,
) -> Response {
    let pool = MCPPool {
        metadata: kube::api::ObjectMeta {
            name: Some("new-pool".to_string()),
            namespace: Some("default".to_string()),
            ..Default::default()
        },
        spec,
        status: None,
    };
    (StatusCode::CREATED, Json(pool)).into_response()
}
