use super::{Controller, MCP_SERVER_OPERATOR_MANAGER};
use crate::{Error, MCPServer, MCPServerTransport, Result};
use crate::{MCPServerConditionType as Condition, MCPServerPhase as Phase};
use k8s_openapi::api::core::v1;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::api::{ObjectMeta, Patch, PatchParams};
use kube::Api;
use tracing::info;

impl Controller {
    /// Request the server to start
    #[allow(clippy::style)]
    pub async fn request_start(&self, server: &MCPServer) -> Result<()> {
        // --- Check if the server is already running.
        // if server.status.clone().unwrap_or_default().phase == Phase::Running {
        //     info!("Server is already running");
        //     return Ok(());
        // }

        // --- Start the server pod.
        self.set_server_phase(server, Phase::Starting).await?;
        self.set_server_condition(server, Condition::PodStarting, None)
            .await?;
        info!("Starting server pod");
        match self.start_server_pod(server).await {
            Ok(_) => {
                self.set_server_condition(server, Condition::PodRunning, None)
                    .await?;
                self.set_server_phase(server, Phase::Running).await?;
            }
            Err(error) => {
                info!("Error starting server pod: {:?}", error);
                self.set_server_condition(server, Condition::PodError, Some(error.to_string()))
                    .await?;
                self.set_server_phase(server, Phase::Error).await?;
            }
        }

        // --- Start the service.
        self.set_server_condition(server, Condition::ServiceStarting, None)
            .await?;
        match self.start_server_service(server).await {
            Ok(_) => {
                self.set_server_condition(server, Condition::ServiceReady, None)
                    .await?;
                self.set_server_phase(server, Phase::Running).await?;
            }
            Err(e) => {
                info!("Error starting server service: {:?}", e);
                self.set_server_condition(server, Condition::ServiceError, Some(e.to_string()))
                    .await?;
                self.set_server_phase(server, Phase::Error).await?;
            }
        }

        // --- Set the server to running.
        self.set_server_phase(server, Phase::Running).await?;
        self.set_server_condition(server, Condition::Ready, None)
            .await?;
        Ok(())
    }

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

    /// Retrieves the Kubernetes Service associated with a given MCPServer.
    pub async fn get_server_service(&self, server: &MCPServer) -> Result<v1::Service> {
        Api::namespaced(self.get_client(), &self.get_namespace())
            .get(&server.name_service())
            .await
            .map_err(Error::ServerServiceNotFound)
    }

    /// Retrieves the Kubernetes Pod associated with a given MCPServer.
    pub async fn get_server_pod(&self, server: &MCPServer) -> Result<v1::Pod> {
        Api::<v1::Pod>::namespaced(self.get_client(), &self.get_namespace())
            .get(&server.name_pod())
            .await
            .map_err(Error::ServerPodNotFound)
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
}

#[cfg(test)]
mod tests {
    use crate::{MCPServerSpec, MCPServerTransport, TestContext};
    use kube::ResourceExt;

    ///////////////////////////////////////////////////////////////////////

    /// Should return a server pod when it exists.
    #[tokio::test]
    async fn test_get_server_pod() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let server = controller.create_server("s1", Default::default()).await?;
                controller.start_server_pod(&server).await?;
                let pod = controller.get_server_pod(&server).await?;
                assert_eq!(pod.name_any(), server.name_pod());
                Ok(())
            })
            .await
            .unwrap();
    }

    /// Should reject with `Error::ServerPodNotFound` when the server pod does not exist.
    #[tokio::test]
    async fn test_get_server_pod_not_found() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let server = controller.create_server("s1", Default::default()).await?;
                let result = controller.get_server_pod(&server).await;
                assert!(result.is_err());
                Ok(())
            })
            .await
            .unwrap();
    }

    ///////////////////////////////////////////////////////////////////////

    /// Should return a server service when it exists.
    #[tokio::test]
    async fn test_get_server_service() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let spec = MCPServerSpec {
                    transport: MCPServerTransport::Sse { port: 8080 },
                    ..Default::default()
                };
                let server = controller.create_server("s1", spec).await?;
                controller.start_server_service(&server).await?;
                let service = controller.get_server_service(&server).await?;
                assert_eq!(service.name_any(), server.name_service());
                Ok(())
            })
            .await
            .unwrap();
    }

    /// Should reject with `Error::ServerServiceNotFound` when the server service does not exist.
    #[tokio::test]
    async fn test_get_server_service_not_found() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let server = controller.create_server("s1", Default::default()).await?;
                let result = controller.get_server_service(&server).await;
                assert!(result.is_err());
                Ok(())
            })
            .await
            .unwrap();
    }
}
