use super::IntoResource;
use crate::{MCPServer, MCPServerTransport, MCP_SERVER_CONTAINER_NAME};
use k8s_openapi::api::core::v1;
use kube::{api::ObjectMeta, ResourceExt};

impl IntoResource<v1::Pod> for MCPServer {
    /// Returns the name of the `v1::Pod` for the `MCPServer`.
    fn resource_name(&self) -> String {
        format!(
            "mcp-pod-{}-{}-{}",
            self.spec.pool,
            self.name_any(),
            &self.metadata.uid.clone().unwrap()[..8]
        )
    }

    // /// Returns the labels for the `MCPServer`.
    // pub fn labels(&self) -> BTreeMap<String, String> {
    //     let uid = self.metadata.uid.clone().unwrap_or_default();
    //     let mut labels = std::collections::BTreeMap::new();
    //     labels.insert("app".to_string(), self.name_pod());
    //     labels.insert("nmcp.nwrx.io/uid".to_string(), uid);
    //     labels.insert("nmcp.nwrx.io/pool".to_string(), self.spec.pool.clone());
    //     labels
    // }

    fn resource(&self) -> v1::Pod {
        let mut pod = v1::Pod::default();

        // --- Set the pod metadata.
        let mut env = self.spec.env.clone();
        env.push(v1::EnvVar {
            name: "MCP_SERVER_NAME".to_string(),
            value: Some(self.name_any()),
            ..Default::default()
        });
        env.push(v1::EnvVar {
            name: "MCP_SERVER_UUID".to_string(),
            value: self.metadata.uid.clone(),
            ..Default::default()
        });
        env.push(v1::EnvVar {
            name: "MCP_SERVER_POOL".to_string(),
            value: Some(self.spec.pool.clone()),
            ..Default::default()
        });

        // --- Create container ports if transport is "SSE"
        let mut container_ports = Vec::new();
        match self.spec.transport {
            MCPServerTransport::Stdio => {
                // No ports needed for stdio transport
            }
            MCPServerTransport::Sse { port } => {
                container_ports.push(v1::ContainerPort {
                    name: Some("http".to_string()),
                    container_port: port.into(),
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                });
            }
            MCPServerTransport::StreamableHttp { port } => {
                container_ports.push(v1::ContainerPort {
                    name: Some("http".to_string()),
                    container_port: port.into(),
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                });
            }
        }

        // Define startup probe to ensure the process is fully initialized
        let startup_probe = match self.spec.transport {
            MCPServerTransport::Stdio => None,
            MCPServerTransport::Sse { .. } => None,
            MCPServerTransport::StreamableHttp { .. } => None,
        };

        // Define readiness probe to ensure container is ready for connections
        let readiness_probe = match self.spec.transport {
            MCPServerTransport::Stdio => None,
            MCPServerTransport::Sse { .. } => None,
            MCPServerTransport::StreamableHttp { .. } => None,
        };

        // --- Create container
        let container = v1::Container {
            name: MCP_SERVER_CONTAINER_NAME.to_string(),
            image: Some(self.spec.image.clone()),
            command: self.spec.command.clone(),
            args: self.spec.args.clone(),
            env: Some(env),
            ports: Some(container_ports),
            stdin: Some(true),
            tty: Some(false),
            startup_probe,
            readiness_probe,
            // resources,
            // security_context,
            ..Default::default()
        };

        // Update pod metadata
        pod.metadata = ObjectMeta {
            name: Some(<Self as IntoResource<v1::Pod>>::resource_name(self)),
            namespace: self.clone().metadata.namespace,
            labels: Some(self.labels().clone()),
            ..Default::default()
        };

        pod.spec = Some(v1::PodSpec {
            containers: vec![container],
            restart_policy: Some("Always".to_string()),
            termination_grace_period_seconds: Some(10),
            share_process_namespace: Some(true),
            ..Default::default()
        });

        // Create the Patch<Pod> object and return it.
        pod
    }
}
