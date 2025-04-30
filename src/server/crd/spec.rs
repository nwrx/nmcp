use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use super::{MCPServerEnvVar, MCPServerStatus, MCPServerTransport};

/// MCPServer custom resource definition
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema, Default)]
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
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#,
    crates(serde_json = "::serde_json"),
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
    pub env: Vec<MCPServerEnvVar>,

    /// The MCP transport used by the server. This will be used to determine how the server
    /// communicates with other components in the system, such as the database or other servers.
    /// Can either be "sse" (HTTP) or "stdio" (STDIN/STDOUT).
    #[serde(default = "default_transport")]
    pub transport: Option<MCPServerTransport>,
}

/// Default transport configuration
fn default_transport() -> Option<MCPServerTransport> {
    Some(MCPServerTransport::default())
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
fn default_env() -> Vec<MCPServerEnvVar> {
    vec![]
}

/// Default command
fn default_command() -> Option<Vec<String>> {
    None
}
