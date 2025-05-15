use super::GatewayContext;
use crate::{MCPPoolBody, MCPPoolSpec};
use aide::axum::routing::get_with;
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::transform::TransformOperation;
use axum::extract::{Json, Path, State};
use axum::response::{IntoResponse, Response};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Handler for GET /api/v1/pools
pub async fn search(State(ctx): State<GatewayContext>) -> impl IntoApiResponse {
    ctx.controller()
        .await
        .clone()
        .search_pools()
        .await
        .into_response()
}

/// Documentation for the GET /api/v1/pools endpoint
fn search_docs(op: TransformOperation) -> TransformOperation {
    op.id("searchPools")
        .tag("Pool")
        .summary("Search Pools")
        .description("Retrieves a list of all `MCPPool` resources in the current namespace, including their specifications and statuses. Pools manage server limits and default configurations.")
        .response_with::<200, Json<Vec<MCPPoolBody>>, _>(|response| {
            response
                .description("The `MCPPool`s were found successfully.")
                .example(vec![MCPPoolBody::example(), MCPPoolBody::example()])
        })
}

///////////////////////////////////////////////////////////////////////////////

/// Handler for GET /api/v1/pools/{name}
pub async fn get(Path(name): Path<String>, State(ctx): State<GatewayContext>) -> Response {
    ctx.controller()
        .await
        .clone()
        .get_pool_by_name(&name)
        .await
        .into_response()
}

/// Documentation for the GET /api/v1/pools/{name} endpoint
fn get_docs(op: TransformOperation) -> TransformOperation {
    op.id("getPoolByName")
        .tag("Pool")
        .summary("Get Pool")
        .description("Retrieves a specific `MCPPool` by name, returning its complete specifications, status, and configuration. This includes server limits, resource requirements, and idle timeout settings.")
        .response_with::<200, Json<MCPPoolBody>, _>(|response| {
            response
                .description("The `MCPPool` was retrieved successfully.")
                .example(MCPPoolBody::example())
        })
}

///////////////////////////////////////////////////////////////////////////////

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

/// Handler for POST /api/v1/pools
async fn create(
    State(ctx): State<GatewayContext>,
    Json(body): Json<CreateBody>,
) -> impl IntoApiResponse {
    ctx.controller()
        .await
        .create_pool(&body.name, body.spec)
        .await
        .into_response()
}

/// Documentation for the POST /api/v1/pools endpoint
fn create_docs(op: TransformOperation) -> TransformOperation {
    op.id("createPool")
        .tag("Pool")
        .summary("Create Pool")
        .description("Creates a new `MCPPool` resource with the specified name and configurations. The pool defines server limits, resource allocations, and idle timeout settings that apply to all servers within it.")
        .response_with::<200, Json<MCPPoolBody>, _>(|response| {
            response
                .description("The `MCPPool` was created successfully.")
                .example(MCPPoolBody::example())
        })
}

///////////////////////////////////////////////////////////////////////////////

/// Handler for PATCH /api/v1/pools/{name}
async fn patch(
    State(ctx): State<GatewayContext>,
    Path(name): Path<String>,
    Json(spec): Json<MCPPoolSpec>,
) -> impl IntoApiResponse {
    ctx.controller()
        .await
        .patch_pool_spec(&name, &spec)
        .await
        .into_response()
}

/// Documentation for the PATCH /api/v1/pools/{name} endpoint
fn patch_docs(op: TransformOperation) -> TransformOperation {
    op.id("patchPoolByName")
        .tag("Pool")
        .summary("Patch Pool")
        .description("Updates the configuration of an existing `MCPPool`. This allows modifying server limits, resource allocations, and idle timeout settings without recreating the pool.")
        .response_with::<200, Json<MCPPoolBody>, _>(|response| {
            response
                .description("The `MCPPool` was patched successfully.")
                .example(MCPPoolBody::example())
        })
}

///////////////////////////////////////////////////////////////////////////////

/// Handler for DELETE /api/v1/pools/{name}
async fn delete(Path(name): Path<String>, State(ctx): State<GatewayContext>) -> Response {
    ctx.controller()
        .await
        .delete_pool(&name)
        .await
        .into_response()
}

/// Documentation for the DELETE /api/v1/pools/{name} endpoint
fn delete_docs(op: TransformOperation) -> TransformOperation {
    op.id("deletePoolByName")
        .tag("Pool")
        .summary("Delete Pool")
        .description("Removes an `MCPPool` and cleans up all associated resources. The deletion process uses Kubernetes finalizers to ensure proper cleanup of dependent resources before removing the pool completely. This includes stopping server pods, deleting server services, and verifying all cleanup operations.")
        .response_with::<200, Json<MCPPoolBody>, _>(|response| {
            response
                .description("The `MCPPool` was deleted successfully.")
                .example(MCPPoolBody::example())
        })
}

///////////////////////////////////////////////////////////////////////////////

pub fn routes(ctx: GatewayContext) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            get_with(search, search_docs).post_with(create, create_docs),
        )
        .api_route(
            "/{name}",
            get_with(get, get_docs)
                .patch_with(patch, patch_docs)
                .delete_with(delete, delete_docs),
        )
        .with_state(ctx)
}
