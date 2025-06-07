use super::{sse_docs, GatewayContext};
use crate::{Error, MCPServer, ResourceManager};
use aide::axum::routing::{get_with, post_with};
use aide::axum::{ApiRouter, IntoApiResponse};
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use rmcp::model::ClientJsonRpcMessage;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageQuery {
    /// The ID of the session to which the message should be sent.
    session_id: String,
}

/// Handler for GET /{name}/sse
#[tracing::instrument(name = "GET /{name}/sse", skip_all)]
async fn stream(
    Path(name): Path<String>,
    State(ctx): State<GatewayContext>,
) -> impl IntoApiResponse {
    async {
        let client = ctx.get_client().await;
        let server = MCPServer::get_by_name(&client, &name).await?;
        let server = server.request_server_up(&client).await?;
        let server = server.register_server_request(&client).await?;
        let transport = ctx.get_or_create_transport(&server).await?;
        let peer = transport.subscribe().await?;
        let endpoint = format!("/{name}/message");
        let stream = peer.sse(endpoint).await;
        Ok::<_, Error>(stream)
    }
    .await
    .map_err(|e| e.trace())
    .into_response()
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
        let server = server.request_server_up(&client).await?;
        let server = server.register_server_request(&client).await?;
        let transport = ctx.get_or_create_transport(&server).await?;
        let peer = transport.get_peer(query.session_id).await?;
        let result = peer.send_request(request).await?;
        Ok::<_, Error>(Json(result))
    }
    .await
    .map_err(|e| e.trace())
    .into_response()
}

/// Router for SSE-related endpoints
pub fn router(ctx: GatewayContext) -> ApiRouter<()> {
    ApiRouter::new()
        .api_route("/sse", get_with(stream, sse_docs::stream_docs))
        .api_route("/message", post_with(message, sse_docs::message_docs))
        .with_state(ctx)
}
