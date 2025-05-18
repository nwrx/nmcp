use crate::{Error, Result};
use serde_json as JSON;
use serde_yml as YAML;

/// Serialize a CRD object to JSON or YAML based on the output format
pub fn serialize<T: serde::Serialize>(crd: &T, format: &str) -> Result<String> {
    match format {
        "json" => JSON::to_string_pretty(&crd).map_err(Error::SerializeJsonError),
        "yaml" => YAML::to_string(&crd).map_err(Error::YamlError),
        _ => Err(Error::UnsupportedFormat(format.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MCPPool;
    use kube::CustomResourceExt;

    #[test]
    fn test_serialize_crd_json() {
        let crd = MCPPool::crd();
        let result = serialize(&crd, "json").unwrap();
        assert!(result.contains("\"kind\": \"MCPPool\""));
    }

    #[test]
    fn test_serialize_crd_yaml() {
        let crd = MCPPool::crd();
        let result = serialize(&crd, "yaml").unwrap();
        assert!(result.contains("kind: MCPPool"));
    }

    #[test]
    fn test_serialize_crd_invalid_format() {
        let crd = MCPPool::crd();
        let result = serialize(&crd, "xml");
        assert!(result.is_err());
    }
}
