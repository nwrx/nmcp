use super::{sse_docs, GatewayContext};
use crate::{
    Error, MCPServer, MCPServerCondition as Condition, MCPServerRequestedState as RequestState,
    ResourceManager,
};
use aide::axum::routing::{get_with, post_with};
use aide::axum::{ApiRouter, IntoApiResponse};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::Json;
use futures::AsyncBufReadExt;
use rmcp::model::ClientJsonRpcMessage;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_util::bytes;

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SseQuery {
    /// The maximum time to wait for the server to be ready before sending the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    timeout: Option<u64>,
}

/// Handler for GET /{name}/sse
#[tracing::instrument(name = "GET /{name}/sse", skip_all)]
async fn sse(
    Path(name): Path<String>,
    Query(query): Query<SseQuery>,
    State(ctx): State<GatewayContext>,
) -> impl IntoApiResponse {
    async {
        let client = ctx.get_client().await;
        let server = MCPServer::get_by_name(&client, &name).await?;
        let timeout = query.timeout.map(Duration::from_secs);
        let reason = RequestState::Connection;
        let condition = Condition::Requested(reason);

        // --- Update the status so it can be picked-up by the operator.
        server.request(&client).await?;
        server.notify_connect(&client).await?;
        server.push_condition(&client, condition).await?;
        server.wait_until_ready(&client, timeout).await?;

        // --- Get the transport for the server and create a peer.
        let mut transport = ctx.get_transport(&server)?;
        let peer = transport.subscribe().await?;
        let endpoint = format!("/{name}/message");

        // --- Create the handler for the SSE stream closure.
        let server = server.clone();
        let on_close = || tokio::spawn(async move { server.notify_disconnect(&client).await });
        let stream = peer.sse(endpoint, on_close).await;
        Ok::<_, Error>(stream)
    }
    .await
    .into_response()
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageQuery {
    /// The ID of the session to which the message should be sent.
    session_id: String,
    /// The maximum time to wait for the server to be ready before sending the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    timeout: Option<u64>,
}

/// Handler for POST /{name}/message
#[tracing::instrument(name = "POST /{name}/message", skip_all)]
async fn message(
    State(ctx): State<GatewayContext>,
    Path(name): Path<String>,
    Query(query): Query<MessageQuery>,
    Json(request): Json<ClientJsonRpcMessage>,
) -> impl IntoApiResponse {
    async {
        let client = ctx.get_client().await;
        let server = MCPServer::get_by_name(&client, &name).await?;
        let timeout = query.timeout.map(Duration::from_secs);

        // --- Request the server and wait until it's ready.
        server.request(&client).await?;
        server.wait_until_ready(&client, timeout).await?;

        // --- Get the transport for the server and send the request.
        let transport = ctx.get_transport(&server)?;
        let peer = transport.get_peer(query.session_id).await?;
        let result = peer.send_request(request).await?;
        Ok::<_, Error>(Json(result))
    }
    .await
    .map_err(|e| e.trace())
    .into_response()
}

/// Handler for `GET /{name}/logs`
#[tracing::instrument(name = "GET /{name}/logs", skip_all)]
async fn logs(Path(name): Path<String>, State(ctx): State<GatewayContext>) -> impl IntoApiResponse {
    async {
        let client = ctx.get_client().await;
        let server = MCPServer::get_by_name(&client, &name).await?;
        server.wait_until_ready(&client, None).await?;

        // --- Get the log stream for the server.
        let stream = server.get_logs(&client).await?;
        let stream = stream.lines();
        let stream = futures::StreamExt::map(stream, |bytes| match bytes {
            Ok(bytes) => Ok(bytes::Bytes::from(bytes)),
            Err(e) => Err(std::io::Error::other(e)),
        });

        // --- Wrap the stream in an axum Body.
        let body = Body::from_stream(stream);
        let response = axum::response::Response::builder()
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .header(header::CONNECTION, "keep-alive")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(body)
            .unwrap();

        // --- Send the response.
        Ok::<_, Error>(response)
    }
    .await
    .into_response()
}

/// Handler for POST /{name}/request
#[tracing::instrument(name = "POST /{name}/request", skip_all)]
async fn request(
    Path(name): Path<String>,
    State(ctx): State<GatewayContext>,
) -> impl IntoApiResponse {
    async {
        let client = ctx.get_client().await;
        let server = MCPServer::get_by_name(&client, &name).await?;
        let reason = RequestState::ManualStart;
        let condition = Condition::Requested(reason);

        // --- Request the server and update its status.
        server.request(&client).await?;
        server.push_condition(&client, condition).await?;
        Ok::<(), Error>(())
    }
    .await
    .into_response()
}

/// Handler for `POST /{name}/shutdown`
#[tracing::instrument(name = "POST /{name}/shutdown", skip_all)]
async fn shutdown(
    Path(name): Path<String>,
    State(ctx): State<GatewayContext>,
) -> impl IntoApiResponse {
    async {
        let client = ctx.get_client().await;
        let server = MCPServer::get_by_name(&client, &name).await?;
        let reason = RequestState::ManualStop;
        let condition = Condition::Requested(reason);

        // --- Request the server to shutdown and update its status.
        server.shutdown(&client).await?;
        server.push_condition(&client, condition).await?;
        Ok::<(), Error>(())
    }
    .await
    .into_response()
}

/// Router for SSE-related endpoints
pub fn router(ctx: GatewayContext) -> ApiRouter<()> {
    ApiRouter::new()
        .api_route("/sse", get_with(sse, sse_docs::sse_docs))
        .api_route("/logs", get_with(logs, sse_docs::logs_docs))
        .api_route("/message", post_with(message, sse_docs::message_docs))
        .api_route("/request", post_with(request, sse_docs::request_docs))
        .api_route("/shutdown", post_with(shutdown, sse_docs::shutdown_docs))
        .with_state(ctx)
}
