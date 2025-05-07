use super::{Controller, MCP_SERVER_FINALIZER, MCP_SERVER_OPERATOR_MANAGER};
use crate::{Error, MCPServer, MCPServerPhase, MCPServerTransport, Result};
use chrono::Utc;
use futures::StreamExt;
use k8s_openapi::api::core::v1;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::api::{ObjectMeta, Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::runtime::finalizer;
use kube::runtime::finalizer::{Error as FinalizerError, Event};
use kube::runtime::{watcher::Config, Controller as RuntimeController};
use kube::{Api, ResourceExt};
use std::result::Result as StdResult;
use std::sync::Arc;
use std::time::Duration;
use tracing::error;

impl Controller {
    /// Creates a patch for the Pod resource based on the `MCPServer` spec.
    ///
    /// This function generates a patch for the Pod resource associated with the `MCPServer`.
    /// It sets the necessary environment variables and container specifications based on the server's
    /// configuration.
    ///
    /// # Arguments
    /// * `server` - A reference to the `MCPServer` object.
    /// * `pod` - The Pod object to be patched.
    ///
    /// # Returns
    /// * `Patch<v1::Pod>` - The generated patch for the Pod resource.
    ///
    /// # Errors
    /// * `Error::ServerPodTemplateError` - If there is an error creating the Pod patch.
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

    /// Check if the server is currently running.
    pub async fn is_server_running(&self, server: &MCPServer) -> Result<bool> {
        match self
            .get_server_pod(server)
            .await?
            .status
            .clone()
            .unwrap_or_default()
            .phase
        {
            Some(ref phase) if phase == "Running" => Ok(true),
            _ => Ok(false),
        }
    }

    /// Create the `v1::Pod` resource for the `MCPServer`.
    pub async fn start_server_pod(&self, server: &MCPServer) -> Result<v1::Pod> {
        Api::namespaced(self.get_client(), &self.get_namespace())
            .patch(
                &server.name_pod(),
                &PatchParams::apply(MCP_SERVER_OPERATOR_MANAGER),
                &self
                    .create_server_pod_patch(server, Default::default())
                    .await,
            )
            .await
            .map_err(Error::ServerPodTemplate)
    }

    /// Create the v1::Service resource for the `MCPServer`.
    pub async fn start_server_service(&self, server: &MCPServer) -> Result<Option<v1::Service>> {
        if server.spec.transport == MCPServerTransport::Stdio {
            return Ok(None);
        }
        Api::<v1::Service>::namespaced(self.get_client(), &self.get_namespace())
            .patch(
                &server.name_service(),
                &PatchParams::apply(MCP_SERVER_OPERATOR_MANAGER),
                &self
                    .create_server_service_patch(server, Default::default())
                    .await,
            )
            .await
            .map_err(Error::ServerServiceTemplate)
            .map(Some)
    }

    /// Delete the `v1::Pod` resource for the `MCPServer`.
    pub async fn delete_server_pod(&self, server: &MCPServer) -> Result<()> {
        Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace())
            .delete(&server.name_pod(), &Default::default())
            .await
            .map_err(Error::ServerPodDelete)?;
        Ok(())
    }

    /// Delete the `v1::Service` resource for the `MCPServer`.
    pub async fn delete_server_service(&self, server: &MCPServer) -> Result<()> {
        if server.spec.transport == MCPServerTransport::Stdio {
            return Ok(());
        }
        Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace())
            .delete(&server.name_service(), &Default::default())
            .await
            .map_err(Error::ServerServiceDelete)?;
        Ok(())
    }

    /// Determine if a server should be up based on pool limits and idle timeout
    pub async fn should_server_be_running(&self, server: &MCPServer) -> Result<bool> {
        let pool = self.get_server_pool(server).await?;

        // Check pool limits
        if let Some(pool_status) = &pool.status {
            // If managed_servers_count > max_servers_limit, server should not be up
            if pool_status.managed_servers_count > pool.spec.max_servers_limit {
                return Ok(false);
            }

            // If active_servers_count >= max_servers_active, server should not be up
            if pool_status.active_servers_count >= pool.spec.max_servers_active {
                return Ok(false);
            }
        }

        // Check idle timeout
        if let Some(server_status) = &server.status {
            if server_status.phase == MCPServerPhase::Running {
                if let Some(last_request) = &server_status.last_request_at {
                    // Get the relevant idle timeout (server's value or pool's default)
                    let idle_timeout = if server.spec.idle_timeout > 0 {
                        server.spec.idle_timeout
                    } else {
                        pool.spec.default_idle_timeout
                    };

                    // If elapsed time is greater than the idle timeout, server should not be up
                    let now = Utc::now();
                    let elapsed = now.signed_duration_since(*last_request);
                    let elapsed_secs = elapsed.num_seconds();
                    if elapsed_secs > idle_timeout as i64 {
                        return Ok(false);
                    }
                }
            }
        }

        // If none of the conditions to be down are met, the server should be up
        Ok(true)
    }

    /// Reconcile the MCPServer resource by checking its status and updating it accordingly.
    async fn reconcile_server(&self, server: &MCPServer) -> Result<Action> {
        let api = Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace());
        let obj = Arc::new(server.clone());

        // --- Handle the reconciliation process using finalizers to ensure
        // --- that the cleanup process is completed before the resource is deleted.
        let result: StdResult<Action, FinalizerError<Error>> =
            finalizer(&api, MCP_SERVER_FINALIZER, obj, {
                let server = server.clone();
                let controller = self.clone();
                move |event| async move {
                    match event {
                        Event::Cleanup { .. } => {
                            self.delete_server_pod(&server).await?;
                            self.delete_server_service(&server).await?;
                            Ok(Action::await_change())
                        }
                        Event::Apply { .. } => {
                            if controller.should_server_be_running(&server).await? {
                                self.start_server_pod(&server).await?;
                                self.start_server_service(&server).await?;
                            } else {
                                self.delete_server_pod(&server).await?;
                                self.delete_server_service(&server).await?;
                            }
                            Ok(Action::requeue(Duration::from_secs(60)))
                        }
                    }
                }
            })
            .await;

        match result {
            Ok(action) => Ok(action),
            Err(error) => {
                error!("[{}] {}", server.name_any(), error);
                Ok(Action::requeue(Duration::from_secs(5)))
            }
        }
    }

    /// Start the operator for managing MCPServer resources.
    pub async fn start_server_operator(&self) -> Result<()> {
        let ns = self.get_namespace();
        let wc = Config::default();

        // --- Create API clients for MCPServer, Pod, and Service.
        let api = Api::<MCPServer>::namespaced(self.get_client(), &ns);
        let api_pod = Api::<v1::Pod>::namespaced(self.get_client(), &ns);
        let api_services = Api::<v1::Service>::namespaced(self.get_client(), &ns);

        // --- Start the controller for MCPServer resources.
        RuntimeController::new(api, wc.clone())
            .owns(api_pod, wc.clone())
            .owns(api_services, wc.clone())
            .run(
                |server, controller| async move { controller.reconcile_server(&server).await },
                |_, _, _| Action::requeue(Duration::from_secs(5)),
                Arc::new(self.clone()),
            )
            .for_each(|result| async move {
                result.is_err().then(|| {
                    error!("Error in controller: {:?}", result);
                });
            })
            .await;

        Ok(())
    }
}
