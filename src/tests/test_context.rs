use crate::{get_kube_client, Controller, ControllerOptions, Kubeconfig, MCPPool, MCPServer};
use anyhow::Result;
use k8s_openapi::api::core::v1::Namespace;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::{DeleteParams, PostParams};
use kube::{Api, CustomResourceExt};
use std::env::temp_dir;
use std::future::Future;
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers_modules::k3s::K3s;
use tokio::sync::OnceCell;
use uuid::Uuid;

/// Global container for the test context.
static TEST_CONTEXT: OnceCell<TestContext> = OnceCell::const_new();

pub async fn get_test_context() -> TestContext {
    TEST_CONTEXT.get_or_init(TestContext::new).await.clone()
}

#[derive(Clone, Debug)]
pub struct TestContext {
    kubeconfig: kube::config::Kubeconfig,
    container: Arc<ContainerAsync<K3s>>,
}

impl TestContext {
    pub async fn new() -> Self {
        let container = K3s::default()
            .with_conf_mount(temp_dir())
            .with_privileged(true)
            .with_userns_mode("host")
            .with_reuse(testcontainers::ReuseDirective::Never)
            .start()
            .await
            .expect("Failed to start K3s instance");

        // --- Get the kubeconfig path from the K3s instance.
        let kubeconfig = Kubeconfig::from_container(&container).await.unwrap();
        let client = get_kube_client(kubeconfig.clone()).await.unwrap();

        // --- Create the CRDs.
        let api: Api<CustomResourceDefinition> = Api::all(client);
        let pp = PostParams::default();
        let _ = api.create(&pp, &MCPServer::crd()).await.unwrap();
        let _ = api.create(&pp, &MCPPool::crd()).await.unwrap();

        // --- Return the test context with it's associated kubeconfig.
        Self {
            kubeconfig: kubeconfig.into(),
            container: Arc::new(container),
        }
    }

    /// Create a test namespace and run the test function within that namespace.
    /// The namespace will be deleted after the test function completes.
    pub async fn run<F, Fut, T>(&self, r#fn: F) -> Result<T>
    where
        F: FnOnce(Controller) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        // --- Create the controller options.
        let name = Uuid::new_v4().to_string();
        let options = ControllerOptions {
            namespace: name.clone(),
            kubeconfig: self.kubeconfig.clone().into(),
        };

        // --- Create the controller.
        let controller = Controller::new(&options)
            .await
            .expect("Failed to create controller");

        // --- Create the namespace.
        let mut namespace = Namespace::default();
        namespace.metadata.name = Some(name.clone());
        let client = controller.get_client();

        let api = Api::<Namespace>::all(client);
        let _ = api
            .create(&PostParams::default(), &namespace)
            .await
            .expect("Failed to create namespace");

        // --- Clone the context and pass ownership to the test function
        let result = r#fn(controller).await;

        // --- Clean up by deleting the namespace.
        let _ = api
            .delete(&name, &DeleteParams::default())
            .await
            .expect("Failed to delete namespace");

        result
    }

    /// Dispose of the test context. This will stop the K3s instance.
    /// This is called automatically when the test context is dropped.
    pub async fn dispose(&self) {
        self.container
            .stop()
            .await
            .expect("Failed to stop K3s instance");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the creation of a namespace and verifies its existence.
    #[tokio::test]
    async fn test_run_create_namespace() {
        let result = get_test_context()
            .await
            .run(|controller| async move {
                let client = controller.get_client();
                let namespace_name = controller.get_namespace();
                let namespace_api = Api::<Namespace>::all(client);
                let namespace = namespace_api.get(&namespace_name).await.unwrap();
                assert_eq!(namespace.metadata.name, Some(namespace_name));
                Ok(())
            })
            .await;
        assert!(result.is_ok());
    }

    /// Tests the deletion of a namespace after running a test function.
    #[tokio::test]
    async fn test_run_namespace_teardown() {
        let context = get_test_context().await;
        let namespace = context
            .run(|controller| async move { Ok(controller.get_namespace()) })
            .await
            .expect("Failed to run test function");

        let config = context.kubeconfig.clone().into();
        let client = get_kube_client(config).await.unwrap();
        let phase = Api::<Namespace>::all(client)
            .get(&namespace)
            .await
            .unwrap()
            .status
            .unwrap()
            .phase
            .unwrap();

        // --- Check if the namespace is in the "Terminating" phase.
        assert_eq!(
            phase, "Terminating",
            "Namespace was not deleted successfully"
        );
    }
}
