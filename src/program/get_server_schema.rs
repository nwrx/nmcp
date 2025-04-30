use schemars::schema_for;
use crate::utils::{serialize, SerializeFormat};
use crate::{Result, MCPServer};
use super::Program;

impl Program {
    pub async fn get_server_schema(format: SerializeFormat) -> Result<String> {
        let schema = schema_for!(MCPServer);
        let output = serialize(&schema, format)?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::{schema::RootSchema, schema_for};

    #[tokio::test]
    async fn test_get_server_schema_yaml() {
        let result = Program::get_server_schema(SerializeFormat::Yaml).await;
        let deserialized: RootSchema = serde_yaml::from_str(&result.unwrap()).unwrap();
        let expected = schema_for!(MCPServer);
        assert_eq!(deserialized, expected);
    }

    #[tokio::test]
    async fn test_get_server_schema_json() {
        let result = Program::get_server_schema(SerializeFormat::Json).await;
        let deserialized: RootSchema = serde_json::from_str(&result.unwrap()).unwrap();
        let expected = schema_for!(MCPServer);
        assert_eq!(deserialized, expected);
    }
}
