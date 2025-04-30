use kube::CustomResourceExt;
use crate::utils::{serialize, Result, SerializeFormat};
use crate::MCPServer;
use super::Program;

impl Program {
    pub async fn get_server_crd(format: SerializeFormat) -> Result<String> {
        let crd = MCPServer::crd();
        let output = serialize(&crd, format)?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;

    #[tokio::test]
    async fn test_get_server_crd_yaml() {
        let result = Program::get_server_crd(SerializeFormat::Yaml).await;
        let deserialized: CustomResourceDefinition = serde_yaml::from_str(&result.unwrap()).unwrap();
        let expected = MCPServer::crd();
        assert_eq!(deserialized, expected);
    }

    #[tokio::test]
    async fn test_get_server_crd_json() {
        let result = Program::get_server_crd(SerializeFormat::Json).await;
        let deserialized: CustomResourceDefinition = serde_json::from_str(&result.unwrap()).unwrap();
        let expected = MCPServer::crd();
        assert_eq!(deserialized, expected);
    }
}
