use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// MCPServer transport configuration
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct MCPServerTransport {

    /// The transport type to use for the server. This will be used to determine how the
    /// server communicates with other components in the system, such as the database or
    /// other servers. Can either be "sse" (HTTP) or "stdio" (STDIN/STDOUT).
    /// This field is required.
    #[serde(rename = "type")]
    pub type_: MCPServerTransportType,

    /// Port to use for the transport. This will be used to determine which port the server
    /// listens on for incoming connections. This field is required for the "sse" transport
    /// type, and ignored for the "stdio" transport type.
    #[serde(default)]
    pub port: Option<i32>,
}

impl MCPServerTransport {
    /// Check if the transport type is "sse"
    pub fn is_sse(&self) -> bool {
        matches!(self.type_, MCPServerTransportType::Sse)
    }

    /// Check if the transport type is "stdio"
    pub fn is_stdio(&self) -> bool {
        matches!(self.type_, MCPServerTransportType::Stdio)
    }
}

impl Default for MCPServerTransport {
    fn default() -> Self {
        MCPServerTransport {
            type_: MCPServerTransportType::Stdio,
            port: None,
        }
    }
}

impl Display for MCPServerTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.type_ {
            MCPServerTransportType::Sse => write!(f, "sse-{}", self.port.unwrap_or(0)),
            MCPServerTransportType::Stdio => write!(f, "stdio"),
        }
    }
}

/// MCPServer transport type
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MCPServerTransportType {
    /// Server-Sent Events (HTTP). This transport type is used for
    /// communication over HTTP.
    Sse,

    /// Standard Input/Output (STDIO). This transport type is used for
    /// communication over standard input and output.
    Stdio,
}
