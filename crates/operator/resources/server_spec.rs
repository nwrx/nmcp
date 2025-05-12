use crate::{MCPServerStatus, MCPServerTransport};
use k8s_openapi::api::core::v1;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// MCPServer custom resource definition
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq)]
#[kube(
    group = "unmcp.dev",
    version = "v1",
    kind = "MCPServer",
    plural = "mcpservers",
    singular = "mcpserver",
    shortname = "mcp",
    namespaced,
    status = "MCPServerStatus",
    printcolumn = r#"{"name":"Pool", "type":"string", "jsonPath":".spec.pool"}"#,
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct MCPServerSpec {
    /// Reference to McpPool resource this server belongs to. This will be used to
    /// determine in which pool the server is running, thus allowing the controller to
    /// manage the server's lifecycle based on the pool's specifications.
    #[serde(default = "default_pool")]
    pub pool: String,

    /// Container image to use for the server. This image will be pulled from the
    /// container registry and used to create the server's pod.
    #[serde(default = "default_image")]
    pub image: String,

    /// The command to run the server. This will be used to start the server's
    /// process inside the container.
    #[serde(default = "default_command")]
    pub command: Option<Vec<String>>,

    /// The arguments to pass to the server's command. This will be used to
    /// configure the server's runtime behavior, such as specifying the
    /// configuration file to use or enabling/disabling certain features.
    #[serde(default)]
    pub args: Option<Vec<String>>,

    // Environment variables for the server to use. This will be used to
    // configure the server's runtime environment, such as database connections,
    #[serde(default = "default_env")]
    pub env: Vec<v1::EnvVar>,

    /// The MCP transport used by the server. This will be used to determine how the server
    /// communicates with other components in the system, such as the database or other servers.
    /// Can either be "sse" (HTTP) or "stdio" (STDIN/STDOUT).
    #[serde(default)]
    pub transport: MCPServerTransport,

    /// The time in seconds that a server is allowed to run without receiving
    /// any requests before it's terminated. This helps to conserve resources by
    /// shutting down idle servers.
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u32,
}

impl Default for MCPServerSpec {
    fn default() -> Self {
        Self {
            pool: default_pool(),
            image: default_image(),
            command: default_command(),
            args: None,
            env: default_env(),
            transport: MCPServerTransport::default(),
            idle_timeout: default_idle_timeout(),
        }
    }
}

/// Default pool name
fn default_pool() -> String {
    "default".to_string()
}

/// Default image string
fn default_image() -> String {
    "mcp/fetch:latest".to_string()
}

/// Default environment variables
fn default_env() -> Vec<v1::EnvVar> {
    vec![]
}

/// Default command
fn default_command() -> Option<Vec<String>> {
    None
}

/// Default idle timeout in seconds
fn default_idle_timeout() -> u32 {
    60 // 1 minutes
}

#[cfg(test)]
mod tests {
    use super::*;
    use kube::CustomResourceExt;

    #[test]
    fn test_mcp_server_crd() {
        let crd = MCPServer::crd();
        assert_eq!(crd.spec.names.kind, "MCPServer");
        assert_eq!(crd.spec.names.plural, "mcpservers");
        assert_eq!(crd.spec.names.singular, Some("mcpserver".to_string()));
        assert_eq!(crd.spec.group, "unmcp.dev");
        assert_eq!(crd.spec.versions[0].name, "v1");
    }

    #[test]
    fn test_mcp_server_spec_defaults() {
        let spec = MCPServerSpec::default();
        assert_eq!(spec.pool, "default");
        assert_eq!(spec.image, "mcp/fetch:latest");
        assert_eq!(spec.command, None);
        assert_eq!(spec.args, None);
        assert_eq!(spec.env.len(), 0);
        assert_eq!(spec.transport, MCPServerTransport::Stdio);
        assert_eq!(spec.idle_timeout, 60);
    }

    #[test]
    fn test_mcp_server_json_deserialization() {
        let json = r#"
        {
            "apiVersion": "unmcp.dev/v1",
            "kind": "MCPServer",
            "metadata": {
                "name": "test-server",
                "namespace": "default"
            },
            "spec": {
                "pool": "test-pool",
                "image": "mcp/fetch:latest",
                "command": ["mcp", "--config", "/etc/mcp/config.yaml"],
                "args": ["--verbose"],
                "env": [
                    {"name": "ENV_VAR", "value": "value"}
                ],
                "transport": {"type": "sse", "port": 8080},
                "idleTimeout": 120
            }
        }
        "#;

        let server: MCPServer = serde_json::from_str(json).unwrap();
        assert_eq!(server.spec.pool, "test-pool");
        assert_eq!(server.spec.image, "mcp/fetch:latest");
        assert_eq!(
            server.spec.command,
            Some(vec![
                "mcp".to_string(),
                "--config".to_string(),
                "/etc/mcp/config.yaml".to_string()
            ])
        );
        assert_eq!(server.spec.args, Some(vec!["--verbose".to_string()]));
        assert_eq!(server.spec.env[0].name, "ENV_VAR");
        assert_eq!(server.spec.env[0].value, Some("value".to_string()));
        assert_eq!(
            server.spec.transport,
            MCPServerTransport::Sse { port: 8080 }
        );
        assert_eq!(server.spec.idle_timeout, 120);
        assert_eq!(server.metadata.name, Some("test-server".to_string()));
        assert_eq!(server.metadata.namespace, Some("default".to_string()));
    }

    #[test]
    fn test_mcp_server_json_serialization() {
        let server = MCPServer {
            metadata: kube::core::ObjectMeta {
                name: Some("test-server".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: MCPServerSpec {
                pool: "test-pool".to_string(),
                image: "mcp/fetch:latest".to_string(),
                command: Some(vec![
                    "mcp".to_string(),
                    "--config".to_string(),
                    "/etc/mcp/config.yaml".to_string(),
                ]),
                args: Some(vec!["--verbose".to_string()]),
                env: vec![v1::EnvVar {
                    name: "ENV_VAR".to_string(),
                    value: Some("value".to_string()),
                    ..Default::default()
                }],
                transport: MCPServerTransport::Sse { port: 8080 },
                idle_timeout: 120,
            },
            status: None,
        };

        let json = serde_json::to_string(&server).unwrap();
        assert!(json.contains("\"name\":\"test-server\""));
        assert!(json.contains("\"namespace\":\"default\""));
        assert!(json.contains("\"pool\":\"test-pool\""));
        assert!(json.contains("\"image\":\"mcp/fetch:latest\""));
        assert!(json.contains("\"command\":[\"mcp\",\"--config\",\"/etc/mcp/config.yaml\"]"));
        assert!(json.contains("\"args\":[\"--verbose\"]"));
        assert!(json.contains("\"env\":[{\"name\":\"ENV_VAR\",\"value\":\"value\"}]"));
        assert!(json.contains("\"transport\":{\"type\":\"sse\",\"port\":8080}"));
        assert!(json.contains("\"idleTimeout\":120"));
    }
}
