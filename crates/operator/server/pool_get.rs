use super::ServerState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;

/// Handler for GET /api/v1/pools/{name}
pub async fn pool_get(Path(name): Path<String>, State(state): State<Arc<ServerState>>) -> Response {
    let servers = state
        .controller()
        .list_servers()
        .await
        .unwrap()
        .into_iter()
        .filter(|server| server.spec.pool == name)
        .collect::<Vec<_>>();

    match state.controller().get_pool(&name).await {
        Ok(pool) => (StatusCode::OK, Json(pool.into_response(Some(servers)))).into_response(),
        Err(error) => (StatusCode::NOT_FOUND, Json(error.to_string())).into_response(),
    }
}
