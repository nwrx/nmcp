use crate::{Controller, Error, MCPServer, Result, TaskMap, Transport};
use aide::axum::routing::get;
use aide::axum::ApiRouter;
use aide::openapi::OpenApi;
use aide::scalar::Scalar;
use axum::Extension;
use clap::Parser;
use kube::{Client, ResourceExt};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

/// Configuration for the API server
#[derive(Debug, Copy, Clone, Parser)]
pub struct GatewayOptions {
    /// Host address for the API server to bind to
    #[arg(long, default_value = "127.0.0.1")]
    pub host: IpAddr,

    /// Port for the API server to listen on
    #[arg(short, long, default_value = "8080")]
    pub port: u16,
}

pub type TransportStore = TaskMap<String, Transport, Error>;

/// Server struct for the API server
#[derive(Clone)]
pub struct Gateway {
    address: SocketAddr,
    controller: Controller,
    transports: TransportStore,
}

impl std::fmt::Debug for Gateway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gateway")
            .field("address", &self.address)
            .field("controller", &"Controller(...)")
            .field("transports", &self.transports.len())
            .finish()
    }
}

pub type GatewayContext = Arc<Gateway>;

impl Gateway {
    /// Create a new server instance
    pub async fn new(options: GatewayOptions, controller: Controller) -> Result<Self> {
        Ok(Self {
            address: SocketAddr::new(options.host, options.port),
            controller,
            transports: TaskMap::new(),
        })
    }

    pub async fn get_client(&self) -> Client {
        self.controller.get_client()
    }

    pub async fn get_transport(&self, server: &MCPServer) -> Option<Result<Transport>> {
        let client = self.controller.get_client();
        let server = server.clone();
        let key = format!("{}-{}", client.default_namespace(), server.name_any());
        self.clone().transports.get(&key).await
    }

    pub async fn get_or_create_transport(&self, server: &MCPServer) -> Result<Transport> {
        let client = self.controller.get_client();
        let server = server.clone();
        let key = format!("{}-{}", client.default_namespace(), server.name_any());

        // --- Instantiate the transport if it doesn't exist.
        self.clone()
            .transports
            .get_or_insert(&key, || async move { Transport::new(&client, &server) })
            .await
            .expect("Failed to create transport")
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
            .nest_api_service("/{name}", super::sse::router(ctx.clone()))
            .nest_api_service("/health", super::health::router(ctx.clone()))
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
