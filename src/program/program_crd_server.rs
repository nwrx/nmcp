use crate::utils::{serialize, SerializeFormat};
use crate::{Result, MCPServer};
use kube::CustomResourceExt;
use tracing::info;

impl crate::Program {
    /// Generate and output Server CRD
    pub async fn crd_server(format: SerializeFormat) -> Result<()> {
        info!("Generating Server CRD in {} format", format.to_string());
        let crd = MCPServer::crd();
        let output = serialize(&crd, format)?;
        println!("{}", output);
        Ok(())
    }
}
