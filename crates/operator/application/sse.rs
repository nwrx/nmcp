use super::ApplicationContext;
use aide::axum::IntoApiResponse;
use aide::transform::TransformOperation;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Sse};
use axum::Json;
use rmcp::model::{JsonRpcRequest, JsonRpcResponse, ServerJsonRpcMessage};
use tokio_stream::StreamExt;

/// Handler for GET /api/v1/servers/{name}/sse
pub async fn stream(
    Path(name): Path<String>,
    State(ctx): State<ApplicationContext>,
) -> impl IntoApiResponse {
    let controller = ctx.controller().await;
    tracing::debug!("Received request: {:?}", name);

    // --- Get server by name from the controller.
    let server = match controller.get_server_by_name(&name).await {
        Ok(server) => server,
        Err(error) => {
            tracing::error!("Failed to get server by name: {}", error);
            return error.into_response();
        }
    };

    match controller.register_server_request(&server).await {
        Ok(_) => {}
        Err(error) => {
            tracing::error!("Failed to register server request: {}", error);
            return error.into_response();
        }
    }

    // --- Request the server to start and wait until it is ready.
    match controller.request_server_up(&server).await {
        Ok(_) => {}
        Err(error) => {
            tracing::error!("Failed to register server request: {}", error);
            return error.into_response();
        }
    }

    // --- Get server SSE channels for communication
    let channels = match controller.get_server_transport(&server).await {
        Ok(channels) => channels,
        Err(error) => {
            tracing::error!("Failed to get server transport: {}", error);
            return error.into_response();
        }
    };

    // --- Send the request and handle response
    let endpoint = format!("/api/v1/servers/{name}/message");
    let channels = channels.read().await;

    let _ = channels.listen().await;
    let stream = channels.subscribe(endpoint).await.map(|data| match data {
        Ok(event) => Ok(event.into()),
        Err(e) => Err(axum::Error::new(e)),
    });

    // --- Check if the stream is empty
    Sse::new(stream).into_response()
}

/// Documentation for the GET /api/v1/servers/{name}/sse endpoint
pub fn stream_docs(op: TransformOperation) -> TransformOperation {
    op.id("getServerSse")
        .tag("Server")
        .summary("Server SSE")
        .description("Establishes a Server-Sent Events (SSE) connection to the server. This allows for real-time updates and notifications from the server.")
        .response::<200, Json<ServerJsonRpcMessage>>()
}

///////////////////////////////////////////////////////////////////////////////

/// Handler for POST /api/v1/servers/{name}/message
pub async fn message(
    State(ctx): State<ApplicationContext>,
    Path(name): Path<String>,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoApiResponse {
    let controller = ctx.controller().await;
    tracing::debug!("Received request: {:?}", request.id.clone());

    // --- Get server by name from the controller
    let server = match controller.get_server_by_name(&name).await {
        Ok(server) => server,
        Err(error) => {
            tracing::error!("Failed to get server by name: {}", error);
            return error.into_response();
        }
    };

    // --- Request the server to start and wait until it is ready
    match controller.request_server_up(&server).await {
        Ok(_) => {}
        Err(error) => {
            tracing::error!("Failed to register server request: {}", error);
            return error.into_response();
        }
    }

    // --- Get server SSE channels for communication
    let transport = match controller.get_server_transport(&server).await {
        Ok(channels) => {
            tracing::error!("Server transport channels: {:?}", server.name_pod());
            channels
        }
        Err(error) => {
            tracing::error!("Failed to get server transport: {}", error);
            return error.into_response();
        }
    };

    // --- Update the "last_request_at" field in the server status
    match controller.register_server_request(&server).await {
        Ok(_) => {}
        Err(error) => {
            tracing::error!("Failed to register server request: {}", error);
            return error.into_response();
        }
    }

    // --- Send the request and handle response
    let transport = transport.read().await;
    match transport.send(request).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(error) => {
            tracing::error!("Failed to send request: {}", error);
            error.into_response()
        }
    }
}

/// Documentation for the POST /api/v1/servers/{name}/message endpoint
pub fn message_docs(op: TransformOperation) -> TransformOperation {
    op.id("postServerSseMessage")
        .tag("Server")
        .summary("Post SSE Message")
        .description("Sends a message to the server. The server will process the message and return a response.")
        .response::<200, Json<JsonRpcResponse>>()
}
