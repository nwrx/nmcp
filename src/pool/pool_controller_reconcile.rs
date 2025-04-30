use tokio::sync::RwLock;
use tracing::info;
use kube::api::{Api, PostParams, ResourceExt};
use kube::runtime::controller::Action;
use std::sync::Arc;

use crate::utils::{Error, Result};
use crate::pool::pool_crd::MCPPool;
use crate::pool::pool_controller::{MCPPoolController, MCPPoolContext};
use crate::MCPServer;

impl MCPPoolController {
    /// Reconcile a MCPPool resource
    pub async fn reconcile(pool: Arc<MCPPool>, _ctx_data: Arc<()>, context: Arc<RwLock<MCPPoolContext>>) -> Result<Action> {
        
        // Get the context data
        let ctx = context.read().await;
        let client = ctx.client.clone();

        // Get the namespace and pool name
        let namespace = pool.namespace().ok_or_else(|| Error::ReconciliationError("MCPPool has no namespace".to_string()))?;
        let pool_name = pool.name_any();
        
        info!("Reconciling MCPPool {}/{}", namespace, pool_name);

        // Create API for MCPServer resources
        let servers_api: Api<MCPServer> = Api::namespaced(client.clone(), &namespace);

        // Get existing server count in this pool
        let servers = servers_api
            .list(&kube::api::ListParams::default()
                .labels(&format!("unmcp.dev/pool={}", pool_name)))
            .await
            .map_err(Error::KubeError)?;
        
        let current_servers = servers.items.len() as i32;
        let min_servers = pool.spec.min_servers;

        // Create servers if we have fewer than min_servers
        if current_servers < min_servers {
            let servers_to_create = min_servers - current_servers;
            info!("Creating {} servers for pool {}/{}", servers_to_create, namespace, pool_name);
            
            for i in 0..servers_to_create {
                let server_name = format!("mcp-pool-{}-{}", pool_name, current_servers + i);
                let server = Self::create_server_resource(&pool, &server_name, &namespace)?;
                servers_api.create(&PostParams::default(), &server).await.map_err(Error::KubeError)?;
            }
        }

        // Update pool status
        Self::update_pool_status(context.clone(), &pool).await?;
        
        // Reconcile again after reconciliation_interval
        let config = ctx.config.clone();
        Ok(Action::requeue(config.reconciliation_interval))
    }
}
