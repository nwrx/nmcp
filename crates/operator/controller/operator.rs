use super::{Controller, MCP_SERVER_FINALIZER, MCP_SERVER_OPERATOR_MANAGER};
use crate::{Error, MCPServer, Result};
use crate::{MCPServerConditionType as Condition, MCPServerPhase as Phase};
use crate::{MCPServerTransport, MCPServerTransportStdio, DEFAULT_POD_BUFFER_SIZE};
use chrono::Utc;
use futures::StreamExt;
use k8s_openapi::api::core::v1;
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::api::AttachParams;
use kube::api::{ObjectMeta, Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::runtime::finalizer;
use kube::runtime::finalizer::Event;
use kube::runtime::{watcher::Config, Controller as RuntimeController};
use kube::{Api, ResourceExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

pub enum PodStatus {
    Running,
    Pending,
    Succeeded,
    Failed,
    Unknown,
    NotFound,
}

impl Controller {
    /// Creates a patch for the Pod resource based on the `MCPServer` spec.
    async fn create_server_pod_patch(
        &self,
        server: &MCPServer,
        mut pod: v1::Pod,
    ) -> Patch<v1::Pod> {
        let mut env = server.spec.env.clone();
        env.push(v1::EnvVar {
            name: "MCP_SERVER_NAME".to_string(),
            value: Some(server.name_pod()),
            ..Default::default()
        });
        env.push(v1::EnvVar {
            name: "MCP_SERVER_UUID".to_string(),
            value: server.metadata.uid.clone(),
            ..Default::default()
        });
        env.push(v1::EnvVar {
            name: "MCP_SERVER_POOL".to_string(),
            value: Some(server.spec.pool.clone()),
            ..Default::default()
        });

        // --- Create container ports if transport is "SSE"
        let mut container_ports = Vec::new();
        match server.spec.transport {
            MCPServerTransport::Sse { port } => {
                container_ports.push(v1::ContainerPort {
                    name: Some("http".to_string()),
                    container_port: port.into(),
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                });
            }
            MCPServerTransport::Stdio => {
                // No ports needed for stdio transport
            }
        }

        // --- Create container
        let container = v1::Container {
            name: "server".to_string(),
            image: Some(server.spec.image.clone()),
            command: server.spec.command.clone(),
            args: server.spec.args.clone(),
            env: Some(env),
            ports: Some(container_ports),
            stdin: Some(true),
            tty: Some(true),
            // resources,
            // security_context,
            ..Default::default()
        };

        // Update pod metadata
        pod.metadata = ObjectMeta {
            name: Some(server.name_pod()),
            namespace: Some(self.namespace.clone()),
            labels: Some(server.labels()),
            ..Default::default()
        };

        pod.spec = Some(v1::PodSpec {
            containers: vec![container],
            restart_policy: Some("Always".to_string()),
            ..Default::default()
        });

        // Create the Patch<Pod> object and return it.
        Patch::Apply(pod)
    }

    /// Create a Patch for the v1::Service resource based on the `MCPServer` spec.
    async fn create_server_service_patch(
        &self,
        server: &MCPServer,
        mut service: v1::Service,
    ) -> Patch<v1::Service> {
        let mut ports: Vec<v1::ServicePort> = Vec::new();
        match server.spec.transport {
            MCPServerTransport::Sse { port } => {
                ports.push(v1::ServicePort {
                    name: Some("http".to_string()),
                    port: port.into(),
                    target_port: Some(IntOrString::Int(port.into())),
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                });
            }
            // No ports needed for stdio transport
            MCPServerTransport::Stdio => {}
        }

        // Create service metadata
        service.metadata = ObjectMeta {
            name: Some(server.name_service()),
            namespace: Some(self.namespace.clone()),
            labels: Some(server.labels()),
            ..Default::default()
        };
        service.spec = Some(v1::ServiceSpec {
            selector: Some(server.labels()),
            ports: Some(ports),
            type_: Some("ClusterIP".to_string()),
            ..v1::ServiceSpec::default()
        });

        // Create the Patch<v1::Service> object and return it.
        Patch::Apply(service)
    }

    /// Create the v1::Pod resource for the `MCPServer`.
    #[tracing::instrument(name = "PatchPod", skip_all, err)]
    pub async fn patch_server_pod(&self, server: &MCPServer) -> Result<()> {
        tracing::debug!("Creating pod {}", server.name_pod());
        Api::<v1::Pod>::namespaced(self.get_client(), &self.get_namespace())
            .patch(
                &server.name_pod(),
                &PatchParams::apply(MCP_SERVER_OPERATOR_MANAGER),
                &self
                    .create_server_pod_patch(server, Default::default())
                    .await,
            )
            .await
            .map_err(Error::from)?;
        Ok(())
    }

    /// Create the v1::Service resource for the `MCPServer`.
    #[tracing::instrument(name = "PatchService", skip_all, err)]
    pub async fn patch_server_service(&self, server: &MCPServer) -> Result<()> {
        if server.spec.transport == MCPServerTransport::Stdio {
            return Ok(());
        }
        tracing::debug!("Creating service {}", server.name_service());
        Api::<v1::Service>::namespaced(self.get_client(), &self.get_namespace())
            .patch(
                &server.name_service(),
                &PatchParams::apply(MCP_SERVER_OPERATOR_MANAGER),
                &self
                    .create_server_service_patch(server, Default::default())
                    .await,
            )
            .await
            .map_err(Error::from)?;
        Ok(())
    }

    /// Delete the `v1::Pod` resource for the `MCPServer`.
    #[tracing::instrument(name = "DeletePod", skip_all, err)]
    pub async fn delete_server_pod(&self, server: &MCPServer) -> Result<()> {
        let result = Api::<v1::Pod>::namespaced(self.get_client(), &self.get_namespace())
            .delete(&server.name_pod(), &Default::default())
            .await
            .map_err(Error::from);

        // --- Check the result of the pod deletion.
        match result {
            Ok(..) => {
                let condition = Condition::PodTerminating;
                self.set_server_status(server, condition).await?;
            }
            Err(Error::KubeError(kube::Error::Api(error))) if error.code == 404 => {}
            Err(error) => {
                let message = format!("Failed to delete pod: {error}");
                let condition = Condition::PodTerminationFailed(message);
                self.set_server_status(server, condition).await?;
                return Err(error);
            }
        }

        Ok(())
    }

    /// Delete the `v1::Service` resource for the `MCPServer`.
    #[tracing::instrument(name = "DeleteService", skip_all)]
    pub async fn delete_server_service(&self, server: &MCPServer) -> Result<()> {
        if server.spec.transport == MCPServerTransport::Stdio {
            return Ok(());
        }
        Api::<v1::Service>::namespaced(self.get_client(), &self.get_namespace())
            .delete(&server.name_service(), &Default::default())
            .await
            .map_err(Error::from)?;
        Ok(())
    }

    /// Retrieves the Kubernetes Service associated with a given MCPServer.
    #[tracing::instrument(name = "GetServerService", skip_all)]
    pub async fn get_server_service(&self, server: &MCPServer) -> Result<v1::Service> {
        Api::namespaced(self.get_client(), &self.get_namespace())
            .get(&server.name_service())
            .await
            .map_err(Error::from)
    }

    /// Retrieves the Kubernetes Pod associated with a given MCPServer.
    pub async fn get_server_pod(&self, server: &MCPServer) -> Result<v1::Pod> {
        Api::<v1::Pod>::namespaced(self.get_client(), &self.get_namespace())
            .get(&server.name_pod())
            .await
            .map_err(Error::from)
    }

    /// Get the pod status for the given MCPServer.
    pub async fn get_server_pod_status(&self, server: &MCPServer) -> Result<PodStatus> {
        match self.get_server_pod(server).await {
            Ok(pod) => {
                let phase = pod.status.unwrap_or_default().phase.unwrap_or_default();
                match phase.as_str() {
                    "Running" => Ok(PodStatus::Running),
                    "Pending" => Ok(PodStatus::Pending),
                    "Succeeded" => Ok(PodStatus::Succeeded),
                    "Failed" => Ok(PodStatus::Failed),
                    _ => Ok(PodStatus::Unknown),
                }
            }
            Err(Error::KubeError(kube::Error::Api(error))) if error.code == 404 => {
                Ok(PodStatus::NotFound)
            }
            Err(error) => Err(error),
        }
    }

    /// Start the server pod and service for the given MCPServer.
    #[tracing::instrument(name = "Up", skip_all)]
    pub async fn ensure_server_up(&self, server: &MCPServer) -> Result<()> {
        tracing::debug!("Ensuring server is up {}", server.name_any());

        // --- Check if the server is already running.
        match self.get_server_pod_status(server).await? {
            PodStatus::Running => {
                tracing::debug!("Server is already running");
            }
            PodStatus::NotFound => {
                self.patch_server_pod(server).await?;
            }
            PodStatus::Pending => {
                tracing::info!("Server Pod is pending");
                let condition = Condition::PodPending;
                self.set_server_status(server, condition).await?;
            }
            PodStatus::Succeeded => {
                tracing::info!("Server Pod Succeeded");
                let condition = Condition::PodTerminated;
                self.set_server_status(server, condition).await?;
            }
            PodStatus::Failed => {
                tracing::error!("Server Pod is in an error state");
                let condition = Condition::PodFailed("Pod is in an error state".to_string());
                self.set_server_status(server, condition).await?;
            }
            PodStatus::Unknown => {
                tracing::error!("Server Pod is in an unknown state");
                let condition = Condition::PodFailed("Pod is in an unknown state".to_string());
                self.set_server_status(server, condition).await?;
            }
        }
        // self.start_server_service(server).await?;

        self.set_server_status(server, Condition::Running).await?;
        Ok(())
    }

    /// Stop the server pod and service for the given MCPServer.
    #[tracing::instrument(name = "Shutdown", skip_all, err)]
    pub async fn ensure_server_down(&self, server: &MCPServer) -> Result<()> {
        tracing::debug!("Ensuring server is down");
        let server = self.get_server_by_name(&server.name_any()).await?;
        let phase = server.status.clone().unwrap().phase;

        // --- Track the pod status and delete it if necessary.
        match self.get_server_pod_status(&server).await? {
            PodStatus::NotFound => {
                tracing::debug!("Pod not found, nothing to do");
                if phase != Phase::Idle {
                    let condition = Condition::PodTerminated;
                    self.set_server_status(&server, condition).await?;
                }
            }
            _ => {
                tracing::debug!("Pod exists, deleting");
                self.delete_server_pod(&server).await?;
                let condition = Condition::PodTerminating;
                self.set_server_status(&server, condition).await?;
            }
        }

        // --- Finally, set the server status to "Idle" and clean up conditions.
        if phase != Phase::Idle {
            self.set_server_status(&server, Condition::Idle).await?;
            self.cleanup_server_conditions(&server).await?;
        }
        Ok(())
    }

    /// Requests the server to start.
    #[tracing::instrument(name = "RequestStart", skip_all, fields(server = %server.name_any()))]
    pub async fn request_server_up(&self, server: &MCPServer) -> Result<()> {
        loop {
            let server = self.get_server_by_name(&server.name_any()).await?;
            let phase = server.status.clone().unwrap_or_default().phase;
            match phase {
                Phase::Idle => {
                    tracing::info!("Requesting server start");
                    let condition = Condition::Requested;
                    self.set_server_status(&server, condition).await?;
                }
                Phase::Failed => {
                    Err(Error::Internal("Server is in an error state".to_string()))?;
                }
                Phase::Running => {
                    tracing::info!("Server running, we can proceed");
                    return Ok(());
                }
                Phase::Stopping => {
                    tracing::info!("Server is stopping, cannot start");
                    Err(Error::Internal(
                        "Server is stopping, pending start".to_string(),
                    ))?;
                }
                Phase::Requested => tracing::info!("Server requested, pending start"),
                Phase::Starting => tracing::info!("Server is starting, waiting for it to be ready"),
            }
            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Attaches to a pod's TTY and returns a stream of events for SSE.
    #[tracing::instrument(name = "GetServerTransport", skip_all)]
    pub async fn get_server_transport(
        &self,
        server: &MCPServer,
    ) -> Result<Arc<RwLock<MCPServerTransportStdio>>> {
        // --- Check if the transport already exists for this server.
        if let Some(transport) = self.transports.read().await.get(&server.name_any()) {
            return Ok(transport.clone());
        }

        // --- Create a new channel if none exists.
        match server.spec.transport {
            MCPServerTransport::Sse { .. } => Err(Error::Internal(
                "SSE transport is not supported yet".to_string(),
            )),

            // --- Create a new transport that will proxy the pod's TTY to a BroadcastStream.
            // --- This will allow us to send and receive messages from the pod via SSE.
            MCPServerTransport::Stdio => {
                let process = Api::<Pod>::namespaced(self.get_client(), &self.get_namespace())
                    .attach(
                        &server.name_pod(),
                        &AttachParams::interactive_tty()
                            .container("server")
                            .max_stdout_buf_size(DEFAULT_POD_BUFFER_SIZE)
                            .max_stdin_buf_size(DEFAULT_POD_BUFFER_SIZE),
                    )
                    .await
                    .map_err(Error::KubeError)?;

                // --- Create a new transport attached to the pod's TTY.
                let transport = MCPServerTransportStdio::new(process, server.clone());
                let transport = Arc::new(RwLock::new(transport));
                self.transports
                    .write()
                    .await
                    .insert(server.name_any(), transport.to_owned());

                // --- Return the Arc of the transport
                Ok(transport)
            }
        }
    }

    /// Determine if the server should be started based on its status and pool limits.
    #[tracing::instrument(name = "ShouldBeUp", skip_all)]
    pub async fn can_server_be_up(&self, server: &MCPServer) -> Result<bool> {
        let pool = self.get_pool_by_name(&server.spec.pool).await?;
        let pool_status = pool.status.clone().unwrap_or_default();

        // --- If active_servers_count >= max_servers_active, server should not be up
        if pool_status.active_servers_count >= pool.spec.max_servers_active {
            tracing::info!(
                "Pool active servers limit reached: {} >= {}",
                pool_status.active_servers_count,
                pool.spec.max_servers_active
            );
            return Ok(false);
        }

        // --- If the server is not "Idle", it should be started
        Ok(true)
    }

    /// Determine if the server should be shutdown based on its status and idle timeout.
    #[tracing::instrument(name = "ShouldBeDown", skip_all)]
    pub async fn should_server_be_down(&self, server: &MCPServer) -> Result<bool> {
        let status = server.status.clone().unwrap_or_default();
        let pool = self.get_pool_by_name(&server.spec.pool).await?;

        // --- Check idle timeout
        if let Some(last_request) = &status.last_request_at {
            // Get the relevant idle timeout (server's value or pool's default)
            let idle_timeout = if server.spec.idle_timeout > 0 {
                server.spec.idle_timeout
            } else {
                pool.spec.default_idle_timeout
            };

            // If elapsed time is greater than the idle timeout, server should not be up
            let now = Utc::now();
            let elapsed = now.signed_duration_since(*last_request).to_std().unwrap();
            let elapsed_secs = elapsed.as_secs() as i64;
            if elapsed_secs > idle_timeout as i64 {
                tracing::info!(
                    "Server idle timeout reached, shutting down server {}",
                    server.name_any()
                );
                return Ok(true);
            } else {
                let remaining = idle_timeout as i64 - elapsed_secs;
                tracing::info!(
                    "Server idle timeout remaining: {}s, server {}",
                    remaining,
                    server.name_any()
                );
            }
        }

        // --- If all checks pass, the server should be up.
        Ok(false)
    }

    /// Reconcile the MCPServer resource by checking its status and updating it accordingly.
    #[tracing::instrument(name = "Reconcile", skip_all, fields(server = %server.name_any()))]
    async fn reconcile(&self, server: Arc<MCPServer>) -> Result<Action> {
        let api = Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace());

        // --- Handle the reconciliation process using finalizers to ensure
        // --- that the cleanup process is completed before the resource is deleted.
        finalizer(&api, MCP_SERVER_FINALIZER, server, {
            let controller = self.clone();
            move |event| async move {
                match event {
                    Event::Cleanup(server) => {
                        self.ensure_server_down(&server).await?;
                        Ok(Action::requeue(Duration::from_secs(5)))
                    }
                    Event::Apply(server) => {
                        let phase = server.status.clone().unwrap_or_default().phase;
                        match phase {
                            Phase::Idle => {
                                tracing::info!("Server is idle, ensuring it is down",);
                                self.ensure_server_down(&server).await?;
                                Ok(Action::requeue(Duration::from_secs(5)))
                            }
                            Phase::Stopping => {
                                tracing::info!("Server is stopping, ensuring it is down");
                                self.ensure_server_down(&server).await?;
                                Ok(Action::requeue(Duration::from_secs(5)))
                            }
                            Phase::Failed => {
                                tracing::info!("Server is in error state, ensuring it is down");
                                self.ensure_server_down(&server).await?;
                                Ok(Action::requeue(Duration::from_secs(5)))
                            }
                            Phase::Requested => {
                                tracing::info!("Server is Requested, checking if it should be up");
                                if controller.can_server_be_up(&server).await? {
                                    self.ensure_server_up(&server).await?;
                                } else {
                                    self.ensure_server_down(&server).await?;
                                }
                                Ok(Action::requeue(Duration::from_secs(5)))
                            }
                            Phase::Starting => {
                                tracing::info!("Server is Starting, checking if it should be up");
                                if controller.can_server_be_up(&server).await? {
                                    self.ensure_server_up(&server).await?;
                                } else if controller.should_server_be_down(&server).await? {
                                    self.ensure_server_down(&server).await?;
                                }
                                Ok(Action::await_change())
                            }
                            Phase::Running => {
                                tracing::debug!("Server is Running, checking if it should be down");
                                if controller.should_server_be_down(&server).await? {
                                    self.ensure_server_down(&server).await?;
                                } else {
                                    self.ensure_server_up(&server).await?;
                                }
                                Ok(Action::requeue(Duration::from_secs(5)))
                            }
                        }
                    }
                }
            }
        })
        .await
        .map_err(Error::from)
    }

    /// Handle an error during the reconciliation process.
    #[tracing::instrument(name = "ErrorPolicy", skip_all)]
    fn error_policy(&self, _server: &MCPServer, error: &Error) -> Result<Action> {
        tracing::error!("{}", error.to_message());
        Ok(Action::requeue(Duration::from_secs(5)))
    }

    /// Start the operator for managing MCPServer resources.
    #[tracing::instrument(name = "Operator", skip_all, err)]
    pub async fn start_server_operator(&self) -> Result<()> {
        let ns = self.get_namespace();
        let wc = Config::default();

        // --- Create API clients for MCPServer, Pod, and Service.
        let api = Api::<MCPServer>::namespaced(self.get_client(), &ns);
        let api_pod = Api::<v1::Pod>::namespaced(self.get_client(), &ns);
        let api_services = Api::<v1::Service>::namespaced(self.get_client(), &ns);

        // --- Start the controller for MCPServer resources.
        tracing::info!("Starting");
        RuntimeController::new(api, wc.clone())
            .owns(api_pod, wc.clone())
            .owns(api_services, wc.clone())
            .run(
                |server, controller| async move { controller.reconcile(server).await },
                |server, error, controller| controller.error_policy(&server, error).unwrap(),
                Arc::new(self.clone()),
            )
            // --- Consume each event from the controller.
            .for_each(async move |event| match event {
                Ok(..) => {
                    sleep(Duration::from_secs(5)).await;
                }
                Err(error) => {
                    tracing::error!("Error: {}", error.to_string());
                    sleep(Duration::from_secs(5)).await;
                }
            })
            .await;

        Ok(())
    }
}
