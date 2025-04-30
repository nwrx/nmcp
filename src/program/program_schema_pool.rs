use crate::utils::{serialize, SerializeFormat};
use crate::{Result, MCPPool};
use schemars::schema_for;
use tracing::info;

impl crate::Program {
    /// Generate and output Pool schema
    pub async fn schema_pool(format: SerializeFormat) -> Result<()> {
        info!("Generating Pool schema in {} format", format.to_string());
        let schema = schema_for!(MCPPool);
        let output = serialize(&schema, format)?;
        println!("{}", output);
        Ok(())
    }
}
