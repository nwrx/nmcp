use super::ServerState;
use crate::MCPServer;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::Event;
use axum::response::{IntoResponse, Response, Sse};
use futures::StreamExt;
use std::convert::Infallible;
use std::sync::Arc;

/// Handler for GET /api/v1/servers/{uid}/sse
pub async fn server_sse(
    Path(uid): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Response {
    // --- First, get the existing server.
    let server: MCPServer = match state.controller().get_server(&uid).await {
        Ok(server) => server,
        Err(_error) => {
            return (StatusCode::NOT_FOUND, format!("Server '{uid}' not found")).into_response()
        }
    };

    // --- Then, we get it's associated Service and Pod.
    // match state.controller().get_server_service(&server).await {
    //     Ok(_) => (),
    //     Err(error) => {
    //         return (StatusCode::NOT_FOUND, format!("Service not found: {error}")).into_response()
    //     }
    // }
    // match state.controller().get_server_pod(&server).await {
    //     Ok(_) => (),
    //     Err(error) => {
    //         return (StatusCode::NOT_FOUND, format!("Pod not found: {error}")).into_response()
    //     }
    // }

    // --- Now proxy the TTY stream to SSE
    let event_stream = match state.controller().get_server_stream(&server).await {
        Ok(stream) => stream,
        Err(error) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
        }
    };

    // Serialize the stream of events into SSE format.
    let sse_stream = event_stream.map(|event| {
        let event = event.to_string();
        Ok::<_, Infallible>(Event::default().data(event))
    });

    // Return the SSE response
    Sse::new(sse_stream).into_response()
}
