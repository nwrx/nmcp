use super::Controller;
use crate::{Error, MCPPool, MCPServer, Result};
use kube::api::{ListParams, ObjectList};
use kube::Api;

impl Controller {
    /// Lists all MCPPool objects in the current Kubernetes namespace.
    pub async fn list_pools(&self) -> Result<ObjectList<MCPPool>> {
        Api::namespaced(self.get_client(), &self.get_namespace())
            .list(&ListParams::default())
            .await
            .map_err(Error::KubeError)
    }

    /// Lists all MCPServer objects in the current Kubernetes namespace.
    pub async fn list_servers(&self) -> Result<ObjectList<MCPServer>> {
        Api::namespaced(self.get_client(), &self.get_namespace())
            .list(&ListParams::default())
            .await
            .map_err(Error::KubeError)
    }

    /// Gets a specific MCPPool by name.
    pub async fn get_pool_by_name(&self, name: &str) -> Result<MCPPool> {
        Api::namespaced(self.get_client(), &self.get_namespace())
            .get(name)
            .await
            .map_err(Error::PoolGetError)
    }

    /// Gets a specific MCPServer by its UID.
    pub async fn get_server_by_name(&self, name: &str) -> Result<MCPServer> {
        Api::<MCPServer>::namespaced(self.get_client(), &self.get_namespace())
            .get(name)
            .await
            .map_err(Error::ServerGetFailed)
    }

    /// Retrieves the MCPPool that is associated with a given MCPServer.
    pub async fn get_server_pool(&self, server: &MCPServer) -> Result<MCPPool> {
        Api::namespaced(self.get_client(), &self.get_namespace())
            .get(&server.spec.pool)
            .await
            .map_err(Error::ServerPoolNotFound)
    }
}

#[cfg(test)]
mod tests {
    use crate::{MCPServerSpec, TestContext};
    use kube::ResourceExt;

    /// Should return an empty list of servers when no servers are present.
    #[tokio::test]
    async fn test_list_servers_empty() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let servers = controller.list_servers().await.unwrap();
                assert!(servers.items.is_empty());
                Ok(())
            })
            .await
            .unwrap();
    }

    /// Should return an empty list of pools when no pools are present.
    #[tokio::test]
    async fn test_list_pools_empty() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let pools = controller.list_pools().await.unwrap();
                assert!(pools.items.is_empty());
                Ok(())
            })
            .await
            .unwrap();
    }

    /// Should return a list of pools when pools are present.
    #[tokio::test]
    async fn test_list_pools_with_data() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let pool1 = controller.create_pool("p1", Default::default()).await?;
                let pool2 = controller.create_pool("p2", Default::default()).await?;
                let pools = controller.list_pools().await.unwrap();
                assert_eq!(pools.items.len(), 2);
                assert_eq!(pools.items[0].name_any(), pool1.name_any());
                assert_eq!(pools.items[1].name_any(), pool2.name_any());
                Ok(())
            })
            .await
            .unwrap();
    }

    /// Should return a list of servers when servers are present.
    #[tokio::test]
    async fn test_list_servers_with_data() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let server1 = controller.create_server("s1", Default::default()).await?;
                let server2 = controller.create_server("s2", Default::default()).await?;
                let servers = controller.list_servers().await.unwrap();
                assert_eq!(servers.items.len(), 2);
                assert_eq!(servers.items[0].name_any(), server1.name_any());
                assert_eq!(servers.items[1].name_any(), server2.name_any());
                Ok(())
            })
            .await
            .unwrap();
    }

    ///////////////////////////////////////////////////////////////////////

    /// Should return a pool when it exists.
    #[tokio::test]
    async fn test_get_pool() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let pool_created = controller.create_pool("p1", Default::default()).await?;
                let pool_fetched = controller.get_pool_by_name("p1").await?;
                assert_eq!(pool_fetched.name_any(), pool_created.name_any());
                assert_eq!(pool_fetched.spec, pool_created.spec);
                Ok(())
            })
            .await
            .unwrap();
    }

    /// Should reject with `Error::PoolGetError` when the pool does not exist.
    #[tokio::test]
    async fn test_get_pool_not_found() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let result = controller.get_pool_by_name("nonexistent").await;
                assert!(result.is_err());
                Ok(())
            })
            .await
            .unwrap();
    }

    ///////////////////////////////////////////////////////////////////////

    /// Should return a server when it exists.
    #[tokio::test]
    async fn test_get_server() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let server_created = controller.create_server("s1", Default::default()).await?;
                let server_uid = server_created.metadata.uid.as_deref().unwrap();
                let server_fetched = controller.get_server_by_name(server_uid).await?;
                assert_eq!(server_fetched.name_any(), server_created.name_any());
                assert_eq!(server_fetched.spec, server_created.spec);
                Ok(())
            })
            .await
            .unwrap();
    }

    /// Should reject with `Error::ServerNotFound` when the server does not exist.
    #[tokio::test]
    async fn test_get_server_not_found() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let result = controller.get_server_by_name("nonexistent").await;
                assert!(result.is_err());
                Ok(())
            })
            .await
            .unwrap();
    }

    ///////////////////////////////////////////////////////////////////////

    /// Should return the pool associated with the server.
    #[tokio::test]
    async fn test_get_server_pool() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let pool = controller.create_pool("p1", Default::default()).await?;
                let spec = MCPServerSpec {
                    pool: pool.name_any(),
                    ..Default::default()
                };
                let server = controller.create_server("s1", spec).await?;
                let server_pool = controller.get_server_pool(&server).await?;
                assert_eq!(server_pool.name_any(), pool.name_any());
                Ok(())
            })
            .await
            .unwrap();
    }

    /// Should reject with `Error::ServerPoolNotFound` when the server pool does not exist.
    #[tokio::test]
    async fn test_get_server_pool_not_found() {
        TestContext::new()
            .await
            .run(|controller| async move {
                let server = controller.create_server("s1", Default::default()).await?;
                let result = controller.get_server_pool(&server).await;
                assert!(result.is_err());
                Ok(())
            })
            .await
            .unwrap();
    }
}
