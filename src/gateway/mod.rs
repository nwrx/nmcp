use crate::{Controller, Result};
use aide::axum::routing::get;
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::openapi::{OpenApi, Tag};
use aide::scalar::Scalar;
use aide::transform::TransformOpenApi;
use axum::{Extension, Json};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, RwLock};
use structopt::StructOpt;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

mod pool;
mod server;
mod sse;

/// Configuration for the API server
#[derive(Debug, Clone, StructOpt)]
pub struct GatewayOptions {
    /// Host address for the API server to bind to
    #[structopt(long, default_value = "127.0.0.1")]
    pub host: IpAddr,

    /// Port for the API server to listen on
    #[structopt(short, long, default_value = "8080")]
    pub port: u16,
}

/// Server struct for the API server
#[derive(Clone)]
pub struct Gateway {
    address: SocketAddr,
    controller: Arc<RwLock<Controller>>,
}

pub type GatewayContext = Arc<Gateway>;

// Note that this clones the document on each request.
// To be more efficient, we could wrap it into an Arc,
// or even store it as a serialized string.
async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("UNMCP")
        .summary("Kubernetes operator for managing MCP servers")
        .description("This API provides a way to manage MCP servers and pools.")
        .tag(Tag {
            name: "Server".to_string(),
            description: Some("Operations related to the `MCPServer` resources.".to_string()),
            ..Default::default()
        })
        .tag(Tag {
            name: "Pool".to_string(),
            description: Some("Operations related to the `MCPPool` resources.".to_string()),
            ..Default::default()
        })
}

impl Gateway {
    /// Create a new server instance
    pub async fn new(options: GatewayOptions, controller: Controller) -> Result<Self> {
        Ok(Self {
            address: SocketAddr::new(options.host, options.port),
            controller: Arc::new(RwLock::new(controller)),
        })
    }

    pub async fn controller(&self) -> Controller {
        self.controller.read().unwrap().clone()
    }

    /// Start the server
    #[tracing::instrument(name = "Server", skip_all)]
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        aide::generate::extract_schemas(true);
        let ctx = Arc::new(self.clone());
        let mut api = OpenApi::default();

        // --- Set up the API router with the routes.
        let router = ApiRouter::new()
            .route("/openapi.json", get(serve_api))
            .route("/", Scalar::new("/openapi.json").axum_route())
            .nest_api_service("/api/v1/servers", server::routes(ctx.clone()))
            .nest_api_service("/api/v1/pools", pool::routes(ctx.clone()))
            .finish_api_with(&mut api, api_docs)
            .layer(Extension(api))
            .layer(TraceLayer::new_for_http())
            .with_state(ctx.clone());

        // --- Set up the TCP listener and bind to the address.
        let listener = match TcpListener::bind(&self.address).await {
            Ok(listener) => {
                tracing::info!("Listening on http://{}", self.address);
                listener
            }
            Err(error) => {
                tracing::error!("Failed to bind to address {}: {}", self.address, error);
                return Err(Box::new(error));
            }
        };

        // --- Start serving the API.
        axum::serve(listener, router).await.unwrap();
        Ok(())
    }
}
