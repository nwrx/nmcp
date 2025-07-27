use aide::transform::TransformOperation;
use axum::Json;
use rmcp::model::ServerJsonRpcMessage;

/// Documentation for the GET /{name}/sse endpoint
pub fn sse_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("getServerSse")
        .tag("Server")
        .summary("Server SSE")
        .description("Establishes a Server-Sent Events (SSE) connection to the server. This allows for real-time updates and notifications from the server. Returns a stream of JSON messages.")
}

/// Documentation for the POST /{name}/message endpoint
pub fn message_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("postServerSseMessage")
        .tag("Server")
        .summary("Post SSE Message")
        .description("Sends a message to the server. The server will process the message and return a response.")
        .response::<200, Json<ServerJsonRpcMessage>>()
}

/// Documentation for the GET /{name}/logs endpoint
pub fn logs_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("getServerLogs")
        .tag("Server")
        .summary("Get Server Logs")
        .description("Retrieves the logs for the server. This is useful for debugging and monitoring server activity.")
        .response::<200, String>()
}

/// Documentation for the POST /{name}/request endpoint
pub fn request_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("postServerRequest")
        .tag("Server")
        .summary("Request Server")
        .description("Requests the server and waits until it's ready. This is useful for ensuring the server is available before performing other operations. Does not return or send any data.")
        .response::<200, ()>()
}

/// Documentation for the POST /{name}/shutdown endpoint
pub fn shutdown_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("postServerShutdown")
        .tag("Server")
        .summary("Shutdown Server")
        .description("Shuts down the server. This is useful for stopping the server gracefully. Does not return or send any data.")
        .response::<200, ()>()
}
