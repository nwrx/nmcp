use crate::utils::{serialize, SerializeFormat};
use crate::{Result, MCPPool};
use kube::CustomResourceExt;
use tracing::info;

impl crate::Program {
    /// Generate and output Pool CRD
    pub async fn crd_pool(format: SerializeFormat) -> Result<()> {
        info!("Generating Pool CRD in {} format", format.to_string());
        let crd = MCPPool::crd();
        let output = serialize(&crd, format)?;
        println!("{}", output);
        Ok(())
    }
}
