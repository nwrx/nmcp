use core::result::Result as StdResult;
use serde_yaml::Error as YamlError;
use std::io::Error as IoError;
use thiserror::Error;

/// Errors that can occur when working with MCP resources
#[derive(Error, Debug)]
pub enum Error {
    ///////////////////////////////////////////////////////////
    /// Transparent errors
    ///////////////////////////////////////////////////////////

    #[error("Failed to create config from kubeconfig: {0}")]
    KubeconfigError(#[from] kube::config::KubeconfigError),

    #[error("Kubernetes client error: {0}")]
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

    ///////////////////////////////////////////////////////////
    /// MCPServer errors
    ///////////////////////////////////////////////////////////

    #[error("Failed to get the Pod assigned to the MCPServer")]
    ServerPodNotFoundError(#[source] kube::Error),

    #[error("Failed to apply the Pod template assigned to the MCPServer")]
    ServerPodTemplateError(#[source] kube::Error),

    #[error("Failed to delete the Pod assigned to the MCPServer")]
    ServerPodDeleteError(#[source] kube::Error),

    #[error("Failed to get the Service assigned to the MCPServer")]
    ServerServiceNotFoundError(#[source] kube::Error),

    #[error("Failed to apply the Service template assigned to the MCPServer")]
    ServerServiceTemplateError(#[source] kube::Error),

    #[error("Failed to delete the Service assigned to the MCPServer")]
    ServerServiceDeleteError(#[source] kube::Error),

    #[error("Failed to get the pool assigned to the MCPServer")]
    ServerPoolNotFoundError(#[source] kube::Error),

    ///////////////////////////////////////////////////////////
    /// Axum errors
    ///////////////////////////////////////////////////////////

    #[error("Failed to create the Axum server: {0}")]
    AxumError(#[from] axum::Error),
}

/// Shorthand Result type for MCP operations
pub type Result<T, E = Error> = StdResult<T, E>;
