use super::GatewayContext;
use aide::axum::routing::get;
use aide::axum::ApiRouter;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Status {
    pub ok: bool,
    pub version: String,
    pub timestamp: String,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            ok: true,
            version: "0.0.1".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Handler for the `/health/status` endpoint.
pub async fn status(State(_): State<GatewayContext>) -> Response {
    let status = Status::default();
    (StatusCode::OK, Json(status)).into_response()
}

/// Handler for the `/health/ping` endpoint.
pub async fn ping(State(_): State<GatewayContext>) -> Response {
    (StatusCode::OK, ()).into_response()
}

/// Route for health checks.
pub fn router(ctx: GatewayContext) -> ApiRouter<()> {
    ApiRouter::new()
        .api_route("/status", get(status))
        .api_route("/ping", get(ping))
        .with_state(ctx)
}
