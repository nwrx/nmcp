use super::{MCPServer, MCPServerList, MCPServerSpec, MCPServerStatus};
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

impl MCPServerBody {
    pub fn example() -> Self {
        MCPServerBody {
            id: Uuid::new_v4().to_string(),
            name: "example-server".to_string(),
            spec: MCPServerSpec::default(),
            status: MCPServerStatus::default(),
        }
    }
}

impl From<MCPServer> for MCPServerBody {
    fn from(server: MCPServer) -> Self {
        MCPServerBody {
            id: server.metadata.uid.clone().unwrap_or_default(),
            name: server.name_any(),
            spec: server.spec.clone(),
            status: server.status.unwrap_or_default(),
        }
    }
}

impl IntoResponse for MCPServer {
    fn into_response(self) -> Response {
        let body: MCPServerBody = self.into();
        (StatusCode::OK, Json(body)).into_response()
    }
}

impl IntoResponse for MCPServerList {
    fn into_response(self) -> Response {
        let servers = self
            .0
            .items
            .into_iter()
            .map(|server| server.into())
            .collect::<Vec<MCPServerBody>>();
        (StatusCode::OK, Json(servers)).into_response()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::{MCPServerSpec, MCPServerTransport};

    // #[test]
    // fn test_mcp_server_response_from_server() {
    //     let response = MCPServer::new(
    //         "test-server",
    //         MCPServerSpec {
    //             pool: "test-pool".to_string(),
    //             transport: MCPServerTransport::Sse { port: 8080 },
    //             ..Default::default()
    //         },
    //     )
    //     .into_response()
    //     .body();

    //     assert_eq!(body.id, "12345678-1234-1234-1234-123456789012");
    //     assert_eq!(response.name, "test-server");
    //     assert_eq!(response.pool, "test-pool");
    //     assert_eq!(response.namespace, "default");
    //     assert_eq!(response.transport_port, Some(8080));
    //     assert_eq!(response.transport_type, "sse");
    //     assert_eq!(response.url, "/api/v1/servers/test-server");
    //     assert_eq!(response.url_sse, "/api/v1/servers/test-server/sse");
    //     assert_eq!(response.url_messages, "/api/v1/servers/test-server/message");
    //     assert_eq!(response.pool_url, "/api/v1/pools/test-pool");
    // }

    // #[test]
    // fn test_mcp_server_response_from_server_stdio_transport() {
    //     let response = MCPServer::new(
    //         "test-server",
    //         MCPServerSpec {
    //             pool: "test-pool".to_string(),
    //             transport: MCPServerTransport::Stdio,
    //             ..Default::default()
    //         },
    //     )
    //     .into_response();
    //     assert_eq!(response.transport_port, None);
    //     assert_eq!(response.transport_type, "stdio");
    // }
}
