use super::{Controller, MCP_SERVER_OPERATOR_MANAGER};
use crate::{Error, MCPPool, MCPPoolSpec, MCPServer, MCPServerSpec, Result};
use kube::api::{Api, DeleteParams, Patch, PatchParams, PostParams};
use serde_json::json;

impl Controller {
    /// Create a new `MCPPool` resource in Kubernetes.
    pub async fn create_pool(&self, name: &str, spec: MCPPoolSpec) -> Result<MCPPool> {
        Api::namespaced(self.get_client(), &self.get_namespace())
            .create(
                &PostParams {
                    field_manager: Some(MCP_SERVER_OPERATOR_MANAGER.to_string()),
                    ..Default::default()
                },
                &MCPPool::new(name, spec),
            )
            .await
            .map_err(Error::PoolCreateError)
    }

    /// Create a new `MCPServer` resource in Kubernetes.
    pub async fn create_server(&self, name: &str, spec: MCPServerSpec) -> Result<MCPServer> {
        Api::namespaced(self.get_client(), &self.get_namespace())
            .create(
                &PostParams {
                    field_manager: Some(MCP_SERVER_OPERATOR_MANAGER.to_string()),
                    ..Default::default()
                },
                &MCPServer::new(name, spec),
            )
            .await
            .map_err(Error::ServerCreateFailed)
    }

    /// Update an existing `MCPPool` resource in Kubernetes.
    pub async fn patch_pool_spec(&self, name: &str, spec: &MCPPoolSpec) -> Result<MCPPool> {
        let patch = json!({
            "apiVersion": "unmcp.dev/v1",
            "kind": "MCPPool",
            "spec": spec
        });

        Api::namespaced(self.get_client(), &self.get_namespace())
            .patch(
                name,
                &PatchParams::apply(MCP_SERVER_OPERATOR_MANAGER).force(),
                &Patch::Apply(&patch),
            )
            .await
            .map_err(Error::PoolUpdateError)
    }

    /// Update an existing `MCPServer` resource in Kubernetes.
    pub async fn patch_server_spec(&self, name: &str, spec: &MCPServerSpec) -> Result<MCPServer> {
        let patch = json!({
            "apiVersion": "unmcp.dev/v1",
            "kind": "MCPServer",
            "spec": spec
        });

        Api::namespaced(self.get_client(), &self.get_namespace())
            .patch(
                name,
                &PatchParams::apply(MCP_SERVER_OPERATOR_MANAGER).force(),
                &Patch::Apply(&patch),
            )
            .await
            .map_err(Error::ServerUpdateFailed)
    }

