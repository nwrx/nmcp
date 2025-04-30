use tracing::{error, info};
use futures::stream::StreamExt;
use kube::api::{Api, ResourceExt};
use kube::runtime::{controller::Action, Controller, watcher::Config};
use std::sync::Arc;

use crate::utils::Result;
use crate::pool::pool_crd::MCPPool;
use crate::pool::pool_controller::MCPPoolController;

impl MCPPoolController {
    /// Start the controller
    pub async fn start(&self) -> Result<()> {
        info!("Starting MCPPool controller");
        
        // --- Get the context data and client.
        let context = self.context.clone();
        let client = context.read().await.client.clone();
        let config = context.read().await.config.clone();
        
        // --- Define the API for MCPPool resources
        let pools: Api<MCPPool> = match &config.namespace {
            Some(namespace) => Api::namespaced(client.clone(), namespace),
            None => Api::all(client.clone()),
        };
        
        // Start watching for MCPPool resources and handle events
        // The controller will watch for changes to MCPPool resources and call the reconcile function
        // when a change is detected (create, update, or delete). It will also handle errors
        // that occur during reconciliation.
        Controller::new(pools.clone(), Config::default())
            .run(
                // This closure is called when a MCPPool resource is created, updated, or deleted.
                // It receives the pool resource and the context data (which is empty in this case).
                |pool, ctx_data| {
                    let context = context.clone();
                    async move { Self::reconcile(pool, ctx_data, context).await }
                },

                // This closure is called when an error occurs during reconciliation.
                // It receives the pool resource, the error, and the context data (which is empty in this case).
                // It logs the error and returns an Action to requeue the reconciliation after a specified interval.
                move |pool, error, _| {
                    error!("Error reconciling MCPPool {}: {:?}", pool.name_any(), error);
                    Action::requeue(config.reconciliation_interval)
                },

                // This closure is called when the controller starts.
                Arc::new(()),
            )
            // Iterate over the events and handle them as they occur.
            .for_each(|event| {
                async move {
                    match event {
                        Ok(_) => {}
                        Err(e) => error!("Error during reconciliation: {}", e),
                    }
                }
            })
            .await;

        Ok(())
    }
}
