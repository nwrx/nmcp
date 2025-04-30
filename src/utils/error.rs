use std::io::Error as IoError;
use kube::config::KubeconfigError;
use kube::Error as KubeError;
use thiserror::Error;

/// Controller-specific errors
#[derive(Debug, Error)]
pub enum Error {
    
    /// General controller errors
    #[error("{0}")]
    Generic(String),

    /// Kubernetes API errors
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Reconciliation errors
    #[error("Reconciliation error: {0}")]
    ReconciliationError(String),

    ///////////////////////////////
    /// Finalizer-related errors
    ///////////////////////////////

    #[error("Failed to add finalizer '{finalizer}' to resource '{name}': {message}")]
    FinalizerError {
        name: String,
        finalizer: String,
        message: String,
    },

    ///////////////////////////////
    /// Kubeconfig-related errors
    ///////////////////////////////

    #[error("Failed to read kubeconfig file at path '{path}': {error}")]
    KubeconfigReadError {
        path: String,
        error: IoError,
    },

    #[error("Failed to parse kubeconfig file at '{path}': {error}")]
    KubeconfigParseError {
        path: String,
        error: KubeconfigError,
    },

    #[error("Failed to create Kubernetes client configuration from kubeconfig at '{path}': {error}")]
    KubeconfigConfigError {
        path: String,
        error: KubeconfigError,
    },

    #[error("Failed to create Kubernetes client from configuration: {error}")]
    KubeClientCreationError {
        path: String,
        error: KubeError,
    },

    #[error("Failed to create Kubernetes client using default kubeconfig: {error}")]
    KubeconfigError {
        error: KubeError,
    },
}

/// Result type for controller operations
pub type Result<T, E = Error> = std::result::Result<T, E>;
