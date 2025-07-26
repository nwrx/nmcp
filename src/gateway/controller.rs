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
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tower_http::trace::TraceLayer;
use tracing::Instrument;

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
}

pub type TransportStore = TaskMap<String, Transport, Error>;

/// Server struct for the API server
pub struct Gateway {
    address: SocketAddr,
    controller: Controller,
    transports: TransportStore,
    cleanup_task: Option<JoinHandle<()>>,
    cleanup_interval: Duration,
    max_age: Duration,
    max_idle_age: Duration,
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
            cleanup_task: None,
            cleanup_interval: Duration::from_secs(options.cleanup_interval),
            max_idle_age: Duration::from_secs(options.max_idle_age),
            max_age: Duration::from_secs(options.max_age),
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
    pub async fn get_transport(&self, server: &MCPServer) -> Result<Transport> {
        let client = self.controller.get_client();
        let server = server.clone();
        let key = format!("{}-{}", client.default_namespace(), server.name_any());

        // --- Instantiate the transport if it doesn't exist.
        self.transports
            .clone()
            .get_or_insert(&key, || {
                async move { Transport::new(&client, &server) }.instrument(tracing::Span::current())
            })
            .await
            .expect("Failed to create transport")
    }

    /// Clean up idle transports that have not been accessed for a specified duration.
    /// This method iterates through the `transports` map and removes any transport that has been idle
    /// for longer than the configured `max_idle_age`. It also logs the cleanup activity.
    async fn start_cleanup_task(&mut self) -> Result<()> {
        let cleanup_interval = self.cleanup_interval;
        let mut transports = self.transports.clone();
        let max_age = self.max_age;
        let max_idle_age = self.max_idle_age;
        self.cleanup_task = Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            loop {
                let _ = interval.tick().await;
                let mut stale_keys = Vec::new();

                // --- Find stale transports.
                for (key, transport) in transports.iter_tasks().await {
                    let transport = transport.read().await;
                    let transport = transport.peek().await;
                    match transport {
                        Some(Ok(transport)) => {
                            let age = transport.age();
                            let idle_time = transport.idle_time().await;
                            if age > max_age {
                                stale_keys.push(key.clone());
                                tracing::info!("Transport {} is too old (age: {:?})", key, age);
                            } else if idle_time > max_idle_age {
                                stale_keys.push(key.clone());
                                tracing::info!(
                                    "Transport {} is idle for too long (idle time: {:?})",
                                    key,
                                    idle_time
                                );
                            } else {
                                tracing::trace!(
                                    "Transport {} is active (age: {:?}, idle time: {:?})",
                                    key,
                                    age,
                                    idle_time
                                );
                            }
                        }
                        Some(Err(error)) => {
                            stale_keys.push(key.clone());
                            tracing::error!("Error accessing transport {}: {}", key, error);
                        }
                        None => {
                            tracing::warn!("Transport {} is not available yet", key);
                        }
                    }
                }

                // --- Remove stale transports.
                for key in &stale_keys {
                    match transports.remove(key).await {
                        Some(Ok(transport)) => {
                            if let Ok(mut transport) = Arc::try_unwrap(transport) {
                                let _ = transport.close().await;
                            } else {
                                tracing::warn!(
                                    "Transport {} still has references, cannot close cleanly",
                                    key
                                );
                            }
                        }
                        Some(Err(error)) => {
                            tracing::error!("Failed to remove transport {}: {}", key, error);
                        }
                        None => {
                            tracing::warn!("Transport not found for key: {}", key);
                        }
                    }
                }
            }
        }));
        Ok(())
    }

    /// Start the HTTP server and listen for incoming requests. This method sets up the API routes,
    /// Start the HTTP server and listen for incoming requests. This method sets up the API routes,
    /// binds to the specified address, and starts serving the API using Axum + Aide.
    #[tracing::instrument(name = "Gateway", skip_all)]
    pub async fn start(mut self) -> Result<()> {
        aide::generate::extract_schemas(true);
        let address = self.address;
        let mut api = OpenApi::default();

        // --- Start cleaning up stale transports periodically.
        self.start_cleanup_task().await?;

        // --- Set up the API router with the routes.
        let ctx = Arc::new(self);
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
