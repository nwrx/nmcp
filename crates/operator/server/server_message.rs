use super::ServerState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

/// Message to send to a server
#[derive(Deserialize)]
pub struct ServerMessage {
    pub content: String,
}

/// Handler for POST /api/v1/servers/{uid}/messages
pub async fn server_message(
    Path(uid): Path<String>,
    State(_state): State<Arc<ServerState>>,
    Json(message): Json<ServerMessage>,
) -> Response {
    // In a real implementation, we would send the message to the server's input stream
    // For now, just return a success response
    (
        StatusCode::OK,
        Json(json!({
            "server": uid,
            "status": "sent",
            "message": message.content,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
    )
        .into_response()
}
