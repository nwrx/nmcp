use aide::transform::TransformOperation;
use axum::Json;
use rmcp::model::ServerJsonRpcMessage;

/// Documentation for the GET /api/v1/servers/{name}/sse endpoint
pub fn stream_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("getServerSse")
        .tag("Server")
        .summary("Server SSE")
        .description("Establishes a Server-Sent Events (SSE) connection to the server. This allows for real-time updates and notifications from the server. Returns a stream of JSON messages.")
}

/// Documentation for the POST /api/v1/servers/{name}/message endpoint
pub fn message_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("postServerSseMessage")
        .tag("Server")
        .summary("Post SSE Message")
        .description("Sends a message to the server. The server will process the message and return a response.")
        .response::<200, Json<ServerJsonRpcMessage>>()
}
