use k8s_openapi::api::core::v1::EnvVar;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Wrapper for EnvVar to implement JsonSchema and allow direct access to name and value
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MCPServerEnvVar {
    /// The name of the environment variable.
    pub name: String,

    /// The value of the environment variable.
    #[serde(default)]
    pub value: String,
}

impl MCPServerEnvVar {
    /// Creates a new `MCPServerEnvVar` instance.
    ///
    /// # Parameters
    /// - `name`: The name of the environment variable.
    /// - `value`: The value of the environment variable.
    ///
    /// # Returns
    /// A new `MCPServerEnvVar` instance.
    /// 
    /// # Details
    /// This function initializes a new `MCPServerEnvVar` instance with the provided name and value.
    /// The `name` and `value` fields are stored as `String`.
    /// 
    /// # Example
    /// ```
    /// use unmcp::MCPServerEnvVar;
    ///
    /// let env_var = MCPServerEnvVar::new("TEST_VAR", "test_value");
    /// assert_eq!(env_var.name, "TEST_VAR");
    /// assert_eq!(env_var.value, "test_value");
    /// ```
    pub fn new(name: &str, value: &str) -> Self {
        MCPServerEnvVar {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}

impl From<MCPServerEnvVar> for EnvVar {

    /// Converts a `MCPServerEnvVar` instance to a Kubernetes `EnvVar`.
    ///
    /// # Parameters
    /// - `env_var`: The `MCPServerEnvVar` instance to convert.
    ///
    /// # Returns
    /// A new `EnvVar` instance with the same name and value as the `MCPServerEnvVar`.
    ///
    /// # Details
    /// This function takes a `MCPServerEnvVar` instance and converts it into a Kubernetes `EnvVar`.
    /// The `name` and `value` fields are directly copied, while the `value_from` field is set to `None`.
    /// 
    /// # Example
    /// ```
    /// use unmcp::MCPServerEnvVar;
    /// use k8s_openapi::api::core::v1::EnvVar;
    /// 
    /// let env_var = MCPServerEnvVar::new("TEST_VAR", "test_value");
    /// let kube_env_var: EnvVar = env_var.into();
    /// assert_eq!(kube_env_var.name, "TEST_VAR");
    /// assert_eq!(kube_env_var.value, Some("test_value".to_string()));
    /// ```
    fn from(env_var: MCPServerEnvVar) -> Self {
        EnvVar {
            name: env_var.name,
            value: Some(env_var.value),
            value_from: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_new_env_var() {
        let env_var = MCPServerEnvVar::new("TEST_VAR", "test_value");
        assert_eq!(env_var.name, "TEST_VAR");
        assert_eq!(env_var.value, "test_value");
    }

    #[test]
    fn test_mcp_server_env_var_from() {
        let env_var = MCPServerEnvVar::new("TEST_VAR", "test_value");
        let kube_env_var: EnvVar = env_var.clone().into();
        assert_eq!(kube_env_var.name, env_var.name);
        assert_eq!(kube_env_var.value, Some(env_var.value));
    }

    #[test]
    fn test_mcp_server_env_var_from_empty_value() {
        let env_var = MCPServerEnvVar::new("TEST_VAR", "");
        let kube_env_var: EnvVar = env_var.clone().into();
        assert_eq!(kube_env_var.name, env_var.name);
        assert_eq!(kube_env_var.value, Some(env_var.value));
    }
}
