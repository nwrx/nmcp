use super::{health_docs, ManagerContext};
use crate::SystemStatus;
use aide::axum::routing::get_with;
use aide::axum::ApiRouter;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::time::SystemTime;

// Store the application start time
static APP_START_TIME: LazyLock<SystemTime> = LazyLock::new(SystemTime::now);

/// Represents the health status of the manager service.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ManagerStatus {
    /// Indicates if the system is operational.
    pub ok: bool,

    /// The version of the application.
    pub version: String,

    /// The uptime of the application in seconds.
    pub uptime: u64,

    /// Optional system information.
    pub system: SystemStatus,
}

impl Default for ManagerStatus {
    fn default() -> Self {
        Self {
            ok: true,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime: APP_START_TIME.elapsed().unwrap_or_default().as_secs(),
            system: SystemStatus::default(),
        }
    }
}

/// Handler for the `/health/status` endpoint.
pub async fn status(State(_): State<ManagerContext>) -> Response {
    let status = ManagerStatus::default();
    (StatusCode::OK, Json(status)).into_response()
}

/// Handler for the `/health/ping` endpoint.
pub async fn ping(State(_): State<ManagerContext>) -> Response {
    (StatusCode::NO_CONTENT, ()).into_response()
}

/// Route for health checks.
pub fn router(ctx: ManagerContext) -> ApiRouter<()> {
    ApiRouter::new()
        .api_route("/status", get_with(status, health_docs::status_docs))
        .api_route("/ping", get_with(ping, health_docs::ping_docs))
        .with_state(ctx)
}
