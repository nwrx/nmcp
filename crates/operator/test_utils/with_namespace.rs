use super::TestContext;
use anyhow::Result;
use k8s_openapi::api::core::v1::Namespace;
use kube::api::{DeleteParams, ObjectMeta, PostParams};
use kube::Api;
use rand::{distr::Alphanumeric, Rng};
use std::future::Future;

impl TestContext {
    /// Create a test namespace and run the test function within that namespace.
    /// The namespace will be deleted after the test function completes.
    pub async fn with_namespace<F, Fut, T>(&self, test_fn: F) -> Result<T>
    where
        F: FnOnce(String, TestContext) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        // --- Get the client.
        let client = self.get_client().await?;

        // --- Create a random namespace name.
        let namespace_name: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(|c| c.to_ascii_lowercase())
            .map(char::from)
            .collect();

        // --- Define a namespace.
        let namespace = Namespace {
            metadata: ObjectMeta {
                name: Some(namespace_name.to_string()),
                ..Default::default()
            },
            spec: None,
            status: None,
        };

        // --- Create the namespace.
        let namespace_api = Api::<Namespace>::all(client.clone());
        namespace_api
            .create(&PostParams::default(), &namespace)
            .await?;

        // --- Clone the context and pass ownership to the test function
        let test_context = self.clone();
        let result = match test_fn(namespace_name.clone(), test_context).await {
            Ok(r) => {
                println!("Test function completed successfully");
                Ok(r)
            }
            Err(e) => Err(e),
        };

        // --- Clean up by deleting the namespace
        let delete_params = DeleteParams::default();
        namespace_api
            .delete(&namespace_name, &delete_params)
            .await?;

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_with_namespace_create() {
        let context = TestContext::new().unwrap();
        let result = context
            .with_namespace(|namespace, context| async move {
                let client = context.get_client().await?;
                let namespace_api = Api::<Namespace>::all(client);
                let ns = namespace_api.get(&namespace).await?;
                assert_eq!(ns.metadata.name, Some(namespace));
                Ok(())
            })
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_with_namespace_teardown() {
        let context = TestContext::new().unwrap();
        let client = context.get_client().await.unwrap();

        // --- Call the function within a temporary namespace.
        let namespace = context
            .with_namespace(|ns, _| async move { Ok(ns) })
            .await
            .unwrap();

        // --- Check if the namespace still exists.
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
