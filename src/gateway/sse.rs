use super::GatewayContext;
use aide::axum::IntoApiResponse;
use aide::transform::TransformOperation;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Sse};
use axum::Json;
use rmcp::model::{ClientJsonRpcMessage, ServerJsonRpcMessage};
use tokio_stream::StreamExt;

/// Handler for GET /api/v1/servers/{name}/sse
pub async fn stream(
    Path(name): Path<String>,
    State(ctx): State<GatewayContext>,
) -> impl IntoApiResponse {
    let controller = ctx.controller().await;

    // --- Get server by name from the controller.
    let server = match controller.get_server_by_name(&name).await {
        Ok(server) => server,
        Err(error) => return error.into_response(),
    };

    match controller.register_server_request(&server).await {
        Ok(_) => {}
        Err(error) => return error.into_response(),
    }

    // --- Request the server to start and wait until it is ready.
    match controller.request_server_up(&server).await {
        Ok(_) => {}
        Err(error) => return error.into_response(),
    }

    // --- Get server SSE channels for communication
    let channels = match controller.get_server_transport(&server).await {
        Ok(channels) => channels,
        Err(error) => return error.into_response(),
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
    State(ctx): State<GatewayContext>,
    Path(name): Path<String>,
    Json(request): Json<ClientJsonRpcMessage>,
) -> impl IntoApiResponse {
    let controller = ctx.controller().await;

    let response = async {
        let server = controller.get_server_by_name(&name).await?;
        controller.request_server_up(&server).await?;
        controller.register_server_request(&server).await?;
        let transport = controller.get_server_transport(&server).await?;
        let transport = transport.read().await;
        transport.send(request).await
    };

    match response.await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Documentation for the POST /api/v1/servers/{name}/message endpoint
pub fn message_docs(op: TransformOperation) -> TransformOperation {
    op.id("postServerSseMessage")
        .tag("Server")
        .summary("Post SSE Message")
        .description("Sends a message to the server. The server will process the message and return a response.")
        .response::<200, Json<ServerJsonRpcMessage>>()
}
