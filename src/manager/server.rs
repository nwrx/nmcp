use super::server_body::MCPServerList;
use super::{server_docs, ManagerContext};
use crate::{MCPServer, MCPServerSpec, ResourceManager};
use aide::axum::routing::get_with;
use aide::axum::{ApiRouter, IntoApiResponse};
use axum::extract::{Json, Path, State};
use axum::response::IntoResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Query parameters for server list endpoint
#[derive(Deserialize)]
struct SearchQuery {
    // pool: Option<String>,
}

/// Request body for creating a new pool
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateBody {
    /// Name of the pool to be created. This is a required field
    /// and should be unique within the namespace.
    pub name: String,

    // Include all fields from MCPServerSpec directly
    #[serde(flatten)]
    pub spec: MCPServerSpec,
}

/// Handler for GET /api/v1/servers
pub async fn search(
    State(ctx): State<ManagerContext>,
    // Query(query): Query<SearchQuery>,
) -> impl IntoApiResponse {
    let client = ctx.get_client().await;
    MCPServer::search(&client, None)
        .await
        .map(MCPServerList)
        .into_response()
}

/// Handler for GET /api/v1/servers/{name}
pub async fn get(
    Path(name): Path<String>,
    State(ctx): State<ManagerContext>,
) -> impl IntoApiResponse {
    let client = ctx.get_client().await;
    MCPServer::get_by_name(&client, &name).await.into_response()
}

/// Handler for POST /api/v1/servers
pub async fn create(
    State(ctx): State<ManagerContext>,
    Json(body): Json<CreateBody>,
) -> impl IntoApiResponse {
    let client = ctx.get_client().await;
    MCPServer::new(&body.name, body.spec)
        .apply(&client)
        .await
        .into_response()
}

/// Handler for DELETE /api/v1/servers/{name}
pub async fn delete(
    State(ctx): State<ManagerContext>,
    Path(name): Path<String>,
) -> impl IntoApiResponse {
    async {
        let client = ctx.get_client().await;
        MCPServer::get_by_name(&client, &name)
            .await?
            .delete(&client)
            .await
    }
    .await
    .into_response()
}

/// Handler for PATCH /api/v1/servers/{name}
pub async fn patch(
    State(ctx): State<ManagerContext>,
    Path(name): Path<String>,
    Json(spec): Json<MCPServerSpec>,
) -> impl IntoApiResponse {
    async {
        let client = ctx.get_client().await;
        MCPServer::get_by_name(&client, &name)
            .await?
            .patch(&client, spec)
            .await
    }
    .await
    .into_response()
}

/// Router for server-related endpoints
pub fn router(ctx: ManagerContext) -> ApiRouter<()> {
    ApiRouter::new()
        .api_route(
            "/",
            get_with(search, server_docs::search).post_with(create, server_docs::create_docs),
        )
        .api_route(
            "/{name}",
            get_with(get, server_docs::get_docs)
                .patch_with(patch, server_docs::patch_docs)
                .delete_with(delete, server_docs::delete_docs),
        )
        .with_state(ctx)
}
