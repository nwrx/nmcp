use crate::MCPServer;
use kube::api::ResourceExt;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

impl MCPServer {
    /// Returns the name of the `v1::Service` for the `MCPServer`.
    pub fn name_service(&self) -> String {
        format!(
            "mcp-server-svc-{}-{}-{}",
            self.spec.pool,
            self.name_any(),
            &self.metadata.uid.clone().unwrap()[..8]
        )
    }

    /// Returns the name of the `v1::Pod` for the `MCPServer`.
    pub fn name_pod(&self) -> String {
        format!(
            "mcp-server-{}-{}-{}",
            self.spec.pool,
            self.name_any(),
            &self.metadata.uid.clone().unwrap()[..8]
        )
    }

    /// Returns the labels for the `MCPServer`.
    pub fn labels(&self) -> BTreeMap<String, String> {
        let uid = self.metadata.uid.clone().unwrap_or_default();
        let mut labels = std::collections::BTreeMap::new();
        labels.insert("app".to_string(), self.name_pod());
        labels.insert("unmcp.dev/uid".to_string(), uid);
        labels.insert("unmcp.dev/pool".to_string(), self.spec.pool.clone());
        labels
    }

    /// Return the URL for the `MCPServer` resource.
    pub fn url(&self) -> String {
        format!("/api/v1/servers/{}", self.name_any())
    }

    /// Returns the URL for the `MCPServer` SSE endpoint.
    pub fn url_sse(&self) -> String {
        format!("{}/sse", self.url())
    }

    /// Returns the URL for the `MCPServer` messages endpoint.
    pub fn url_messages(&self) -> String {
        format!("{}/messages", self.url())
    }

    /// Returns the URL for the `MCPPool` resource.
    pub fn url_pool(&self) -> String {
        format!("/api/v1/pools/{}", self.spec.pool)
    }

    /// Returns the Response for the `MCPServer`.
    pub fn into_response(&self) -> MCPServerResponse {
        MCPServerResponse {
            id: self.metadata.uid.clone().unwrap_or_default(),
            name: self.name_any(),
            pool: self.spec.pool.clone(),
            namespace: self.namespace().unwrap_or_default(),
            transport_port: self.spec.transport.port(),
            transport_type: self.spec.transport.transport_type(),
            url: self.url(),
            url_sse: self.url_sse(),
            url_messages: self.url_messages(),
            pool_url: self.url_pool(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MCPServerResponse {
    pub id: String,
    pub name: String,
    pub pool: String,
    pub namespace: String,
    pub transport_port: Option<u16>,
    pub transport_type: String,
    pub url: String,
    pub url_sse: String,
    pub url_messages: String,
    pub pool_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MCPServerSpec, MCPServerTransport};
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

    #[test]
    fn test_mcp_server_name_service() {
        let server = MCPServer {
            metadata: ObjectMeta {
                name: Some("test-server".to_string()),
                namespace: Some("default".to_string()),
                uid: Some("12345678-1234-1234-1234-123456789012".to_string()),
                ..Default::default()
            },
            spec: MCPServerSpec {
                pool: "test-pool".to_string(),
                ..Default::default()
            },
            status: None,
        };

        let service_name = server.name_service();
        assert_eq!(
            service_name,
            "mcp-server-svc-test-pool-test-server-12345678"
        );
    }

    #[test]
    fn test_mcp_server_name_pod() {
        let server = MCPServer {
            metadata: ObjectMeta {
                name: Some("test-server".to_string()),
                namespace: Some("default".to_string()),
                uid: Some("12345678-1234-1234-1234-123456789012".to_string()),
                ..Default::default()
            },
            spec: MCPServerSpec {
                pool: "test-pool".to_string(),
                ..Default::default()
            },
            status: None,
        };

        let pod_name = server.name_pod();
        assert_eq!(pod_name, "mcp-server-test-pool-test-server-12345678");
    }

    #[test]
    fn test_mcp_server_labels() {
        let server = MCPServer {
            metadata: ObjectMeta {
                name: Some("test-server".to_string()),
                namespace: Some("default".to_string()),
                uid: Some("12345678-1234-1234-1234-123456789012".to_string()),
                ..Default::default()
            },
            spec: MCPServerSpec {
                pool: "test-pool".to_string(),
                transport: MCPServerTransport::Stdio,
                ..Default::default()
            },
            status: None,
        };

        let labels = server.labels();
        assert_eq!(labels.len(), 3);
        assert_eq!(
            labels.get("app"),
            Some(&"mcp-server-test-pool-test-server-12345678".to_string())
        );
        assert_eq!(
            labels.get("unmcp.dev/uid"),
            Some(&"12345678-1234-1234-1234-123456789012".to_string())
        );
        assert_eq!(labels.get("unmcp.dev/pool"), Some(&"test-pool".to_string()));
    }

    #[test]
    fn test_mcp_server_url() {
        let url = MCPServer::new(
            "test-server",
            MCPServerSpec {
                pool: "test-pool".to_string(),
                transport: MCPServerTransport::Sse { port: 8080 },
                ..Default::default()
            },
        )
        .url();
        assert_eq!(url, "/api/v1/servers/test-server");
    }

    #[test]
    fn test_mcp_server_url_sse() {
        let url = MCPServer::new(
            "test-server",
            MCPServerSpec {
                pool: "test-pool".to_string(),
                transport: MCPServerTransport::Sse { port: 8080 },
                ..Default::default()
            },
        )
        .url_sse();
        assert_eq!(url, "/api/v1/servers/test-server/sse");
    }

    #[test]
    fn test_mcp_server_url_messages() {
        let url = MCPServer::new(
            "test-server",
            MCPServerSpec {
                pool: "test-pool".to_string(),
                transport: MCPServerTransport::Sse { port: 8080 },
                ..Default::default()
            },
        )
        .url_messages();
        assert_eq!(url, "/api/v1/servers/test-server/messages");
    }

    #[test]
    fn test_mcp_server_response_from_server() {
        let response = MCPServer::new(
            "test-server",
            MCPServerSpec {
                pool: "test-pool".to_string(),
                transport: MCPServerTransport::Sse { port: 8080 },
                ..Default::default()
            },
        )
        .into_response();
        assert_eq!(response.id, "12345678-1234-1234-1234-123456789012");
        assert_eq!(response.name, "test-server");
        assert_eq!(response.pool, "test-pool");
        assert_eq!(response.namespace, "default");
        assert_eq!(response.transport_port, Some(8080));
        assert_eq!(response.transport_type, "sse");
        assert_eq!(response.url, "/api/v1/servers/test-server");
        assert_eq!(response.url_sse, "/api/v1/servers/test-server/sse");
        assert_eq!(
            response.url_messages,
            "/api/v1/servers/test-server/messages"
        );
        assert_eq!(response.pool_url, "/api/v1/pools/test-pool");
    }

    #[test]
    fn test_mcp_server_response_from_server_stdio_transport() {
        let response = MCPServer::new(
            "test-server",
            MCPServerSpec {
                pool: "test-pool".to_string(),
                transport: MCPServerTransport::Stdio,
                ..Default::default()
            },
        )
        .into_response();
        assert_eq!(response.transport_port, None);
        assert_eq!(response.transport_type, "stdio");
    }
}
