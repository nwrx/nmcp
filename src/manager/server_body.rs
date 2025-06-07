use crate::{MCPServer, MCPServerSpec, MCPServerStatus};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use kube::api::ResourceExt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MCPServerBody {
    /// Unique identifier for the server
    pub id: String,

    /// Name of the server
    pub name: String,

    #[serde(flatten)]
    pub spec: MCPServerSpec,

    /// Status of the server
    pub status: MCPServerStatus,
}

impl From<MCPServer> for MCPServerBody {
    fn from(server: MCPServer) -> Self {
        Self {
            id: server.metadata.uid.clone().unwrap_or_default(),
            name: server.name_any(),
            spec: server.spec.clone(),
            status: server.status.unwrap_or_default(),
        }
    }
}

impl Default for MCPServerBody {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "example-server".to_string(),
            spec: MCPServerSpec::default(),
            status: MCPServerStatus::default(),
        }
    }
}

impl IntoResponse for MCPServer {
    fn into_response(self) -> Response {
        let body: MCPServerBody = self.into();
        (StatusCode::OK, Json(body)).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MCPServerList(pub Vec<MCPServer>);

impl IntoResponse for MCPServerList {
    fn into_response(self) -> Response {
        let servers = self
            .0
            .into_iter()
            .map(|server| server.into())
            .collect::<Vec<MCPServerBody>>();
        (StatusCode::OK, Json(servers)).into_response()
    }
}
