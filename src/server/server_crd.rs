use schemars::JsonSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use kube::CustomResource;

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
    printcolumn = r#"{"name":"Status", "type":"string", "jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#,
    printcolumn = r#"{"name":"Type", "type":"string", "jsonPath":".spec.metadata.serverType"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct MCPServerSpec {
    /// Server version
    #[serde(default = "default_version")]
    pub version: String,
    
    /// Container image to use
    #[serde(default = "default_image")]
    pub image: String,
    
    /// Reference to McpPool resource
    #[serde(default)]
    pub pool: String,
    
    /// Server command configuration
    pub server: ServerConfig,
    
    /// Resource requirements
    #[serde(default)]
    pub resources: Resources,
    
    /// Liveness probe configuration
    #[serde(default)]
    pub liveness_probe: Option<ProbeConfig>,
    
    /// Readiness probe configuration
    #[serde(default)]
    pub readiness_probe: Option<ProbeConfig>,
    
    /// Security context
    #[serde(default)]
    pub security_context: Option<SecurityContext>,
    
    /// Server metadata
    #[serde(default)]
    pub metadata: ServerMetadata,
    
    /// Network configuration
    #[serde(default)]
    pub networking: NetworkingConfig,
    
    /// Storage configuration
    #[serde(default)]
    pub storage: StorageConfig,
}

/// Default version string
fn default_version() -> String {
    "1.0.0".to_string()
}

/// Default image string
fn default_image() -> String {
    "mcp/time:latest".to_string()
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub type_: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_transition_time: Option<DateTime<Utc>>,
}

/// Server command configuration
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    /// Main command to execute
    pub command: String,
    
    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,
    
    /// Environment variables
    #[serde(default)]
    pub env: std::collections::BTreeMap<String, String>,
}

/// Resource requirements
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Resources {
    /// Resource limits
    #[serde(default)]
    pub limits: Option<ResourceQuantity>,
    
    /// Resource requests
    #[serde(default)]
    pub requests: Option<ResourceQuantity>,
}

/// Resource quantity
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResourceQuantity {
    /// CPU quantity
    pub cpu: Option<String>,
    
    /// Memory quantity
    pub memory: Option<String>,
}

/// Probe configuration
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProbeConfig {
    /// HTTP get probe
    pub http_get: HttpGetProbe,
    
    /// Initial delay seconds
    #[serde(default = "default_initial_delay")]
    pub initial_delay_seconds: i32,
    
    /// Period seconds
    #[serde(default = "default_period")]
    pub period_seconds: i32,
}

/// Default initial delay value
fn default_initial_delay() -> i32 {
    5
}

/// Default period value 
fn default_period() -> i32 {
    10
}

/// HTTP GET probe
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HttpGetProbe {
    /// Path to probe
    pub path: String,
    
    /// Port to probe
    pub port: i32,
}

/// Security context
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityContext {
    /// Run as non-root
    #[serde(default = "default_true")]
    pub run_as_non_root: bool,
    
    /// Run as user ID
    pub run_as_user: Option<i64>,
    
    /// Allow privilege escalation
    #[serde(default = "default_false")]
    pub allow_privilege_escalation: bool,
    
    /// Seccomp profile
    pub seccomp_profile: Option<SeccompProfile>,
    
    /// Capabilities
    pub capabilities: Option<Capabilities>,
}

/// Default true boolean
fn default_true() -> bool {
    true
}

/// Default false boolean
fn default_false() -> bool {
    false
}

/// Seccomp profile
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SeccompProfile {
    /// Profile type
    #[serde(rename = "type")]
    pub profile_type: String,
}

/// Container capabilities
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    /// Capabilities to drop
    pub drop: Vec<String>,
}

/// Server metadata
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServerMetadata {
    /// Server type
    pub server_type: Option<String>,
    
    /// Server capabilities
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Network configuration
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NetworkingConfig {
    /// Server port
    #[serde(default = "default_port")]
    pub port: i32,

    /// Protocol (HTTP or HTTPS)
    #[serde(default = "default_protocol")]
    pub protocol: String,

    /// Whether to expose externally, will either create a LoadBalancer or ClusterIP service
    /// depending on the value of this field. Keep in mind that exposing externally may incur costs.
    #[serde(default)]
    pub expose_externally: bool,
    
    /// CORS configuration
    pub cors: Option<CorsConfig>,
}

fn default_port() -> i32 {
    8080
}

fn default_protocol() -> String {
    "HTTP".to_string()
}

/// CORS configuration
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CorsConfig {
    /// Allowed origins
    pub allow_origins: Vec<String>,
}

/// Storage configuration
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StorageConfig {
    /// Whether storage is ephemeral
    #[serde(default = "default_true")]
    pub ephemeral: bool,
    
    /// Volume size
    pub volume_size: Option<String>,
}

/// MCPServer status
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MCPServerStatus {
    /// Server phase
    #[serde(default)]
    pub phase: Option<String>,
    
    /// Start time
    pub start_time: Option<DateTime<Utc>>,
    
    /// Status conditions
    #[serde(default)]
    pub conditions: Vec<Condition>,
    
    /// Server endpoint URL
    pub server_endpoint: Option<String>,
    
    /// Server UUID
    pub server_uuid: Option<String>,
    
    /// Server metrics
    pub metrics: Option<ServerMetrics>,
}

/// Server metrics
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServerMetrics {
    /// Request count
    pub request_count: Option<i64>,
    
    /// Active connections
    pub active_connections: Option<i32>,
    
    /// CPU usage
    pub cpu_usage: Option<String>,
    
    /// Memory usage
    pub memory_usage: Option<String>,
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use assert_json_diff::assert_json_eq;
    use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
    use kube::CustomResourceExt as _;
    use schemars::{schema::RootSchema, schema_for};
    use crate::{server::{MCP_SERVER_CRD_FIXTURE, MCP_SERVER_CRD_SCHEMA_FIXTURE}, MCPServer};

    #[test]
    fn test_mcp_server_schema() {
        let result = schema_for!(MCPServer);
        let expected: RootSchema = serde_json::from_str(MCP_SERVER_CRD_SCHEMA_FIXTURE).unwrap();
        assert_json_eq!(result, expected);
    }

    #[test]
    fn test_mcp_server_crd() {
        let result = MCPServer::crd();
        let expected: CustomResourceDefinition = serde_json::from_str(MCP_SERVER_CRD_FIXTURE).unwrap();
        assert_json_eq!(result, expected);
    }
}
