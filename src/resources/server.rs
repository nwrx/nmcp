use super::MCPServer;
use kube::api::{ObjectList, ResourceExt};
use std::collections::BTreeMap;

impl MCPServer {
    /// Returns the name of the `v1::Service` for the `MCPServer`.
    pub fn name_service(&self) -> String {
        format!(
            "mcp-server-svc-{}-{}-{}",
            self.spec.pool,
            self.name_any(),
            &self.metadata.uid.clone().unwrap()[..8]
        )
    }

    /// Returns the name of the `v1::Pod` for the `MCPServer`.
    pub fn name_pod(&self) -> String {
        format!(
            "mcp-server-{}-{}-{}",
            self.spec.pool,
            self.name_any(),
            &self.metadata.uid.clone().unwrap()[..8]
        )
    }

    /// Returns the labels for the `MCPServer`.
    pub fn labels(&self) -> BTreeMap<String, String> {
        let uid = self.metadata.uid.clone().unwrap_or_default();
        let mut labels = std::collections::BTreeMap::new();
        labels.insert("app".to_string(), self.name_pod());
        labels.insert("nmcp.nwrx.io/uid".to_string(), uid);
        labels.insert("nmcp.nwrx.io/pool".to_string(), self.spec.pool.clone());
        labels
    }
}

// Wrapper for ObjectList<MCPServer>
pub struct MCPServerList(pub ObjectList<MCPServer>);
impl From<ObjectList<MCPServer>> for MCPServerList {
    fn from(list: ObjectList<MCPServer>) -> Self {
        MCPServerList(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_name_service() {
        let mut server = MCPServer::new("test-server", Default::default());
        server.metadata.uid = uuid::Uuid::new_v4().to_string().into();
        let service_name = server.name_service();
        let expected = format!(
            "mcp-server-svc-{}-{}-{}",
            server.spec.pool,
            server.name_any(),
            &server.metadata.uid.clone().unwrap()[..8]
        );
        assert_eq!(service_name, expected);
    }

    #[test]
    fn test_mcp_server_name_pod() {
        let mut server = MCPServer::new("test-server", Default::default());
        server.metadata.uid = uuid::Uuid::new_v4().to_string().into();
        let pod_name = server.name_pod();
        let expected = format!(
            "mcp-server-{}-{}-{}",
            server.spec.pool,
            server.name_any(),
            &server.metadata.uid.clone().unwrap()[..8]
        );
        assert_eq!(pod_name, expected);
    }

    #[test]
    fn test_mcp_server_labels() {
        let mut server = MCPServer::new("test-server", Default::default());
        server.metadata.uid = uuid::Uuid::new_v4().to_string().into();
        let labels = server.labels();
        let label_app = labels.get("app").unwrap();
        let label_uid = labels.get("nmcp.nwrx.io/uid").unwrap();
        let label_pool = labels.get("nmcp.nwrx.io/pool").unwrap();
        assert_eq!(labels.len(), 3);
        assert_eq!(label_app, &server.name_pod());
        assert_eq!(label_uid, server.metadata.uid.as_ref().unwrap());
        assert_eq!(label_pool, &server.spec.pool);
    }

    #[test]
    fn test_mcp_server_list() {
        let list = ObjectList::<MCPServer> {
            metadata: Default::default(),
            types: Default::default(),
            items: vec![
                MCPServer::new("s1", Default::default()),
                MCPServer::new("s2", Default::default()),
                MCPServer::new("s3", Default::default()),
            ],
        };
        let mcp_server_list: MCPServerList = list.into();
        assert_eq!(mcp_server_list.0.items.len(), 3);
    }
}
