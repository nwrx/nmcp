// Define constants used in the operator
pub const DEFAULT_SSE_CHANNEL_CAPACITY: usize = 1024;

// Default buffer size for pod streams
pub const DEFAULT_POD_BUFFER_SIZE: usize = 1024 * 256; //  256 KiB

/// The container name for the Pod that runs the MCP server
pub const MCP_SERVER_CONTAINER_NAME: &str = "server";
