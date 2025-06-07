use crate::{MCPPool, MCPPoolSpec, MCPPoolStatus};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use kube::ResourceExt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MCPPoolBody {
    /// Unique identifier for the pool. This corresponds to the metadata.uid field of
    /// the Kubernetes resource associated with this pool and is used to uniquely identify
    /// the pool within the Kubernetes cluster.
    pub id: String,

    /// Name of the `MCPPool`. This is a human-readable name for the pool and is used to
    /// identify the pool in user interfaces and logs. It is not guaranteed to be unique
    /// across different namespaces.
    pub name: String,

    #[serde(flatten)]
    pub spec: MCPPoolSpec,

    /// Status of the `MCPPool`. This provides information about the current state of the
    /// pool, including the number of active, pending, unmanaged, and managed servers,
    /// as well as the total number of servers in the pool.
    pub status: MCPPoolStatus,
}

impl From<MCPPool> for MCPPoolBody {
    fn from(pool: MCPPool) -> Self {
        Self {
            id: pool.metadata.uid.clone().unwrap_or_default(),
            name: pool.name_any(),
            spec: pool.spec.clone(),
            status: pool.status.unwrap_or_default(),
        }
    }
}

impl Default for MCPPoolBody {
    /// Creates an example `MCPPoolBody` instance with default values.
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "example-pool".to_string(),
            spec: MCPPoolSpec::default(),
            status: MCPPoolStatus::default(),
        }
    }
}

impl IntoResponse for MCPPool {
    fn into_response(self) -> Response {
        let body: MCPPoolBody = self.into();
        (StatusCode::OK, Json(body)).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MCPPoolList(pub Vec<MCPPool>);
impl IntoResponse for MCPPoolList {
    fn into_response(self) -> Response {
        let servers = self
            .0
            .into_iter()
            .map(|server| server.into())
            .collect::<Vec<MCPPoolBody>>();
        (StatusCode::OK, Json(servers)).into_response()
    }
}
