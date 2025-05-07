use super::ServerState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;

/// Handler for GET /api/v1/pools
pub async fn pool_list(State(state): State<Arc<ServerState>>) -> Response {
    match state.controller().list_pools().await {
        Ok(pools) => {
            let pools = pools
                .into_iter()
                .map(|pool| pool.into_response(None))
                .collect::<Vec<_>>();
            (StatusCode::OK, Json(pools)).into_response()
        }
        Err(error) => {
            tracing::error!("Failed to list pools: {}", error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error.to_string())).into_response()
        }
    }
}
