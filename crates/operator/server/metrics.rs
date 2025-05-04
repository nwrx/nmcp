use super::ServerState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;

/// Handler for the /api/v1/metrics endpoint
pub async fn metrics_get(State(_state): State<Arc<ServerState>>) -> Response {
    let metrics_data = json!({
        "operator": {
            "pools": 2,
            "servers": 5,
            "reconcileOperations": 42,
            "averageReconcileTime": 0.23
        },
        "api": {
            "requestCount": 120,
            "errorCount": 2,
            "averageResponseTime": 0.05
        }
    });
    (StatusCode::OK, Json(metrics_data)).into_response()
}
