use super::{Controller, NMCP_FINALIZER};
use crate::{Error, MCPServer, Result};
use futures::StreamExt;
use k8s_openapi::api::core::v1;
use kube::runtime::controller::Action;
use kube::runtime::finalizer;
use kube::runtime::finalizer::Event;
use kube::runtime::{watcher::Config, Controller as RuntimeController};
use kube::{Api, ResourceExt};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
struct ReconcileReportError(Error);

impl std::fmt::Display for ReconcileReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ReconcileReportError {}

impl From<ReconcileReportError> for Error {
    fn from(e: ReconcileReportError) -> Self {
        e.0
    }
}

impl Controller {
    /// Reconcile the MCPServer resource by checking its status and updating it accordingly.
    #[tracing::instrument(name = "Reconcile", skip_all, fields(server = %server.name_any()))]
    async fn reconcile(
        &self,
        server: Arc<MCPServer>,
    ) -> core::result::Result<Action, finalizer::Error<ReconcileReportError>> {
        let api = Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace());

        // --- Handle the reconciliation process using finalizers to ensure
        // --- that the cleanup process is completed before the resource is deleted.
        finalizer(&api, NMCP_FINALIZER, server, {
            let client = self.get_client();
            move |event| async move {
                match event {
                    Event::Cleanup(server) => {
                        server.down(&client).await.map_err(ReconcileReportError)?;
                        Ok(Action::requeue(Duration::from_secs(5)))
                    }
                    Event::Apply(server) => async {
                        server
                            .reconcile_server(&client)
                            .await
                            .map_err(ReconcileReportError)?;
                        Result::Ok(Action::requeue(Duration::from_secs(5)))
                    }
                    // The `kube::runtime::finalizer` expects it's reconcile closure to return an error that
                    // implements `std::error::Error`, however, since we are using `error_stack::Report` and
                    // since that type does not implement `std::error::Error`, we need to temporarily wrap it
                    // in a custom error type that does implement `std::error::Error`.
                    .await
                    .map_err(ReconcileReportError),
                }
            }
        })
        .await
    }

    /// Handle an error during the reconciliation process.
    #[tracing::instrument(name = "ErrorPolicy", skip_all)]
    fn error_policy(
        &self,
        _server: &MCPServer,
        error: &finalizer::Error<ReconcileReportError>,
    ) -> Result<Action> {
        match error {
            finalizer::Error::ApplyFailed(e) => {
                let _ = e.0.clone().trace();
                Ok(Action::requeue(Duration::from_secs(5)))
            }
            _ => {
                tracing::error!("Unhandled error during MCPServer reconciliation: {}", error);
                // Requeue the action to retry later
                Ok(Action::requeue(Duration::from_secs(5)))
            }
        }
    }

    /// Start the operator for managing MCPServer resources.
    #[tracing::instrument(name = "Operator", skip_all, err)]
    pub async fn start_server_operator(&self) -> Result<()> {
        let ns = self.get_namespace();
        let wc = Config::default();

        // --- Create API clients for MCPServer, Pod, and Service.
        let api = Api::<MCPServer>::namespaced(self.get_client(), &ns);
        let api_pod = Api::<v1::Pod>::namespaced(self.get_client(), &ns);
        let api_services = Api::<v1::Service>::namespaced(self.get_client(), &ns);

        // --- Start the controller for MCPServer resources.
        tracing::info!("Starting MCPServer operator in namespace '{}'", ns);
        let stream = RuntimeController::new(api, wc.clone())
            .owns(api_pod, Default::default())
            .owns(api_services, Default::default())
            .run(
                |server, controller| async move { controller.reconcile(server).await },
                |server, error, controller| controller.error_policy(&server, error).unwrap(),
                Arc::new(self.clone()),
            );

        // --- Loop to handle the reconciliation stream.
        stream.for_each(|_| futures::future::ready(())).await;

        Ok(())
    }
}
