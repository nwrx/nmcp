use tokio::sync::RwLock;
use tracing::info;
use kube::api::{Api, PostParams, ResourceExt};
use kube::runtime::controller::Action;
use std::sync::Arc;
use k8s_openapi::api::core::v1::{Pod, Service};

use crate::utils::{Error, Result};
use crate::server::server_crd::MCPServer;
use crate::server::server_controller::{MCPServerController, MCPServerContext};

impl MCPServerController {
    /// Reconcile a MCPServer resource
    pub async fn reconcile(
        server: Arc<MCPServer>,
        _ctx_data: Arc<()>,
        context: Arc<RwLock<MCPServerContext>>,
    ) -> Result<Action> {

        // Get the context data
        let ctx = context.read().await;
        let client = ctx.client.clone();

        // Get the namespace and server name
        let namespace = server.namespace().ok_or_else(|| Error::ReconciliationError("MCPServer has no namespace".to_string()))?;
        let server_name = server.name_any();
        
        info!("Reconciling MCPServer {}/{}", namespace, server_name);
        
        // Create API for Pod resources
        let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
        let services_api: Api<Service> = Api::namespaced(client.clone(), &namespace);
        
        // Check if Pod exists for this server
        let pod_name = format!("MCP-server-{}", server_name);
        let pod_exists = pods_api.get(&pod_name).await.is_ok();
        
        // Check if Service exists for this server
        let service_name = format!("MCP-server-svc-{}", server_name);
        let service_exists = services_api.get(&service_name).await.is_ok();
        
        // Create Pod if it doesn't exist
        if !pod_exists {
            info!("Creating Pod for server {}/{}", namespace, server_name);
            let pod = Self::create_pod(&server, &pod_name, &namespace)?;
            pods_api.create(&PostParams::default(), &pod).await.map_err(Error::KubeError)?;
        }
        
        // Create Service if it doesn't exist
        if !service_exists {
            info!("Creating Service for server {}/{}", namespace, server_name);
            let service = Self::create_service(&server, &service_name, &pod_name, &namespace)?;
            services_api.create(&PostParams::default(), &service).await.map_err(Error::KubeError)?;
        }
        
        // Update server status
        let pod = pods_api.get(&pod_name).await.ok();
        let service = services_api.get(&service_name).await.ok();
        
        Self::update_server_status(client, &server, &namespace, &server_name, pod.as_ref(), service.as_ref()).await?;
        
        // Reconcile again after reconciliation_interval
        let config = ctx.config.clone();
        Ok(Action::requeue(config.reconciliation_interval))
    }
}
