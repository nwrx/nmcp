use crate::utils::{serialize, SerializeFormat};
use crate::{Result, MCPServer};
use schemars::schema_for;
use tracing::info;

impl crate::Program {
    /// Generate and output Server schema
    pub async fn schema_server(format: SerializeFormat) -> Result<()> {
        info!("Generating Server schema in {} format", format.to_string());
        let schema = schema_for!(MCPServer);
        let output = serialize(&schema, format)?;
        println!("{}", output);
        Ok(())
    }
}