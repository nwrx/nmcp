use crate::{Controller, Result};
use axum::routing::{delete, get, post};
use axum::Router;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, RwLock};
use structopt::StructOpt;
use tokio::net::TcpListener;
use tracing::info;

// Module declarations
mod health;
mod metrics;
mod pool_create;
mod pool_delete;
mod pool_get;
mod pool_list;
mod pool_update;
mod server_create;
mod server_delete;
mod server_get;
mod server_list;
mod server_message;
mod server_sse;
mod server_update;

// Import handlers within this crate
use health::health_get;
use metrics::metrics_get;
use pool_create::pool_create;
use pool_delete::pool_delete;
use pool_get::pool_get;
use pool_list::pool_list;
use pool_update::pool_update;
use server_create::server_create;
use server_delete::server_delete;
use server_get::server_get;
use server_list::server_list;
use server_message::server_message;
use server_sse::server_sse;
use server_update::server_update;

/// Configuration for the API server
#[derive(Debug, Clone, StructOpt)]
pub struct ServerOptions {
    /// Host address for the API server to bind to
    #[structopt(long, default_value = "127.0.0.1")]
    pub host: IpAddr,

    /// Port for the API server to listen on
    #[structopt(short, long, default_value = "8080")]
    pub port: u16,
}

#[derive(Clone)]
pub struct ServerState {
    namespace: String,
    address: SocketAddr,
    controller: Arc<RwLock<Controller>>,
}

impl ServerState {
    /// Get the namespace
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Get the address
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// Get the controller
    pub fn controller(&self) -> Controller {
        self.controller.read().unwrap().clone()
    }
}

/// Server struct for the API server
pub struct Server {
    namespace: String,
    address: SocketAddr,
    controller: Arc<RwLock<Controller>>,
}

impl Server {
    /// Create a new server instance
    pub async fn new(options: ServerOptions, controller: Controller) -> Result<Self> {
        Ok(Self {
            namespace: "default".to_string(),
            address: SocketAddr::new(options.host, options.port),
            controller: Arc::new(RwLock::new(controller)),
        })
    }

    /// Create the router with all API endpoints
    fn create_router(&self) -> Router {
        let state = ServerState {
            namespace: self.namespace.clone(),
            address: self.address,
            controller: self.controller.clone(),
        };

        Router::new()
            // Health endpoint
            .route("/health", get(health_get))
            .route("/api/v1/metrics", get(metrics_get))
            // Server endpoints
            .route("/api/v1/servers", get(server_list))
            .route("/api/v1/servers", post(server_create))
            .route("/api/v1/servers/{uid}", get(server_get))
            .route("/api/v1/servers/{uid}", delete(server_delete))
            .route("/api/v1/servers/{uid}", post(server_update))
            .route("/api/v1/servers/{uid}/sse", get(server_sse))
            .route("/api/v1/servers/{uid}/messages", post(server_message))
            // Pool endpoints
            .route("/api/v1/pools", get(pool_list))
            .route("/api/v1/pools", post(pool_create))
            .route("/api/v1/pools/{name}", get(pool_get))
            .route("/api/v1/pools/{name}", delete(pool_delete))
            .route("/api/v1/pools/{name}", post(pool_update))
            // Add tracing layer
            .with_state(Arc::new(state))
    }

    /// Start the server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.create_router();
        let listener = TcpListener::bind(&self.address).await?;
        info!("API server listening on {}", self.address);
        let _ = axum::serve(listener, app).await;
        info!("API server stopped");
        Ok(())
    }
}
