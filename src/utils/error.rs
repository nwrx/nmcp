//! Error types for the controller crate.

use kube::config::KubeconfigError;
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
    
    /// Finalizer errors
    #[error("Finalizer error: {0}")]
    FinalizerError(String),

    ////////////////////////////////////////////////////////////////////
    /// Kubeconfig-related errors
    ////////////////////////////////////////////////////////////////////

    #[error("Failed to read kubeconfig file at path '{path}': {error}")]
    KubeconfigReadError {
        path: String,
        error: std::io::Error,
    },

    #[error("Failed to parse kubeconfig file at '{path}': {error}")]
    KubeconfigParseError {
        path: String,
        error: KubeconfigError,
    },

    #[error("Failed to create Kubernetes client configuration from kubeconfig at '{path}': {error}")]
    KubeconfigConfigError {
        path: String,
        error: kube::config::KubeconfigError,
    },

    #[error("Failed to create Kubernetes client from configuration: {error}")]
    KubeClientCreationError {
        path: String,
        error: kube::Error,
    },

    #[error("Failed to create Kubernetes client using default kubeconfig: {error}")]
    KubeconfigError {
        error: kube::Error,
    },
}

/// Result type for controller operations
pub type Result<T, E = Error> = std::result::Result<T, E>;
