use super::ServerState;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response, Sse};
use futures::stream;
use std::convert::Infallible;
use std::sync::Arc;

/// Handler for GET /api/v1/servers/{uid}/sse
pub async fn server_sse(
    Path(uid): Path<String>,
    State(_state): State<Arc<ServerState>>,
) -> Response {
    // In a real implementation, we would connect to the server's output stream
    // For this example, just generate some sample events
    // Create a sample stream of events
    let stream = stream::iter(vec![
        Ok::<_, Infallible>(
            axum::response::sse::Event::default()
                .event("connected")
                .data(format!("Connected to server {uid}")),
        ),
        Ok::<_, Infallible>(
            axum::response::sse::Event::default()
                .event("message")
                .data("This is a sample message from the server"),
        ),
        Ok::<_, Infallible>(
            axum::response::sse::Event::default()
                .event("message")
                .data("Another sample message from the server"),
        ),
    ]);
    Sse::new(stream).into_response()
}
