use super::Controller;
use crate::MCPServer;
use k8s_openapi::api::core::v1;
use kube::api::{ObjectMeta, Patch};

impl Controller {
    /// Creates a patch for the Pod resource based on the `MCPServer` spec.
    ///
    /// This function prepares a Kubernetes Pod resource based on the MCPServer specification.
    /// It configures environment variables, container ports, and other necessary Pod settings.
    ///
    /// # Arguments
    /// * `pod` - A mutable reference to a Pod object that will be patched.
    ///
    /// # Returns
    /// * `Result<Patch<v1::Pod>>` - A patch object that can be applied to create or update the Pod.
    ///
    /// # Errors
    /// * Returns an error if there is an issue preparing the patch.
    pub async fn server_create_pod_patch(
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

        // Create container ports if transport is "SSE"
        let mut container_ports = Vec::new();
        let transport = server.spec.transport.clone();
        if transport.is_sse() {
            let port = transport.port.unwrap_or(3000);
            container_ports.push(v1::ContainerPort {
                name: Some("http".to_string()),
                container_port: port,
                protocol: Some("TCP".to_string()),
                ..v1::ContainerPort::default()
            });
        }

        // Create container
        let container = v1::Container {
            name: "mcp-server".to_string(),
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
}
