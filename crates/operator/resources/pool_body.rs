use super::{MCPPool, MCPPoolList, MCPPoolSpec, MCPPoolStatus};
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

    #[serde(flatten)]
    pub status: MCPPoolStatus,
}

impl MCPPoolBody {
    /// Creates an example `MCPPoolBody` instance with default values.
    pub fn example() -> Self {
        MCPPoolBody {
            id: Uuid::new_v4().to_string(),
            name: "example-pool".to_string(),
            spec: MCPPoolSpec::default(),
            status: MCPPoolStatus::default(),
        }
    }
}

impl From<MCPPool> for MCPPoolBody {
    fn from(pool: MCPPool) -> Self {
        MCPPoolBody {
            id: pool.metadata.uid.clone().unwrap_or_default(),
            name: pool.name_any(),
            spec: pool.spec.clone(),
            status: pool.status.unwrap_or_default(),
        }
    }
}

impl IntoResponse for MCPPool {
    fn into_response(self) -> Response {
        let body: MCPPoolBody = self.into();
        (StatusCode::OK, Json(body)).into_response()
    }
}

impl IntoResponse for MCPPoolList {
    fn into_response(self) -> Response {
        let servers = self
            .0
            .items
            .into_iter()
            .map(|server| server.into())
            .collect::<Vec<MCPPoolBody>>();
        (StatusCode::OK, Json(servers)).into_response()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::MCPPoolStatus;
//     use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

//     #[test]
//     fn test_mcp_pool_response_from_pool_no_status() {
//         let response = MCPPool {
//             metadata: ObjectMeta {
//                 name: Some("test-pool".to_string()),
//                 namespace: Some("default".to_string()),
//                 uid: Some("12345678-1234-1234-1234-123456789012".to_string()),
//                 ..Default::default()
//             },
//             spec: Default::default(),
//             status: None,
//         }
//         .into_response(None);
//         assert_eq!(response.id, "12345678-1234-1234-1234-123456789012");
//         assert_eq!(response.name, "test-pool");
//         assert_eq!(response.namespace, "default");
//         assert_eq!(response.active_servers_count, 0);
//         assert_eq!(response.pending_servers_count, 0);
//         assert_eq!(response.unmanaged_servers_count, 0);
//         assert_eq!(response.managed_servers_count, 0);
//         assert_eq!(response.total_servers_count, 0);
//     }

//     #[test]
//     fn test_mcp_pool_response_from_pool() {
//         let pool = MCPPool {
//             metadata: ObjectMeta {
//                 name: Some("test-pool".to_string()),
//                 namespace: Some("default".to_string()),
//                 uid: Some("12345678-1234-1234-1234-123456789012".to_string()),
//                 ..Default::default()
//             },
//             spec: Default::default(),
//             status: Some(MCPPoolStatus {
//                 active_servers_count: 5,
//                 pending_servers_count: 2,
//                 unmanaged_servers_count: 1,
//                 managed_servers_count: 7,
//                 total_servers_count: 8,
//             }),
//         };

//         let servers = vec![
//             MCPServer::new("s1", Default::default()),
//             MCPServer::new("s2", Default::default()),
//             MCPServer::new("s3", Default::default()),
//         ];
//         let response = pool.into_response(Some(servers));
//         assert_eq!(response.id, "12345678-1234-1234-1234-123456789012");
//         assert_eq!(response.name, "test-pool");
//         assert_eq!(response.namespace, "default");
//         assert_eq!(response.max_servers_limit, 100); // Default value
//         assert_eq!(response.max_servers_active, 100); // Default value
//         assert_eq!(response.default_idle_timeout, 60); // Default value
//         assert_eq!(response.active_servers_count, 5);
//         assert_eq!(response.pending_servers_count, 2);
//         assert_eq!(response.unmanaged_servers_count, 1);
//         assert_eq!(response.managed_servers_count, 7);
//         assert_eq!(response.total_servers_count, 8);
//         assert_eq!(response.url, "/api/v1/pools/test-pool");
//         assert_eq!(response.servers.len(), 3);
//         assert_eq!(response.servers[0], "/api/v1/servers/s1");
//         assert_eq!(response.servers[1], "/api/v1/servers/s2");
//         assert_eq!(response.servers[2], "/api/v1/servers/s3");
//     }
// }
