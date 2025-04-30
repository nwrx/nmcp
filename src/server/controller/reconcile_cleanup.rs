use tokio::sync::RwLock;
use kube::api::{Api, DeleteParams, PropagationPolicy, ResourceExt};
use kube::runtime::controller::Action;
use tracing::info;
use std::sync::Arc;
use k8s_openapi::api::core::v1::{Pod, Service};

use crate::MCPServer;
use crate::utils::Result;
use super::{MCPServerContext, MCPServerController};

impl MCPServerController {
    /// Clean up the MCPServer resource by deleting the associated Pod and Service resources.
    pub async fn reconcile_cleanup(server: Arc<MCPServer>, context: Arc<RwLock<MCPServerContext>>) -> Result<Action> {

        // --- Get the context data.
        let client = context.read().await.client.clone();
        let ns = server.namespace().unwrap();

        // --- Prepare delete params.
        let dp = DeleteParams {
            dry_run: false,
            grace_period_seconds: Some(0),
            propagation_policy: Some(PropagationPolicy::Foreground),
            ..Default::default()
        };

        // --- Delete the Pod associated with the MCPServer.
        let pod_api: Api<Pod> = Api::namespaced(client.clone(), &ns);
        let pod_name = server.name_pod();
        match pod_api.get(&pod_name).await {
            Ok(_) => {
                match pod_api.delete(&pod_name, &dp).await {
                    Ok(_) => info!("Deleted pod: {}/{}", ns, pod_name),
                    Err(e) => info!("Pod {}/{} may not exist or cannot be deleted: {}", ns, pod_name, e),
                }
            },
            Err(_) => info!("Pod {}/{} not found, skipping deletion", ns, pod_name),
        }
        
        // --- Delete the Service associated with the MCPServer.
        let svc_api: Api<Service> = Api::namespaced(client.clone(), &ns);
        let svc_name = server.name_service();
        match svc_api.get(&svc_name).await {
            Ok(_) => {
                match svc_api.delete(&svc_name, &dp).await {
                    Ok(_) => info!("Deleted service: {}/{}", ns, svc_name),
                    Err(e) => info!("Service {}/{} may not exist or cannot be deleted: {}", ns, svc_name, e),
                }
            },
            Err(_) => info!("Service {}/{} not found, skipping deletion", ns, svc_name),
        }

        // The finalizer system will automatically remove the finalizer after this completes successfully
        // We're returning Action::await_change() to indicate that we don't need to requeue
        Ok(Action::await_change())
    }
}
