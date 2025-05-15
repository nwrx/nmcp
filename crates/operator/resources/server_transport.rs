use schemars::gen::SchemaGenerator;
use schemars::schema::{InstanceType, Metadata, Schema, SchemaObject};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt::{self, Display};

/// MCPServer transport configuration
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum MCPServerTransport {
    /// Standard Input/Output (STDIO). This transport type is used for
    /// communication over standard input and output.
    #[serde(rename = "stdio")]
    #[default]
    Stdio,

    /// Server-Sent Events (HTTP). This transport type is used for
    /// communication over HTTP.
    #[serde(rename = "sse")]
    Sse {
        /// When the transport type is `sse`, this field specifies the port
        /// on which the server will listen for incoming connections. This field
        /// is required for the `sse` transport type.
        port: u16,
    },
}

// Custom implementation of JsonSchema to avoid conflicts with the "type" tag field
impl JsonSchema for MCPServerTransport {
    fn schema_name() -> String {
        "MCPServerTransport".to_string()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        // Create a schema object for the enum - simplified approach
        let mut schema_obj = SchemaObject {
            metadata: Some(Box::new(Metadata {
                title: Some("MCPServer Transport Configuration".to_string()),
                description: Some("Configuration for the MCP server transport layer".to_string()),
                ..Default::default()
            })),
            instance_type: Some(schemars::schema::InstanceType::Object.into()),
            ..Default::default()
        };

        // Define the type field schema that accepts either "stdio" or "sse"
        let type_schema = SchemaObject {
            metadata: Some(Box::new(Metadata {
                description: Some("Transport type".to_string()),
                ..Default::default()
            })),
            instance_type: Some(InstanceType::String.into()),
            enum_values: Some(vec!["stdio".into(), "sse".into()]),
            ..Default::default()
        };

        // Define the port field schema (optional)
        let port_schema = SchemaObject {
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "Port number for SSE transport, required when type is 'sse'".to_string(),
                ),
                ..Default::default()
            })),
            instance_type: Some(InstanceType::Integer.into()),
            ..Default::default()
        };

        // Set the properties on the schema object
        schema_obj.object = Some(Box::new(schemars::schema::ObjectValidation {
            properties: {
                let mut properties = schemars::Map::new();
                properties.insert("type".to_string(), Schema::Object(type_schema));
                properties.insert("port".to_string(), Schema::Object(port_schema));
                properties
            },
            required: {
                let mut required = BTreeSet::new();
                required.insert("type".to_string());
                required
            },
            ..Default::default()
        }));

        Schema::Object(schema_obj)
    }
}

impl MCPServerTransport {
    /// Get the type of transport as a string
    pub fn transport_type(&self) -> String {
        match self {
            MCPServerTransport::Sse { .. } => "sse".to_string(),
            MCPServerTransport::Stdio => "stdio".to_string(),
        }
    }

    /// Get the port for SSE transport, if applicable
    pub fn port(&self) -> Option<u16> {
        match self {
            MCPServerTransport::Sse { port } => Some(*port),
            MCPServerTransport::Stdio => None,
        }
    }
}

impl Display for MCPServerTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MCPServerTransport::Sse { port } => write!(f, "sse-{port}"),
            MCPServerTransport::Stdio => write!(f, "stdio"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that the default transport type is set to `Stdio`.
    #[test]
    fn test_transport_defaults() {
        let transport = MCPServerTransport::default();
        assert!(matches!(transport, MCPServerTransport::Stdio));
    }

    /// Tests the string representation of transport types.
    #[test]
    fn test_transport_display() {
        assert_eq!(
            MCPServerTransport::Sse { port: 1234 }.to_string(),
            "sse-1234"
        );
        assert_eq!(MCPServerTransport::Stdio.to_string(), "stdio");
    }

    /// Tests the transport type string representation.
    #[test]
    fn test_transport_type() {
        assert_eq!(
            MCPServerTransport::Sse { port: 1234 }.transport_type(),
            "sse"
        );
        assert_eq!(MCPServerTransport::Stdio.transport_type(), "stdio");
    }

    /// Tests the port retrieval for transport types.
    #[test]
    fn test_transport_port() {
        assert_eq!(MCPServerTransport::Sse { port: 1234 }.port(), Some(1234));
        assert_eq!(MCPServerTransport::Stdio.port(), None);
    }

    /// Tests SSE transport configuration and behavior.
    #[test]
    fn test_transport_sse() {
        let transport = MCPServerTransport::Sse { port: 1234 };
        assert_eq!(transport.to_string(), "sse-1234");
        assert_eq!(transport.port(), Some(1234));
    }

    /// Tests stdio transport configuration and behavior.
    #[test]
    fn test_transport_stdio() {
        let transport = MCPServerTransport::Stdio;
        assert_eq!(transport.to_string(), "stdio");
        assert_eq!(transport.port(), None);
    }

    /// Tests deserialization of transport types from JSON.
    #[test]
    fn test_transport_deserialization() {
        let sse_json = r#"{"type": "sse", "port": 1234}"#;
        let sse: MCPServerTransport = serde_json::from_str(sse_json).unwrap();
        assert_eq!(sse, MCPServerTransport::Sse { port: 1234 });
        let stdio_json = r#"{"type": "stdio"}"#;
        let stdio: MCPServerTransport = serde_json::from_str(stdio_json).unwrap();
        assert_eq!(stdio, MCPServerTransport::Stdio);
    }

    /// Tests serialization of transport types to JSON.
    #[test]
    fn test_transport_serialization() {
        let sse = MCPServerTransport::Sse { port: 1234 };
        let sse_json = serde_json::to_string(&sse).unwrap();
        assert_eq!(sse_json, r#"{"type":"sse","port":1234}"#);

        let stdio = MCPServerTransport::Stdio;
        let stdio_json = serde_json::to_string(&stdio).unwrap();
        assert_eq!(stdio_json, r#"{"type":"stdio"}"#);
    }

    /// Tests the JSON schema generation for transport types.
    #[test]
    fn test_transport_json_schema() {
        let schema = schemars::schema_for!(MCPServerTransport);
        let json_schema = serde_json::to_string_pretty(&schema);
        assert!(json_schema.is_ok());
    }
}
