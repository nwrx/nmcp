use axum::{http::StatusCode, response::IntoResponse};
use axum_thiserror::ErrorStatus;
use core::result::Result as StdResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Shorthand Result type for MCP operations
pub type Result<T, E = Error> = StdResult<T, E>;

/// Errors that can occur when working with MCP resources
#[derive(Error, Debug, ErrorStatus)]
pub enum Error {
    ///////////////////////////////////////////////////////////
    /// Transparent errors
    ///////////////////////////////////////////////////////////

    #[error("{0}")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    JsonError(#[from] serde_json::Error),

    #[error("{0}")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    YamlError(#[from] serde_yml::Error),

    #[error("Failed to create config from kubeconfig: {0}")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    KubeconfigError(#[from] kube::config::KubeconfigError),

    #[error("[KubeError]: {0} ({0:?})")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    KubeError(#[from] kube::Error),

    #[error("Failed to create the Axum server: {0}")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    AxumError(#[from] axum::Error),

    #[error("Failed to aquire the lock: {0}")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    TryLockError(#[from] tokio::sync::TryLockError),

    #[error("Failed to load in-cluster config: {0}")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    InClusterError(#[from] kube::config::InClusterError),

    #[error("Kubernetes finalizer error: {0}")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    KubeFinalizerError(#[from] kube::runtime::finalizer::Error<Box<Error>>),

    ///////////////////////////////////////////////////////////
    /// Serialization errors
    ///////////////////////////////////////////////////////////

    #[error("Internal error: {0}")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    Internal(String),

    #[error("Unsupported output format: {0}")]
    #[status(StatusCode::BAD_REQUEST)]
    UnsupportedFormat(String),

    #[error("Failed to write output to file: {0}")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    WriteError(#[source] std::io::Error),

    ///////////////////////////////////////////////////////////
    /// Kubernetes client errors
    ///////////////////////////////////////////////////////////

    #[error("Kubeconfig path does not exist: {0}")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    KubeconfigPathNotExists(#[source] std::io::Error),

    #[error("Error while parsing kubeconfig: {0}")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    KubeConfigParseError(#[source] serde_yml::Error),
}

impl Error {
    pub fn to_message(&self) -> String {
        match self {
            Error::KubeError(kube::Error::Api(response)) => response.message.clone(),
            Error::KubeFinalizerError(error) => match error {
                kube::runtime::finalizer::Error::AddFinalizer(error) => {
                    format!("Failed to add finalizer: {error}")
                }
                kube::runtime::finalizer::Error::RemoveFinalizer(error) => {
                    format!("Failed to remove finalizer: {error}")
                }
                kube::runtime::finalizer::Error::ApplyFailed(error) => {
                    format!("Failed to apply finalizer: {}", error.to_message())
                }
                kube::runtime::finalizer::Error::CleanupFailed(error) => {
                    format!("Failed to clean up finalizer: {}", error.to_message())
                }
                kube::runtime::finalizer::Error::UnnamedObject => "Object has no name".to_string(),
                kube::runtime::finalizer::Error::InvalidFinalizer => {
                    "Invalid finalizer".to_string()
                }
            },
            _ => format!("Internal error: {:?}", self.to_string()),
        }
    }
}

/// Error body for the API response
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorBody {
    /// The name or identifier of the error. It more or less corresponds to the underlying
    /// scope or module where the error occurred and is used for debugging.
    pub name: String,

    /// A human-readable message describing the error that occurred. This message is intended
    /// for end-users and should be clear and concise.
    pub message: String,

    /// The HTTP status code associated with the error. This is useful for clients to understand
    /// the nature of the error and how to handle it.
    pub status_code: u16,

    /// A human-readable message describing the status of the error. This message is intended
    /// for end-users and should be clear and concise.
    pub status_message: String,
}

impl From<Error> for ErrorBody {
    fn from(error: Error) -> Self {
        match &error {
            Error::KubeError(kube::Error::Api(response)) => Self {
                name: "KubeError".into(),
                message: response.message.clone(),
                status_code: response.code,
                status_message: response.status.clone(),
            },
            _ => {
                let message = error.to_message();
                let status = error.into_response().status();
                Self {
                    name: "Error".into(),
                    message,
                    status_code: status.as_u16(),
                    status_message: status.canonical_reason().unwrap_or_default().to_string(),
                }
            }
        }
    }
}
