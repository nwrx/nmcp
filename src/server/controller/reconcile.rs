use tokio::sync::RwLock;
use kube::api::{Api, ResourceExt};
use kube::runtime::{controller::Action, finalizer::{finalizer, Event}};
use std::sync::Arc;

use crate::MCPServer;
use crate::server::MCP_SERVER_FINALIZER;
use crate::utils::{Result, Error};
use super::{MCPServerContext, MCPServerController};

impl MCPServerController {
    /// Reconcile the MCPServer resource by applying the spec to the Pod and Service resources.
    pub async fn reconcile(server: Arc<MCPServer>, context: Arc<RwLock<MCPServerContext>>) -> Result<Action> {
        
        // --- Get the context data.
        let client = context.read().await.client.clone();
        let ns = server.namespace().unwrap();
        let api = Api::<MCPServer>::namespaced(client, &ns);
        let obj = server.clone();

        // --- Handle the reconciliation process using finalizers to ensure
        // --- that the cleanup process is completed before the resource is deleted.
        finalizer(&api, MCP_SERVER_FINALIZER, obj, {
            let server_clone = server.clone();
            move |event| {
                let server = server_clone.clone();
                let context = context.clone();
                async move {
                    match event {
                        Event::Cleanup { .. } => {
                            Self::reconcile_cleanup(server, context).await
                        },
                        Event::Apply { .. } => {
                            Self::reconcile_apply(server, context).await
                        }
                    }
                }
            }
        })

        // --- Await for the finalizer to complete and wrap any errors
        // --- into a custom error type for better error handling.
        .await
        .map_err(|error| Error::FinalizerError {
            name: server.name_any(),
            message: error.to_string(),
            finalizer: MCP_SERVER_FINALIZER.to_string(),
        })
    }
}

