use std::any::Any;
use thiserror::Error;

/// Errors that can occur when working with MCP resources
#[derive(Error, Debug)]
pub enum ErrorInner {
    #[error("{0}")]
    Generic(String),

    #[error("{0}")]
    IoError(std::io::Error),

    #[error("{0}")]
    JsonError(#[from] serde_json::Error),

    #[error("{0}")]
    YamlError(#[from] serde_yml::Error),

    #[error("{0}")]
    KubeconfigError(#[from] kube::config::KubeconfigError),

    #[error("{0}")]
    KubeError(#[source] kube::Error),

    #[error("{0}")]
    KubeClientError(kube_client::Error),

    #[error("{0}")]
    AxumError(#[from] axum::Error),

    #[error("{0}")]
    TryLockError(#[from] tokio::sync::TryLockError),

    #[error("{0}")]
    InClusterError(#[from] kube::config::InClusterError),

    #[error("{0}")]
    Timeout(#[from] tokio::time::error::Elapsed),

    #[error("{0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("{0}")]
    MPSCSendError(tokio::sync::mpsc::error::SendError<Box<dyn Any + Send + Sync>>),

    #[error("{0}")]
    BroadcastRecvError(tokio::sync::broadcast::error::RecvError),

    #[error("{0}")]
    BroadcastSendError(tokio::sync::broadcast::error::SendError<Box<dyn Any + Send + Sync>>),
}
