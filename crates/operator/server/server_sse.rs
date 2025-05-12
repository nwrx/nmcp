use super::ServerState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response, Sse};
use std::sync::Arc;
use tokio_stream::StreamExt;

/// Handler for GET /api/v1/servers/{name}/sse
pub async fn server_sse(
    Path(name): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Response {
    let controller = state.controller();

    // --- Get server by name from the controller.
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
    let endpoint = server.url_messages();
    let channels = channels.read().await;

    let _ = channels.listen().await;
    let stream = channels.subscribe(endpoint).await.map(|data| match data {
        Ok(event) => Ok(event.into()),
        Err(e) => Err(axum::Error::new(e)),
    });

    // --- Check if the stream is empty
    let mut response = Sse::new(stream).into_response();
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::header::HeaderValue::from_static("text/event-stream"),
    );
    response
}
