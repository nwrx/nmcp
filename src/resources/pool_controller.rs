use super::{MCPPool, ResourceManager};
use kube::api::ObjectMeta;

impl ResourceManager for MCPPool {
    fn new(name: &str, spec: Self::Spec) -> Self {
        Self {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                ..Default::default()
            },
            spec,
            status: Default::default(),
        }
    }
}
