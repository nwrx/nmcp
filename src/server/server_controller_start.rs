use tracing::{error, info};
use futures::stream::StreamExt;
use kube::{
    api::{Api, ResourceExt},
    runtime::{controller::Action, Controller, watcher::Config},
};
use std::sync::Arc;

use crate::utils::Result;
use crate::server::server_controller::MCPServerController;
use crate::server::server_crd::MCPServer;

impl MCPServerController {
    /// Start the controller
    pub async fn start(&self) -> Result<()> {
        info!("Starting MCPServer controller");
        
        let context = self.context.clone();
        let client = context.read().await.client.clone();
        let config = context.read().await.config.clone();
        
        // Define the API for MCPServer resources
        let servers: Api<MCPServer> = match &config.namespace {
            Some(namespace) => Api::namespaced(client.clone(), namespace),
            None => Api::all(client.clone()),
        };
        
        // Start watching for MCPServer resources and handle events
        // The controller will watch for changes to MCPServer resources and call the reconcile function
        // when a change is detected (create, update, or delete). It will also handle errors
        // that occur during reconciliation.
        Controller::new(servers.clone(), Config::default())
            .run(

                // This closure is called when a MCPServer resource is created, updated, or deleted.
                // It receives the server resource and the context data (which is empty in this case).
                |server, ctx_data| {
                    let context = context.clone();
                    async move { Self::reconcile(server, ctx_data, context).await }
                },

                // This closure is called when an error occurs during reconciliation.
                // It receives the server resource, the error, and the context data (which is empty in this case).
                // It logs the error and returns an Action to requeue the reconciliation after a specified interval.
                move |server, error, _| {
                    error!("Error reconciling MCPServer {}: {:?}", server.name_any(), error);
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
