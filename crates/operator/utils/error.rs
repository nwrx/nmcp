use core::result::Result as StdResult;
use serde_yaml::Error as YamlError;
use std::io::Error as IoError;
use thiserror::Error;

/// Errors that can occur when working with MCP resources
#[derive(Error, Debug)]
pub enum Error {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("{0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Could not serialize the object into JSON: {0}")]
    SerializeJsonError(#[source] serde_json::Error),

    #[error("Could not deserialize the object from JSON: {0}")]
    DeserializeJsonError(#[source] serde_json::Error),

    #[error("Could not serialize the object into YAML: {0}")]
    SerializeYamlError(#[source] YamlError),

    #[error("Unsupported output format: {0}")]
    UnsupportedFormat(String),

    #[error("Failed to write output to file: {0}")]
    WriteError(#[source] IoError),

    ///////////////////////////////////////////////////////////
    /// Transparent errors
    ///////////////////////////////////////////////////////////

    #[error("Failed to create config from kubeconfig: {0}")]
    KubeconfigError(#[from] kube::config::KubeconfigError),

    #[error("[KubeError]: {0} ({0:?})")]
    KubeError(#[from] kube::Error),

    ///////////////////////////////////////////////////////////
    /// Kubernetes client errors
    ///////////////////////////////////////////////////////////

    #[error("Kubeconfig path does not exist: {0}")]
    KubeconfigPathNotExists(#[source] IoError),

    #[error("Error while parsing kubeconfig: {0}")]
    KubeConfigParseError(#[source] YamlError),

    #[error("Error while creating the kube client: {0}")]
    KubeClientError(#[source] kube::Error),

    #[error("Failed to load in-cluster config: {0}")]
    InClusterError(#[from] kube::config::InClusterError),

    ///////////////////////////////////////////////////////////
    /// Pool errors
    ///////////////////////////////////////////////////////////

    #[error("Failed to list MCPPool resources: {0}")]
    PoolListError(#[source] kube::Error),

    #[error("Failed to list MCPServer resources: {0}")]
    PoolServerListError(#[source] kube::Error),

    #[error("Could not find MCPServer resource with name: {0}")]
    PoolServerNotFoundError(String),

    #[error("Failed to create MCPPool resource: {0}")]
    PoolCreateError(#[source] kube::Error),

    #[error("Failed to update MCPPool resource: {0}")]
    PoolUpdateError(#[source] kube::Error),

    #[error("Failed to delete MCPPool resource: {0}")]
    PoolDeleteError(#[source] kube::Error),

    #[error("Failed to get MCPPool resource: {0:?}")]
    PoolGetError(#[source] kube::Error),

    ///////////////////////////////////////////////////////////
    /// MCPServer errors
    ///////////////////////////////////////////////////////////

    #[error("Failed to get the Pod assigned to the MCPServer")]
    ServerPodNotFound(#[source] kube::Error),

    #[error("{0}")]
    ServerPodTemplate(#[source] kube::Error),

    #[error("Failed to delete the Pod assigned to the MCPServer")]
    ServerPodDelete(#[source] kube::Error),

    #[error("Failed to get the Service assigned to the MCPServer")]
    ServerServiceNotFound(#[source] kube::Error),

    #[error("{0}")]
    ServerServiceTemplate(#[source] kube::Error),

    #[error("Failed to delete the Service assigned to the MCPServer")]
    ServerServiceDelete(#[source] kube::Error),

    #[error("Failed to get the pool assigned to the MCPServer")]
    ServerPoolNotFound(#[source] kube::Error),

    #[error("Failed to create MCPServer resource: {0}")]
    ServerCreateFailed(#[source] kube::Error),

    #[error("Failed to update MCPServer resource: {0}")]
    ServerUpdateFailed(#[source] kube::Error),

    #[error("Failed to delete MCPServer resource: {0}")]
    ServerDeleteFailed(#[source] kube::Error),

    #[error("Failed to get MCPServer resource: {0}")]
    ServerGetFailed(#[source] kube::Error),

    #[error("MCPServer with UID {0} not found")]
    ServerNotFound(String),

    #[error("Failed to attach to the Pod TTY stream: {0}")]
    ServerStreamError(#[source] kube::Error),

    ///////////////////////////////////////////////////////////
    /// MCPServer errors
    ///////////////////////////////////////////////////////////

    #[error("Failed to aquire the lock: {0}")]
    TryLockError(#[from] tokio::sync::TryLockError),

    ///////////////////////////////////////////////////////////
    /// Axum errors
    ///////////////////////////////////////////////////////////

    #[error("Failed to create the Axum server: {0}")]
    AxumError(#[from] axum::Error),
}

/// Shorthand Result type for MCP operations
pub type Result<T, E = Error> = StdResult<T, E>;
