use k8s_openapi::api::core::v1::{Container, ContainerPort, EnvVar, Pod, PodSpec, Service, ServicePort, ServiceSpec};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::{api::{ObjectMeta, Patch}, ResourceExt};
use crate::utils::Result;
use super::MCPServer;

/// The finalizer name for the MCPServer resource.
pub const MCP_SERVER_FINALIZER: &str = "mcpserver.unmcp.io/finalizer";

impl MCPServer {

    /// Returns the name of the Service for the MCPServer.
    pub fn name_service(&self) -> String {
        format!(
					"mcp-server-svc-{}-{}-{}",
					self.spec.pool,
					self.name_any(),
					&self.metadata.uid.clone().unwrap()[..8]
				)
    }

    /// Returns the name of the Pod for the MCPServer.
    pub fn name_pod(&self) -> String {
        format!(
					"mcp-server-{}-{}-{}",
					self.spec.pool,
					self.name_any(),
					&self.metadata.uid.clone().unwrap()[..8]
				)
    }

    /// Returns the labels for the MCPServer.
    pub fn labels(&self) -> std::collections::BTreeMap<String, String> {
        let transport = self.spec.transport.clone().unwrap_or_default();
        let mut labels = std::collections::BTreeMap::new();
        labels.insert("app".to_string(), self.name_pod());
        labels.insert("unmcp.dev/server".to_string(), self.name_any());
        labels.insert("unmcp.dev/pool".to_string(), self.spec.pool.clone());
        labels.insert("unmcp.dev/transport".to_string(), transport.to_string());
        labels.insert("unmcp.dev/uid".to_string(), self.metadata.uid.clone().unwrap_or_default());
        labels
    }

	/// Creates a patch for the Pod resource based on the MCPServer spec.
	pub fn into_patch_pod(&self, mut pod: Pod) -> Result<Patch<Pod>> {
		
		// --- Transfer environment variables from the server spec to the pod spec.
		let mut env = Vec::new();
		for env_var in self.spec.env.clone().iter() {
			env.push(EnvVar::from(env_var.clone()));
		}
		
		// --- Add a unique server ID, pod name, and pool name to the environment variables
		env.push(EnvVar {
			name: "MCP_SERVER_NAME".to_string(),
			value: Some(self.name_any()),
			..EnvVar::default()
		});
		env.push(EnvVar {
			name: "MCP_SERVER_UUID".to_string(),
			value: self.metadata.uid.clone(),
			..EnvVar::default()
		});
		env.push(EnvVar {
			name: "MCP_SERVER_POOL".to_string(),
			value: Some(self.spec.pool.clone()),
			..EnvVar::default()
		});
		
		// --- Create container ports if transport is "SSE"
		let mut container_ports = Vec::new();
		let transport = self.spec.transport.clone().unwrap();
		if transport.is_sse() {
			let port = transport.port.unwrap_or(3000);
			container_ports.push(ContainerPort {
				name: Some("http".to_string()),
				container_port: port,
				protocol: Some("TCP".to_string()),
				..ContainerPort::default()
			});
		}

		// Create container
		let container = Container {
			name: "mcp-server".to_string(),
			image: Some(self.spec.image.clone()),
			command: self.spec.command.clone(),
			args: self.spec.args.clone(),
			env: Some(env),
			ports: Some(container_ports),
			// resources,
			// security_context,
			..Default::default()
		};
		
		// Update pod metadata
		pod.metadata = ObjectMeta {
			name: Some(self.name_pod()),
			namespace: self.namespace(),
			labels: Some(self.labels()),
			..Default::default()
		};
		pod.spec = Some(PodSpec {
			containers: vec![container],
			restart_policy: Some("Always".to_string()),
			..Default::default()
		});

		// --- Create the Patch<Pod> object and return it.
		Ok(Patch::Apply(pod))
	}

    /// Create a Patch for the Service resource based on the MCPServer spec.
    pub fn into_patch_service(&self, mut service: Service) -> Result<Patch<Service>> {

        let mut ports: Vec<_> = Vec::new();
        let transport = self.spec.transport.clone().unwrap();
        if transport.is_sse() {
            let port = transport.port.unwrap_or(3000);
            ports.push(ServicePort {
                name: Some("http".to_string()),
                port,
                target_port: Some(IntOrString::Int(port)),
                protocol: Some("TCP".to_string()),
                ..Default::default()
            });
        }

        // --- Create service metadata
        service.metadata = kube::api::ObjectMeta {
            name: Some(self.name_service()),
            namespace: self.namespace(),
            labels: Some(self.labels()),
            ..Default::default()
        };
        service.spec = Some(ServiceSpec {
            selector: Some(self.labels()),
            ports: Some(ports),
            type_: Some("ClusterIP".to_string()),
            ..ServiceSpec::default()
        });
        
        // --- Create the Patch<Service> object and return it.
        Ok(Patch::Apply(service))
    }
}