    /// Delete an existing `MCPServer` resource from Kubernetes.
    pub async fn delete_server(&self, name: &str) -> Result<()> {
        match Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace())
            .delete(name, &DeleteParams::default())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                if let kube::Error::Api(err) = &e {
                    if err.code == 404 {
                        return Ok(());
                    }
                }
                Err(Error::ServerDeleteFailed(e))
            }
        }
    }

    /// Delete an existing `MCPPool` resource from Kubernetes.
    pub async fn delete_pool(&self, name: &str) -> Result<()> {
        match Api::<MCPPool>::namespaced(self.get_client(), &self.get_namespace())
            .delete(name, &DeleteParams::default())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                if let kube::Error::Api(err) = &e {
                    if err.code == 404 {
                        return Ok(());
                    }
                }
                Err(Error::PoolDeleteError(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{MCPPool, MCPPoolSpec, MCPServer, MCPServerSpec, TestContext};
    use kube::{Api, ResourceExt};

    /// Should return an `MCPPool` instance with the correct name and spec.
    #[tokio::test]
    async fn test_create_pool_result() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let pool = controller
                    .create_pool("test-pool", Default::default())
                    .await
                    .unwrap();
                assert_eq!(pool.name_any(), "test-pool");
                assert_eq!(pool.spec, Default::default());
                Ok(())
            })
            .await
            .unwrap();
    }

    /// Should create an MCPPool resource in the Kubernetes cluster.
    #[tokio::test]
    async fn test_create_pool_in_kube() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let pool_created = controller
                    .create_pool("test-pool", Default::default())
                    .await
                    .unwrap();
                let pool_fetched = Api::<MCPPool>::namespaced(
                    controller.get_client(),
                    &controller.get_namespace(),
                )
                .get(&pool_created.name_any())
                .await
                .unwrap();
                assert_eq!(pool_fetched.name_any(), "test-pool");
                assert_eq!(pool_fetched.spec, Default::default());
                Ok(())
            })
            .await
            .unwrap();
    }

    ///////////////////////////////////////////////////////////////////////

    /// Should return a `MCPServer` instance with the correct name and spec.
    #[tokio::test]
    async fn test_create_server_result() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let server = controller
                    .create_server("test-server", Default::default())
                    .await
                    .unwrap();
                assert_eq!(server.name_any(), "test-server");
                assert_eq!(server.spec, Default::default());
                Ok(())
            })
            .await
            .unwrap();
    }

    /// Should create an MCPServer resource in the Kubernetes cluster.
    #[tokio::test]
    async fn test_create_server_in_kube() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let server_created = controller
                    .create_server("test-server", Default::default())
                    .await
                    .unwrap();
                let server_fetched = Api::<MCPServer>::namespaced(
                    controller.get_client(),
                    &controller.get_namespace(),
                )
                .get(&server_created.name_any())
                .await
                .unwrap();
                assert_eq!(server_fetched.name_any(), "test-server");
                assert_eq!(server_fetched.spec, Default::default());
                Ok(())
            })
            .await
            .unwrap();
    }

    ///////////////////////////////////////////////////////////////////////

    ///////////////////////////////////////////////////////////////////////

    /// Should return the patched `MCPPool` resource.
    #[tokio::test]
    async fn test_patch_pool_spec() {
        TestContext::new()
            .await
            .run(|controller| async move {
                controller
                    .create_pool("test-pool", Default::default())
                    .await
                    .unwrap();
                let new_spec = MCPPoolSpec {
                    default_idle_timeout: 42,
                    ..Default::default()
                };
                let pool = controller
                    .patch_pool_spec("test-pool", &new_spec)
                    .await
                    .unwrap();
                assert_eq!(pool.spec.default_idle_timeout, 42);
                Ok(())
            })
            .await
            .unwrap();
    }

    /// Should patch the spec of an existing `MCPPool` in Kubernetes.
    #[tokio::test]
    async fn test_patch_pool_in_kube() {
        TestContext::new()
            .await
            .run(|controller| async move {
                controller
                    .create_pool("test-pool", Default::default())
                    .await
                    .unwrap();
                let new_spec = MCPPoolSpec {
                    default_idle_timeout: 42,
                    ..Default::default()
                };
                controller
                    .patch_pool_spec("test-pool", &new_spec)
                    .await
                    .unwrap();
                let pool_fetched = Api::<MCPPool>::namespaced(
                    controller.get_client(),
                    &controller.get_namespace(),
                )
                .get("test-pool")
                .await
                .unwrap();
                assert_eq!(pool_fetched.spec.default_idle_timeout, 42);
                Ok(())
            })
            .await
            .unwrap();
    }

    ///////////////////////////////////////////////////////////////////////

    /// Should returned the patched `MCPServer` resource.
    #[tokio::test]
    async fn test_patch_server_spec() {
        TestContext::new()
            .await
            .run(|controller| async move {
                controller
                    .create_server("test-server", Default::default())
                    .await
                    .unwrap();
                let new_spec = MCPServerSpec {
                    pool: "new-pool".to_string(),
                    ..Default::default()
                };
                let patched_server = controller
                    .patch_server_spec("test-server", &new_spec)
                    .await
                    .unwrap();
                assert_eq!(patched_server.spec.pool, "new-pool");
                Ok(())
            })
            .await
            .unwrap();
    }

    /// Should patch the spec of an existing `MCPServer` in Kubernetes.
    #[tokio::test]
    async fn test_patch_server_in_kube() {
        TestContext::new()
            .await
            .run(|controller| async move {
                controller
                    .create_server("test-server", Default::default())
                    .await
                    .unwrap();
                let new_spec = MCPServerSpec {
                    pool: "new-pool".to_string(),
                    ..Default::default()
                };
                controller
                    .patch_server_spec("test-server", &new_spec)
                    .await
                    .unwrap();
                let server_fetched = Api::<MCPServer>::namespaced(
                    controller.get_client(),
                    &controller.get_namespace(),
                )
                .get("test-server")
                .await
                .unwrap();
                assert_eq!(server_fetched.spec.pool, "new-pool");
                Ok(())
            })
            .await
            .unwrap();
    }

    ///////////////////////////////////////////////////////////////////////

    /// Should delete the `MCPServer` resource from Kubernetes.
    #[tokio::test]
    async fn test_delete_server() {
        TestContext::new()
            .await
            .run(|controller| async move {
                controller
                    .create_server("test-server", Default::default())
                    .await
                    .unwrap();
                controller.delete_server("test-server").await.unwrap();
                let server_fetched = Api::<MCPServer>::namespaced(
                    controller.get_client(),
                    &controller.get_namespace(),
                )
                .get("test-server")
                .await;
                assert!(server_fetched.is_err());
                Ok(())
            })
            .await
            .unwrap();
    }

    /// Should delete the `MCPPool` resource from Kubernetes.
    #[tokio::test]
    async fn test_delete_pool() {
        TestContext::new()
            .await
            .run(|controller| async move {
                controller
                    .create_pool("test-pool", Default::default())
                    .await
                    .unwrap();
                controller.delete_pool("test-pool").await.unwrap();
                let pool_fetched = Api::<MCPPool>::namespaced(
                    controller.get_client(),
                    &controller.get_namespace(),
                )
                .get("test-pool")
                .await;
                assert!(pool_fetched.is_err());
                Ok(())
            })
            .await
            .unwrap();
    }
}
