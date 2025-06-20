use super::pool_body::MCPPoolList;
use super::{pool_docs, ManagerContext};
use crate::{MCPPool, MCPPoolSpec, ResourceManager};
use aide::axum::routing::get_with;
use aide::axum::{ApiRouter, IntoApiResponse};
use axum::extract::{Json, Path, State};
use axum::response::{IntoResponse, Response};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Request body for creating a new pool
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
struct CreateBody {
    /// Name of the pool to be created. This is a required field
    /// and should be unique within the namespace.
    pub name: String,

    // Include all fields from MCPPoolSpec directly
    #[serde(flatten)]
    pub spec: MCPPoolSpec,
}

/// Handler for GET /api/v1/pools
async fn search(State(ctx): State<ManagerContext>) -> impl IntoApiResponse {
    let client = ctx.get_client().await;
    MCPPool::search(&client, None)
        .await
        .map(MCPPoolList)
        .into_response()
}

/// Handler for GET /api/v1/pools/{name}
async fn get(Path(name): Path<String>, State(ctx): State<ManagerContext>) -> Response {
    let client = ctx.get_client().await;
    MCPPool::get_by_name(&client, &name).await.into_response()
}

/// Handler for POST /api/v1/pools
async fn create(
    State(ctx): State<ManagerContext>,
    Json(body): Json<CreateBody>,
) -> impl IntoApiResponse {
    let client = ctx.get_client().await;
    MCPPool::new(&body.name, body.spec)
        .apply(&client)
        .await
        .into_response()
}

/// Handler for PATCH /api/v1/pools/{name}
async fn patch(
    State(ctx): State<ManagerContext>,
    Path(name): Path<String>,
    Json(spec): Json<MCPPoolSpec>,
) -> impl IntoApiResponse {
    async {
        let client = ctx.get_client().await;
        MCPPool::get_by_name(&client, &name)
            .await?
            .patch(&client, spec)
            .await
    }
    .await
    .into_response()
}

/// Handler for DELETE /api/v1/pools/{name}
async fn delete(Path(name): Path<String>, State(ctx): State<ManagerContext>) -> Response {
    async {
        let client = ctx.get_client().await;
        MCPPool::get_by_name(&client, &name)
            .await?
            .delete(&client)
            .await
    }
    .await
    .into_response()
}

/// Router for pool-related endpoints
pub fn router(ctx: ManagerContext) -> ApiRouter<()> {
    ApiRouter::new()
        .api_route(
            "/",
            get_with(search, pool_docs::search_docs).post_with(create, pool_docs::create_docs),
        )
        .api_route(
            "/{name}",
            get_with(get, pool_docs::get_docs)
                .patch_with(patch, pool_docs::patch_docs)
                .delete_with(delete, pool_docs::delete_docs),
        )
        .with_state(ctx.clone())
}
