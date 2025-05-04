use crate::MCPServer;
use kube::api::ResourceExt;
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
        let transport = self.spec.transport.clone();
        let mut labels = std::collections::BTreeMap::new();
        labels.insert("app".to_string(), self.name_pod());
        labels.insert("unmcp.dev/uid".to_string(), uid);
        labels.insert("unmcp.dev/pool".to_string(), self.spec.pool.clone());
        labels.insert("unmcp.dev/transport".to_string(), transport.to_string());
        labels
    }
}
