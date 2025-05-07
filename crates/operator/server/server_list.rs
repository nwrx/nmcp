use super::ServerState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

/// Query parameters for server list endpoint
#[derive(Deserialize)]
pub struct ServerListQuery {
    pool: Option<String>,
}

/// Handler for GET /api/v1/servers
pub async fn server_list(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<ServerListQuery>,
) -> Response {
    match state.controller().list_servers().await {
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch servers").into_response(),
        Ok(servers) => {
            let response = if let Some(pool_name) = query.pool {
                servers
                    .into_iter()
                    .filter(|s| s.spec.pool == pool_name)
                    .map(|s| s.into_response())
                    .collect::<Vec<_>>()
            } else {
                servers
                    .into_iter()
                    .map(|s| s.into_response())
                    .collect::<Vec<_>>()
            };
            (StatusCode::OK, Json(response)).into_response()
        }
    }
}
