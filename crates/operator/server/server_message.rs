use super::ServerState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rmcp::model::JsonRpcRequest;
use std::sync::Arc;

/// Handler for POST /api/v1/servers/{name}/message
pub async fn server_message(
    Path(name): Path<String>,
    State(state): State<Arc<ServerState>>,
    Json(request): Json<JsonRpcRequest>,
) -> Response {
    let controller = state.controller();

    // --- Get server by name from the controller
    let server = match controller.get_server_by_name(&name).await {
        Ok(server) => server,
        Err(_error) => {
            return (StatusCode::NOT_FOUND, format!("Server '{name}' not found")).into_response()
        }
    };

    // --- Get server SSE channels for communication
    let channels = match controller.get_server_sse(&server).await {
        Ok(channels) => channels,
        Err(error) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
        }
    };

    // --- Send the request and handle response
    let channels = channels.read().await;
    match channels.send(request).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}
