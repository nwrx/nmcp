use tokio::sync::RwLock;
use kube::api::{Api, PatchParams, ResourceExt};
use kube::runtime::controller::Action;
use tracing::info;
use std::sync::Arc;
use k8s_openapi::api::core::v1::{Pod, Service};

use crate::MCPServer;
use crate::utils::Result;
use super::{MCPServerContext, MCPServerController};

impl MCPServerController {
    /// Reconcile a MCPServer resource by applying the spec to the Pod and Service resources.
    pub async fn reconcile_apply(server: Arc<MCPServer>, context: Arc<RwLock<MCPServerContext>>) -> Result<Action> {

        // --- Get the context data.
        let ctx: tokio::sync::RwLockReadGuard<'_, MCPServerContext> = context.read().await;
        let client = context.read().await.client.clone();
        let config = ctx.config.clone();
        let ns = server.namespace().unwrap();
        
        // --- Patch the Pod associated with the MCPServer.
        let pod_api: Api<Pod> = Api::namespaced(client.clone(), &ns);
        let pod_name = server.name_pod();
        let pod = pod_api.get(&pod_name).await.unwrap_or_default();
        let pod_patch = server.into_patch_pod(pod)?;
        let pp = PatchParams::apply("unmcp");
        pod_api.patch(&pod_name, &pp, &pod_patch).await?;
        info!("Reconciled pod: {}/{}", ns, pod_name);
        
        // --- Patch the Service associated with the MCPServer.
        let svc_api: Api<Service> = Api::namespaced(client.clone(), &ns);
        let svc_name = server.name_service();
        let svc = svc_api.get(&svc_name).await.unwrap_or_default();
        let svc_patch = server.into_patch_service(svc)?;
        let pp = PatchParams::apply("unmcp");
        svc_api.patch(&svc_name, &pp, &svc_patch).await?;
        info!("Reconciled service: {}/{}", ns, svc_name);

        // --- Reconcile again after reconciliation_interval
        Ok(Action::requeue(config.reconciliation_interval))
    }
}
