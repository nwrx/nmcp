use super::ServerState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;

/// Handler for the `/health` endpoint.
pub async fn health_get(State(_state): State<Arc<ServerState>>) -> Response {
    let health_data = json!({
        "status": "UP",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "api": "UP",
            "operator": "UP",
        }
    });
    (StatusCode::OK, Json(health_data)).into_response()
}
