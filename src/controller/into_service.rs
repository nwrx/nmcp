use super::IntoResource;
use crate::{MCPServer, MCPServerTransport};
use k8s_openapi::api::core::v1;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::{api::ObjectMeta, ResourceExt};

impl IntoResource<v1::Service> for MCPServer {
    /// Returns the name of the `v1::Service` for the `MCPServer`.
    fn resource_name(&self) -> String {
        format!(
            "mcp-svc-{}-{}-{}",
            self.spec.pool,
            self.name_any(),
            &self.metadata.uid.clone().unwrap()[..8]
        )
    }

    /// Create a Patch for the `v1::Service` resource based on the `MCPServer` spec.
    fn resource(&self) -> v1::Service {
        let mut service = v1::Service::default();

        let mut ports: Vec<v1::ServicePort> = Vec::new();
        match self.spec.transport {
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

        // --- Create service metadata and spec.
        service.metadata = ObjectMeta {
            name: Some(<Self as IntoResource<v1::Service>>::resource_name(self)),
            namespace: self.metadata.namespace.clone(),
            labels: Some(self.labels().clone()),
            ..Default::default()
        };
        service.spec = Some(v1::ServiceSpec {
            selector: Some(self.labels().clone()),
            ports: Some(ports),
            type_: Some("ClusterIP".to_string()),
            ..v1::ServiceSpec::default()
        });

        // Create the Patch<v1::Service> object and return it.
        service
    }
}
