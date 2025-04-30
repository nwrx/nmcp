use kube::CustomResourceExt;
use crate::utils::{serialize, SerializeFormat};
use crate::{Result, MCPPool};
use super::Program;

impl Program {
    pub async fn get_pool_crd(format: SerializeFormat) -> Result<String> {
        let crd = MCPPool::crd();
        let output = serialize(&crd, format)?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;

    #[tokio::test]
    async fn test_get_pool_crd_yaml() {
        let result = Program::get_pool_crd(SerializeFormat::Yaml).await;
        let deserialized: CustomResourceDefinition = serde_yaml::from_str(&result.unwrap()).unwrap();
        let expected = MCPPool::crd();
        assert_eq!(deserialized, expected);
    }

    #[tokio::test]
    async fn test_get_pool_crd_json() {
        let result = Program::get_pool_crd(SerializeFormat::Json).await;
        let deserialized: CustomResourceDefinition = serde_json::from_str(&result.unwrap()).unwrap();
        let expected = MCPPool::crd();
        assert_eq!(deserialized, expected);
    }
}
