use crate::{Error, MCPPool, MCPServer, Result};
use kube::api::{Api, ListParams, ObjectList};
use kube::{Client, ResourceExt};
use std::collections::BTreeMap;

impl MCPPool {
    /// Returns the labels for the MCPPool.
    pub fn labels(&self) -> BTreeMap<String, String> {
        let uid = self.metadata.uid.clone().unwrap_or_default();
        let mut labels = BTreeMap::new();
        labels.insert("unmcp.dev/pool".to_string(), self.name_any());
        labels.insert("unmcp.dev/uid".to_string(), uid);
        labels
    }

    /// List all MCPServer resources controlled by this MCPPool.
    pub async fn list_servers(&self, client: Client) -> Result<ObjectList<MCPServer>> {
        let ns = self.namespace().unwrap_or_default();
        let api: Api<MCPServer> = Api::namespaced(client, &ns);

        // --- Create a label selector to filter MCPServer resources by the MCPPool name.
        let name = self.name_any();
        let label = format!("unmcp.dev/pool={name}");
        let lp = ListParams::default().labels(&label);

        // --- List all MCPServer resources in the namespace with the label selector.
        let result = api.list(&lp).await.map_err(Error::PoolListError)?;
        Ok(result)
    }

    // Get a specific MCPServer resource controlled by this MCPPool by it's UID.
    pub async fn get_server_by_uid(&self, client: Client, uid: String) -> Result<MCPServer> {
        let ns = self.namespace().unwrap_or_default();
        let api: Api<MCPServer> = Api::namespaced(client, &ns);

        // --- Create a label selector to filter MCPServer resources by the MCPPool name and UID.
        let name = self.name_any();
        let label_pool = format!("unmcp.dev/pool={name}");
        let label_uid = format!("unmcp.dev/uid={uid}");
        let lp = ListParams::default().labels(&label_pool).labels(&label_uid);

        // --- List all MCPServer resources in the namespace with the label selector.
        let servers = api.list(&lp).await.map_err(Error::PoolServerListError)?;

        // --- Check if any MCPServer resources were found and return the first one.
        if servers.items.is_empty() {
            return Err(Error::PoolServerNotFoundError(uid.clone()));
        }

        // --- Return the first MCPServer resource found.
        let result = servers.items.into_iter().next().unwrap();
        Ok(result)
    }
}
