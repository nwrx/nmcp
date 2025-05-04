pub mod controller;
pub mod resources;
pub mod server;
pub mod utils;

pub use controller::{Controller, ControllerOptions};
pub use resources::{MCPPool, MCPPoolSpec, MCPPoolStatus};
pub use resources::{MCPServer, MCPServerSpec, MCPServerStatus};
pub use resources::{MCPServerTransport, MCPServerTransportType};
pub use server::{Server, ServerOptions, ServerState};
pub use utils::{get_kube_client, Error, Result};

#[cfg(test)]
mod test_utils;

#[cfg(test)]
pub use test_utils::TestContext;
