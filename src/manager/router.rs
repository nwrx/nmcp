use crate::{Controller, Error, Result};
use aide::axum::routing::get;
use aide::axum::ApiRouter;
use aide::openapi::OpenApi;
use aide::redoc::Redoc;
use aide::scalar::Scalar;
use aide::swagger::Swagger;
use axum::Extension;
use clap::Parser;
use kube::Client;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

/// Configuration for the API server
#[derive(Debug, Clone, Copy, Parser)]
pub struct ManagerOptions {
    /// Host address for the API server to bind to
    #[arg(long, default_value = "127.0.0.1")]
    pub host: IpAddr,

    /// Port for the API server to listen on
    #[arg(short, long, default_value = "8080")]
    pub port: u16,
}

/// Server struct for the API server
#[derive(Clone)]
pub struct Manager {
    address: SocketAddr,
    controller: Controller,
}

impl std::fmt::Debug for Manager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Manager")
            .field("address", &self.address)
            .field("controller", &"Controller(...)")
            .finish()
    }
}

pub type ManagerContext = Arc<Manager>;

impl Manager {
    /// Create a new server instance
    pub async fn new(options: ManagerOptions, controller: Controller) -> Result<Self> {
        Ok(Self {
            address: SocketAddr::new(options.host, options.port),
            controller,
        })
    }

    /// Get the context for the gateway, which includes the server address and controller.
    pub fn context(&self) -> ManagerContext {
        Arc::new(self.clone())
    }

    pub async fn get_client(&self) -> Client {
        self.controller.get_client()
    }

    /// Start the server
    #[tracing::instrument(name = "Server", skip_all)]
    pub async fn start(self) -> Result<()> {
        aide::generate::extract_schemas(true);
        let address = self.address;
        let ctx = Arc::new(self);
        let mut api = OpenApi::default();

        // --- Set up the API router with the routes.
        let router = ApiRouter::new()
            .route("/openapi.json", get(super::docs::serve))
            .route("/", Scalar::new("/openapi.json").axum_route())
            .route("/redoc", Redoc::new("/openapi.json").axum_route())
            .route("/swagger", Swagger::new("/openapi.json").axum_route())
            .nest_api_service("/api/v1/servers", super::server::router(ctx.clone()))
            .nest_api_service("/api/v1/pools", super::pool::router(ctx.clone()))
            .finish_api_with(&mut api, super::docs::openapi)
            .layer(Extension(api))
            .layer(TraceLayer::new_for_http())
            .with_state(ctx.clone());

        // --- Set up the TCP listener and bind to the address.
        let listener = TcpListener::bind(&address).await.map_err(Error::from)?;
        tracing::info!("Listening on http://{}", address);

        // --- Start serving the API.
        axum::serve(listener, router).await.unwrap();
        Ok(())
    }
}
