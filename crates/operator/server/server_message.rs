use crate::MCPServer;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rmcp::model::JsonRpcRequest;
use std::sync::Arc;
use tracing::info;

use super::ServerState;

/// Handler for POST /api/v1/servers/{uid}/messages
pub async fn server_message(
    Path(uid): Path<String>,
    State(state): State<Arc<ServerState>>,
    Json(request): Json<JsonRpcRequest>,
) -> Response {
    // --- First, get the existing server
    let server: MCPServer = match state.controller().get_server(&uid).await {
        Ok(server) => server,
        Err(_error) => {
            return (StatusCode::NOT_FOUND, format!("Server '{uid}' not found")).into_response()
        }
    };

    // --- Send the input to the server's stdin
    let json_str = serde_json::to_string(&request).unwrap();
    info!("Sending input to server: {}", json_str);
    match state
        .controller()
        .send_to_server_stdin(&server, json_str)
        .await
    {
        Ok(_) => (StatusCode::OK, "Input sent to server").into_response(),
        Err(error) => {
            tracing::error!("Failed to send input to server: {}", error);
            (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
        }
    }
}
