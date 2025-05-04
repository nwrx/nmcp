use super::Controller;
use crate::MCPServer;
use k8s_openapi::api::core::v1;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::api::Patch;

impl Controller {
    /// Create a Patch for the v1::Service resource based on the `MCPServer` spec.
    ///
    /// This function prepares a Kubernetes Service resource based on the MCPServer specification.
    /// It configures service ports and other necessary Service settings.
    ///
    /// # Arguments
    /// * `server` - A reference to the MCPServer object.
    /// * `service` - A mutable reference to a Service object that will be patched.
    ///
    /// # Returns
    /// * `Result<Patch<v1::Service>>` - A patch object that can be applied to create or update the Service.
    ///
    /// # Errors
    /// * Returns an error if there is an issue preparing the patch.
    ///
    /// # Example
    /// ```
    /// let controller = Controller::new("default".to_string(), kube_client).await?;
    /// let service = v1::Service::default();
    /// let patch = controller.server_create_service_patch(&server, service).await?;
    /// ```
    pub async fn create_server_service_patch(
        &self,
        server: &MCPServer,
        mut service: v1::Service,
    ) -> Patch<v1::Service> {
        let mut ports: Vec<_> = Vec::new();
        let transport = server.spec.transport.clone();
        if transport.is_sse() {
            let port = transport.port.unwrap_or(3000);
            ports.push(v1::ServicePort {
                name: Some("http".to_string()),
                port,
                target_port: Some(IntOrString::Int(port)),
                protocol: Some("TCP".to_string()),
                ..Default::default()
            });
        }

        // Create service metadata
        service.metadata = kube::api::ObjectMeta {
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
}
