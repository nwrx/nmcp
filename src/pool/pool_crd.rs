use schemars::JsonSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use kube::CustomResource;
use crate::server::{NetworkingConfig, ProbeConfig, Resources, SecurityContext};

/// McpPool custom resource definition
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema, Default)]
#[kube(
    group = "unmcp.dev",
    version = "v1",
    kind = "MCPPool",
    singular = "mcppool",
    plural = "mcppools",
    shortname = "mcpp",
    namespaced,
    status = "McpPoolStatus",
    printcolumn = r#"{"name":"Available", "type":"integer", "jsonPath":".status.availableServers"}"#,
    printcolumn = r#"{"name":"In Use", "type":"integer", "jsonPath":".status.inUseServers"}"#,
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct McpPoolSpec {
    /// Maximum servers in the pool
    #[serde(default = "default_max_servers")]
    pub max_servers: i32,

    /// Minimum servers to maintain
    #[serde(default = "default_min_servers")]
    pub min_servers: i32,

    /// Server idle timeout in seconds
    #[serde(default = "default_server_timeout")]
    pub server_timeout: i32,

    /// Delay before scaling down (seconds)
    #[serde(default = "default_scale_down_delay")]
    pub scale_down_delay: i32,

    /// Auto-scaling configuration
    #[serde(default)]
    pub autoscaling: AutoscalingConfig,

    /// Node selector
    #[serde(default)]
    pub node_selector: Option<std::collections::BTreeMap<String, String>>,

    /// Pod tolerations
    #[serde(default)]
    pub tolerations: Vec<Toleration>,

    /// Affinity configuration
    pub affinity: Option<AffinityConfig>,

    /// Priority class name
    pub priority_class_name: Option<String>,

    /// Server defaults
    #[serde(default)]
    pub server_defaults: ServerDefaults,

    /// Upgrade strategy
    #[serde(default)]
    pub upgrade_strategy: UpgradeStrategy,

    /// Cleanup policy
    #[serde(default)]
    pub cleanup_policy: CleanupPolicy,
}

/// Default maximum servers
fn default_max_servers() -> i32 {
    100
}

/// Default minimum servers
fn default_min_servers() -> i32 {
    10
}

/// Default server timeout
fn default_server_timeout() -> i32 {
    300
}

/// Default scale down delay
fn default_scale_down_delay() -> i32 {
    60
}

////////////////////////////////////////

/// Auto-scaling configuration
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AutoscalingConfig {
    /// Whether auto-scaling is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Target CPU utilization percentage
    #[serde(default = "default_target_cpu")]
    pub target_cpu_utilization: i32,

    /// Target memory utilization percentage
    #[serde(default = "default_target_memory")]
    pub target_memory_utilization: i32,

    /// Scale up cooldown in seconds
    #[serde(default = "default_scale_up_cooldown")]
    pub scale_up_cooldown: i32,

    /// Scale down cooldown in seconds
    #[serde(default = "default_scale_down_cooldown")]
    pub scale_down_cooldown: i32,
}

/// Default target CPU utilization percentage
fn default_target_cpu() -> i32 {
    70
}

/// Default target memory utilization percentage
fn default_target_memory() -> i32 {
    80
}

/// Default scale up cooldown period
fn default_scale_up_cooldown() -> i32 {
    60
}

/// Default scale down cooldown period
fn default_scale_down_cooldown() -> i32 {
    300
}

/// Pod toleration
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Toleration {
    /// Toleration key
    pub key: Option<String>,

    /// Toleration operator
    pub operator: Option<String>,

    /// Toleration value
    pub value: Option<String>,

    /// Toleration effect
    pub effect: Option<String>,
}

/// Affinity configuration
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AffinityConfig {
    /// Pod anti-affinity
    pub pod_anti_affinity: Option<PodAntiAffinityConfig>,
}

/// Pod anti-affinity configuration
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodAntiAffinityConfig {
    /// Preferred during scheduling, ignored during execution
    pub preferred_during_scheduling_ignored_during_execution: Option<Vec<WeightedPodAffinityTerm>>,
}

/// Weighted pod affinity term
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WeightedPodAffinityTerm {
    /// Weight
    pub weight: i32,

    /// Pod affinity term
    pub pod_affinity_term: PodAffinityTerm,
}

/// Pod affinity term
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodAffinityTerm {
    /// Label selector
    pub label_selector: LabelSelector,

    /// Topology key
    pub topology_key: String,
}

/// Label selector
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LabelSelector {
    /// Match labels
    pub match_labels: std::collections::BTreeMap<String, String>,
}

/// Server defaults
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServerDefaults {
    /// Default image
    pub image: Option<String>,

    /// Default resources
    pub resources: Option<Resources>,

    /// Default security context
    pub security_context: Option<SecurityContext>,

    /// Default networking
    pub networking: Option<NetworkingConfig>,

    /// Default liveness probe
    pub liveness_probe: Option<ProbeConfig>,
}

/// Upgrade strategy
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeStrategy {
    /// Strategy type
    #[serde(default = "default_strategy_type")]
    pub type_: String,

    /// Max surge
    pub max_surge: Option<String>,

    /// Max unavailable
    pub max_unavailable: Option<String>,
}

/// Default strategy type
fn default_strategy_type() -> String {
    "RollingUpdate".to_string()
}

/// Cleanup policy
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPolicy {
    /// Whether to delete orphaned servers
    #[serde(default = "default_true")]
    pub delete_orphaned_servers: bool,

    /// Grace period for termination
    #[serde(default = "default_terminate_grace")]
    pub terminate_grace_period: i32,
}

/// Default true value
fn default_true() -> bool {
    true
}

/// Default terminate grace period
fn default_terminate_grace() -> i32 {
    30
}

/// McpPool status
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpPoolStatus {
    /// Available servers count
    pub available_servers: Option<i32>,

    /// In use servers count
    pub in_use_servers: Option<i32>,

    /// Pending servers count
    pub pending_servers: Option<i32>,

    /// Status conditions
    #[serde(default)]
    pub conditions: Vec<Condition>,

    /// Pool metrics
    pub metrics: Option<PoolMetrics>,

    /// Last scale up time
    pub last_scale_up_time: Option<DateTime<Utc>>,

    /// Last scale down time
    pub last_scale_down_time: Option<DateTime<Utc>>,
}

/// Pool metrics
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PoolMetrics {
    /// Average CPU utilization
    pub average_cpu_utilization: Option<String>,

    /// Average memory utilization
    pub average_memory_utilization: Option<String>,

    /// Total requests
    pub total_requests: Option<i64>,

    /// Active connections
    pub active_connections: Option<i32>,
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


#[cfg(test)]
mod tests {
    use assert_json_diff::assert_json_eq;
    use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
    use kube::CustomResourceExt as _;
    use schemars::{schema::RootSchema, schema_for};
    use crate::{pool::{MCP_POOL_CRD_FIXTURE, MCP_POOL_CRD_SCHEMA_FIXTURE}, MCPPool};

    #[test]
    fn test_mcp_pool_crd() {
        let result = MCPPool::crd();
        let expected: CustomResourceDefinition = serde_json::from_str(MCP_POOL_CRD_FIXTURE).unwrap();
        assert_json_eq!(result, expected);
    }

    #[test]
    fn test_mcp_pool_schema() {
        let result = schema_for!(MCPPool);
        let expected: RootSchema = serde_json::from_str(MCP_POOL_CRD_SCHEMA_FIXTURE).unwrap();
        assert_json_eq!(result, expected);
    }
}
