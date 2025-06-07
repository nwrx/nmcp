use aide::axum::IntoApiResponse;
use aide::openapi::{OpenApi, Tag};
use aide::transform::TransformOpenApi;
use axum::{Extension, Json};

pub async fn serve(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}

pub fn openapi(api: TransformOpenApi<'_>) -> TransformOpenApi<'_> {
    api.title("NMCP")
        .summary("Kubernetes operator for managing MCP servers")
        .tag(Tag {
            name: "Server".to_string(),
            description: Some("Operations related to the `MCPServer` resources.".to_string()),
            ..Default::default()
        })
}
