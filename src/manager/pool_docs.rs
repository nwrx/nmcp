use super::pool_body::MCPPoolBody;
use aide::transform::TransformOperation;
use axum::extract::Json;

/// Documentation for the GET /api/v1/pools endpoint
pub fn search_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("searchPools")
        .tag("Pool")
        .summary("Search Pools")
        .description("Retrieves a list of all `MCPPool` resources in the current namespace, including their specifications and statuses. Pools manage server limits and default configurations.")
        .response_with::<200, Json<Vec<MCPPoolBody>>, _>(|response| {
            response
                .description("The `MCPPool`s were found successfully.")
                .example(vec![MCPPoolBody::default(), MCPPoolBody::default()])
        })
}

/// Documentation for the GET /api/v1/pools/{name} endpoint
pub fn get_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("getPoolByName")
        .tag("Pool")
        .summary("Get Pool")
        .description("Retrieves a specific `MCPPool` by name, returning its complete specifications, status, and configuration. This includes server limits, resource requirements, and idle timeout settings.")
        .response_with::<200, Json<MCPPoolBody>, _>(|response| {
            response
                .description("The `MCPPool` was retrieved successfully.")
                .example(MCPPoolBody::default())
        })
}

/// Documentation for the POST /api/v1/pools endpoint
pub fn create_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("createPool")
        .tag("Pool")
        .summary("Create Pool")
        .description("Creates a new `MCPPool` resource with the specified name and configurations. The pool defines server limits, resource allocations, and idle timeout settings that apply to all servers within it.")
        .response_with::<200, Json<MCPPoolBody>, _>(|response| {
            response
                .description("The `MCPPool` was created successfully.")
                .example(MCPPoolBody::default())
        })
}

/// Documentation for the PUT /api/v1/pools/{name} endpoint
pub fn update_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("updatePoolByName")
        .tag("Pool")
        .summary("Update Pool")
        .description("Updates the configuration of an existing `MCPPool`. This allows modifying server limits, resource allocations, and idle timeout settings without recreating the pool.")
        .response_with::<200, Json<MCPPoolBody>, _>(|response| {
            response
                .description("The `MCPPool` was updated successfully.")
                .example(MCPPoolBody::default())
        })
}

/// Documentation for the DELETE /api/v1/pools/{name} endpoint
pub fn delete_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("deletePoolByName")
        .tag("Pool")
        .summary("Delete Pool")
        .description("Removes an `MCPPool` and cleans up all associated resources. The deletion process uses Kubernetes finalizers to ensure proper cleanup of dependent resources before removing the pool completely. This includes stopping server pods, deleting server services, and verifying all cleanup operations.")
        .response_with::<200, Json<MCPPoolBody>, _>(|response| {
            response
                .description("The `MCPPool` was deleted successfully.")
                .example(MCPPoolBody::default())
        })
}
