use super::server_body::MCPServerBody;
use aide::transform::TransformOperation;
use axum::extract::Json;

/// Documentation for the GET /api/v1/servers endpoint
pub fn search(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("searchServers")
        .tag("Server")
        .summary("Search Servers")
        .description("Retrieves a list of all `MCPServer` resources in the current namespace. Returns detailed information about each server, including its configuration, current status, and associated pool.")
        .response_with::<200, Json<Vec<MCPServerBody>>, _>(|response| {
            response
                .description("The `MCPServer`s were found successfully.")
                .example(vec![MCPServerBody::default(), MCPServerBody::default()])
        })
}

/// Documentation for the GET /api/v1/servers/{name} endpoint
pub fn get_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("getServerByName")
        .tag("Server")
        .summary("Get Server")
        .description("Retrieves a specific `MCPServer` by name, returning its complete configuration and current status. This includes the server's pool assignment, container details, transport configuration, and running state.")
        .response_with::<200, Json<MCPServerBody>, _>(|response| {
            response
                .description("The `MCPServer` was found successfully.")
                .example(MCPServerBody::default())
        })
}

/// Documentation for the POST /api/v1/servers endpoint
pub fn create_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("createServer")
        .tag("Server")
        .summary("Create Server")
        .description("Creates a new `MCPServer` resource with the specified name and configuration. This initiates deployment of a Kubernetes Pod and, if applicable, a Service based on the server's transport type (SSE or STDIO).")
        .response_with::<200, Json<MCPServerBody>, _>(|response| {
            response
                .description("The `MCPServer` was created successfully.")
                .example(MCPServerBody::default())
        })
}

/// Documentation for the DELETE /api/v1/servers/{name} endpoint
pub fn delete_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("deleteServer")
        .tag("Server")
        .summary("Delete Server")
        .description("Removes an `MCPServer` and cleans up all its associated Kubernetes resources. This includes terminating the server Pod and deleting any associated Service resources, ensuring complete cleanup.")
        .response_with::<200, Json<MCPServerBody>, _>(|response| {
            response
                .description("The MCPServer was deleted successfully.")
                .example(MCPServerBody::default())
        })
}

/// Documentation for the PATCH /api/v1/servers/{name} endpoint
pub fn update_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("updateServerByName")
        .tag("Server")
        .summary("Update Server")
        .description("Updates the configuration of an existing `MCPServer`. This allows modifying the container image, arguments, environment variables, transport configuration, and other settings while maintaining the server's identity.")
        .response_with::<200, Json<MCPServerBody>, _>(|response| {
            response
                .description("The `MCPServer` was patched successfully.")
                .example(MCPServerBody::default())
        })
}
