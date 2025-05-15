use super::{sse, ApplicationContext};
use crate::{MCPServerBody, MCPServerSpec};
use aide::axum::routing::{get_with, post_with};
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::transform::TransformOperation;
use axum::extract::{Json, Path, State};
use axum::response::IntoResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

///////////////////////////////////////////////////////////////////////////////

/// Query parameters for server list endpoint
#[derive(Deserialize)]
struct SearchQuery {
    // pool: Option<String>,
}

/// Handler for GET /api/v1/servers
async fn search(
    State(ctx): State<ApplicationContext>,
    // Query(query): Query<SearchQuery>,
) -> impl IntoApiResponse {
    ctx.controller()
        .await
        .search_servers()
        .await
        .into_response()
}

/// Documentation for the GET /api/v1/servers endpoint
fn search_docs(op: TransformOperation) -> TransformOperation {
    op.id("searchServers")
        .tag("Server")
        .summary("Search Servers")
        .description("Retrieves a list of all `MCPServer` resources in the current namespace. Returns detailed information about each server, including its configuration, current status, and associated pool.")
        .response_with::<200, Json<Vec<MCPServerBody>>, _>(|response| {
            response
                .description("The `MCPServer`s were found successfully.")
                .example(vec![MCPServerBody::example(), MCPServerBody::example()])
        })
}

///////////////////////////////////////////////////////////////////////////////

/// Handler for GET /api/v1/servers/{name}
async fn get(
    Path(name): Path<String>,
    State(ctx): State<ApplicationContext>,
) -> impl IntoApiResponse {
    ctx.controller()
        .await
        .get_server_by_name(&name)
        .await
        .into_response()
}

/// Documentation for the GET /api/v1/servers/{name} endpoint
fn get_docs(op: TransformOperation) -> TransformOperation {
    op.id("getServerByName")
        .tag("Server")
        .summary("Get Server")
        .description("Retrieves a specific `MCPServer` by name, returning its complete configuration and current status. This includes the server's pool assignment, container details, transport configuration, and running state.")
        .response_with::<200, Json<MCPServerBody>, _>(|response| {
            response
                .description("The `MCPServer` was found successfully.")
                .example(MCPServerBody::example())
        })
}

///////////////////////////////////////////////////////////////////////////////

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

/// Handler for POST /api/v1/servers
async fn create(
    State(ctx): State<ApplicationContext>,
    Json(body): Json<CreateBody>,
) -> impl IntoApiResponse {
    ctx.controller()
        .await
        .create_server(&body.name, body.spec)
        .await
        .into_response()
}

/// Documentation for the POST /api/v1/servers endpoint
fn create_docs(op: TransformOperation) -> TransformOperation {
    op.id("createServer")
        .tag("Server")
        .summary("Create Server")
        .description("Creates a new `MCPServer` resource with the specified name and configuration. This initiates deployment of a Kubernetes Pod and, if applicable, a Service based on the server's transport type (SSE or STDIO).")
        .response_with::<200, Json<MCPServerBody>, _>(|response| {
            response
                .description("The `MCPServer` was created successfully.")
                .example(MCPServerBody::example())
        })
}

///////////////////////////////////////////////////////////////////////////////

/// Handler for DELETE /api/v1/servers/{name}
async fn delete(
    State(ctx): State<ApplicationContext>,
    Path(name): Path<String>,
) -> impl IntoApiResponse {
    ctx.controller()
        .await
        .delete_server(&name)
        .await
        .into_response()
}

/// Documentation for the DELETE /api/v1/servers/{name} endpoint
fn delete_docs(op: TransformOperation) -> TransformOperation {
    op.id("deleteServer")
        .tag("Server")
        .summary("Delete Server")
        .description("Removes an `MCPServer` and cleans up all its associated Kubernetes resources. This includes terminating the server Pod and deleting any associated Service resources, ensuring complete cleanup.")
        .response_with::<200, Json<MCPServerBody>, _>(|response| {
            response
                .description("The MCPServer was deleted successfully.")
                .example(MCPServerBody::example())
        })
}

///////////////////////////////////////////////////////////////////////////////

/// Handler for PATCH /api/v1/servers/{name}
async fn patch(
    State(ctx): State<ApplicationContext>,
    Path(name): Path<String>,
    Json(spec): Json<MCPServerSpec>,
) -> impl IntoApiResponse {
    ctx.controller()
        .await
        .patch_server_spec(&name, &spec)
        .await
        .into_response()
}

/// Documentation for the PATCH /api/v1/servers/{name} endpoint
fn patch_docs(op: TransformOperation) -> TransformOperation {
    op.id("patchServerByName")
        .tag("Server")
        .summary("Patch Server")
        .description("Updates the configuration of an existing `MCPServer`. This allows modifying the container image, arguments, environment variables, transport configuration, and other settings while maintaining the server's identity.")
        .response_with::<200, Json<MCPServerBody>, _>(|response| {
            response
                .description("The `MCPServer` was patched successfully.")
                .example(MCPServerBody::example())
        })
}

///////////////////////////////////////////////////////////////////////////////

pub fn routes(ctx: ApplicationContext) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            get_with(search, search_docs).post_with(create, create_docs),
        )
        .api_route(
            "/{name}",
            get_with(get, get_docs)
                .delete_with(delete, delete_docs)
                .patch_with(patch, patch_docs),
        )
        .api_route("/{name}/sse", get_with(sse::stream, sse::stream_docs))
        .api_route(
            "/{name}/message",
            post_with(sse::message, sse::message_docs),
        )
        .with_state(ctx)
}
