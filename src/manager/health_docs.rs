use super::health::Status;
use aide::transform::TransformOperation;
use axum::extract::Json;

/// Documentation for the GET /health/status endpoint
pub fn status_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("getHealthStatus")
        .tag("Health")
        .summary("Health Status")
        .description("Retrieves the health status of the manager service, including version information and current timestamp. This endpoint provides a comprehensive health check that includes service availability and metadata.")
        .response_with::<200, Json<Status>, _>(|response| {
            response
                .description("The service is healthy and operational.")
                .example(Status::default())
        })
}

/// Documentation for the GET /health/ping endpoint
pub fn ping_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.id("pingHealth")
        .tag("Health")
        .summary("Health Ping")
        .description("Simple health check endpoint that returns a basic HTTP 204 status. This lightweight endpoint is ideal for load balancers, monitoring systems, and automated health checks that only need to verify service availability.")
        .response_with::<204, (), _>(|response| {
            response
                .description("The service is alive and responding to requests.")
        })
}
