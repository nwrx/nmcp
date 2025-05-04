use crate::MCPPoolStatus;
use k8s_openapi::api::core::v1;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    status = "MCPPoolStatus",
    printcolumn = r#"{"name":"In Use", "type":"integer", "jsonPath":".status.serverInUse"}"#,
    printcolumn = r#"{"name":"Waiting", "type":"integer", "jsonPath":".status.serverWaiting"}"#,
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct MCPPoolSpec {
    /// Maximum amount of MCPServer resources that can be managed by this MCPPool. After
    /// this limit is reached, the overflow servers will be marked as "ignored" and no Pod
    /// or Service resources will be created for them until older MCPServer resources are
    /// deleted.
    #[serde(default = "default_max_servers")]
    pub max_servers_limit: i32,

    /// The maxcimum number of concurrent active servers that can be created in the pool.
    /// After this limit is reached, the overflow servers will be marked as "waiting" and
    /// no Pod or Service resources will be created for them until Pod and Service resources
    /// are deleted by the operator.
    #[serde(default = "default_max_servers")]
    pub max_servers_active: i32,

    /// The default resource requirements for each server in the pool. This will be used to
    /// determine the resource limits and requests for each server's pod. This is to
    /// ensure that each server has the necessary resources to run efficiently and
    /// effectively. This is also to prevent the pool from overwhelming the system with
    /// too many servers at once.
    #[serde(default)]
    pub default_resources: v1::ResourceRequirements,

    /// The default time in seconds that a server is allowed to run without receiving
    /// any requests before it's terminated. This helps to conserve resources by
    /// shutting down idle servers.
    #[serde(default = "default_idle_timeout")]
    pub default_idle_timeout: i32,
}

/// Default maximum servers
fn default_max_servers() -> i32 {
    100
}

/// Default idle timeout in seconds
fn default_idle_timeout() -> i32 {
    60 // 1 minutes
}
