use std::collections::BTreeMap;
use k8s_openapi::api::core::v1::{Service, ServiceSpec, ServicePort};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::ResourceExt as _;

use crate::utils::Result;
use crate::server::server_crd::MCPServer;
use crate::server::server_controller::MCPServerController;

impl MCPServerController {

    /// Create a Service for a MCPServer resource. This will take the server's
    /// specifications and create a Service object that can be used to expose
    /// the server's pods to the network.
    /// 
    /// # Arguments
    /// * `server` - The MCPServer resource for which the Service is being created.
    /// * `service_name` - The name of the Service to be created.
    /// * `pod_name` - The name of the Pod that the Service will target.
    /// * `namespace` - The namespace in which the Service will be created.
    ///
    /// # Returns
    /// A Result containing the created Service object or an error if the creation fails.
    ///
    pub fn create_service(server: &MCPServer, service_name: &str, pod_name: &str, namespace: &str) -> Result<Service> {

        // --- Create labels
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), pod_name.to_string());
        labels.insert("unmcp.dev/server".to_string(), server.name_any());
        if !server.spec.pool.is_empty() {
            labels.insert("unmcp.dev/pool".to_string(), server.spec.pool.clone());
        }
        
        // --- Create selector
        let mut selector = BTreeMap::new();
        selector.insert("app".to_string(), pod_name.to_string());
        
        // --- Create service metadata
        let metadata = kube::api::ObjectMeta {
            name: Some(service_name.to_string()),
            namespace: Some(namespace.to_string()),
            labels: Some(labels),
            ..Default::default()
        };

        let ports = vec![ServicePort {
            name: Some("http".to_string()),
            port: server.spec.networking.port,
            target_port: Some(IntOrString::Int(server.spec.networking.port)),
            ..ServicePort::default()
        }];

        let service_type = if server.spec.networking.expose_externally {
            Some("LoadBalancer".to_string())
        } else {
            Some("ClusterIP".to_string())
        };

        let spec = ServiceSpec {
            selector: Some(selector),
            ports: Some(ports),
            type_: service_type,
            ..ServiceSpec::default()
        };

        // --- Create the Service object and return it.
        Ok(Service {
            metadata,
            spec: Some(spec),
            ..Service::default()
        })
    }
}
