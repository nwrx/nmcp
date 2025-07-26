use crate::{Controller, Error, MCPServer, Result, Transport};
use aide::axum::routing::get;
use aide::axum::ApiRouter;
use aide::openapi::OpenApi;
use aide::scalar::Scalar;
use axum::Extension;
use clap::Parser;
use kube::{Client, ResourceExt};
use moka::sync::Cache;
use std::fmt::{Debug, Formatter};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
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

    /// Interval for cleaning up orphaned transports (in seconds)
    #[arg(long, default_value = "1")]
    pub cleanup_interval: u64,

    /// Maximum age for idle transports (2 minutes by default)
    /// This is the maximum time a transport can be idle before
    /// it is considered stale and cleaned up.
    #[arg(long, default_value = "120")]
    pub max_idle_age: u64,

    /// Maximum age for transports (30 minutes by default)
    /// This is the maximum time a transport can exist before
    /// it is considered stale and cleaned up.
    #[arg(long, default_value = "1800")]
    pub max_age: u64,

    /// The maximum number of transports to keep in the cache
    /// This number should be high enough to accommodate for all
    /// transports that are expected to be active at the same time
    /// but low enough to avoid excessive memory usage.
    #[arg(long, default_value = "1024")]
    pub max_cache_capacity: u64,
}

pub type TransportStore = Cache<String, Result<Transport>>;

/// Server struct for the API server
pub struct Gateway {
    address: SocketAddr,
    controller: Controller,
    transports: TransportStore,
}

impl Debug for Gateway {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gateway")
            .field("address", &self.address)
            .field("controller", &"Controller(...)")
            .field("transports", &self.transports.entry_count())
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
            transports: Cache::builder()
                .max_capacity(options.max_cache_capacity)
                .time_to_live(Duration::from_secs(options.max_age))
                .time_to_idle(Duration::from_secs(options.max_idle_age))
                .build(),
        })
    }

    /// Get the controller instance associated with this server.
    pub async fn get_client(&self) -> Client {
        self.controller.get_client()
    }

    /// Get or create the `Transport` instance for a given server. If the transport does not exist,
    /// it will be created within a task and stored in the `transports` map. Since `Transport` instanciation
    /// may take some time, this method returns a `Result<Transport>` to ensure no concurrent access issues arise.
    #[tracing::instrument(name = "GetTransport", skip(self, server))]
    pub fn get_transport(&self, server: &MCPServer) -> Result<Transport> {
        let client = self.controller.get_client();
        let server = server.clone();
        let key = format!("{}-{}", client.default_namespace(), server.name_any());

        // --- Get or create a transport for the server. Uses `moka::sync::Cache` to handle concurrent
        // --- requests safely - if multiple requests across multiple threads try to get the same transport,
        // --- only one will create it, while others will wait for the result. This ensures that we do not
        // --- create multiple transports for the same server.
        self.transports
            .clone()
            .get_with(key, || Transport::new(&client, &server))
    }

    /// Start the HTTP server and listen for incoming requests. This method sets up the API routes,
    /// Start the HTTP server and listen for incoming requests. This method sets up the API routes,
    /// binds to the specified address, and starts serving the API using Axum + Aide.
    #[tracing::instrument(name = "Gateway", skip_all)]
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
