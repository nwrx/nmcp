use crate::MCPPool;
use kube::{api::ObjectList, ResourceExt};
use std::collections::BTreeMap;

impl MCPPool {
    /// Returns the labels for the MCPPool.
    pub fn labels(&self) -> BTreeMap<String, String> {
        let uid = self.metadata.uid.clone().unwrap_or_default();
        let mut labels = BTreeMap::new();
        labels.insert("nmcp.nwrx.io/pool".to_string(), self.name_any());
        labels.insert("nmcp.nwrx.io/uid".to_string(), uid);
        labels
    }
}

pub struct MCPPoolList(pub ObjectList<MCPPool>);
impl From<ObjectList<MCPPool>> for MCPPoolList {
    fn from(list: ObjectList<MCPPool>) -> Self {
        MCPPoolList(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_pool_labels() {
        let pool = MCPPool::new("test-pool", Default::default());
        let labels = pool.labels();
        let label_pool = labels.get("nmcp.nwrx.io/pool").unwrap();
        let label_uid = labels.get("nmcp.nwrx.io/uid").unwrap();
        assert_eq!(labels.len(), 2);
        assert_eq!(label_pool, "test-pool");
        assert_eq!(label_uid, pool.metadata.uid.as_ref().unwrap());
    }

    #[test]
    fn test_mcp_pool_list() {
        let pool = MCPPool::new("test-pool", Default::default());
        let list = ObjectList {
            items: vec![pool],
            metadata: Default::default(),
            types: Default::default(),
        };
        let list: MCPPoolList = list.into();
        assert_eq!(list.0.items.len(), 1);
        assert_eq!(list.0.items[0].name_any(), "test-pool");
    }
}
