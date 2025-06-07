use crate::{ControllerOptions, GatewayOptions, ManagerOptions, TracingOptions};
use clap::{ColorChoice, Parser};
use std::path::PathBuf;

/// Command-line options for nmcp
#[derive(Debug, Clone, Parser)]
#[command(
    name = "nmcp",
    about,
    version,
    color = ColorChoice::Always,
    after_help = "For more information, visit https://github.com/nwrx/nmcp",
    arg_required_else_help = true,
)]
pub struct Cli {
    /// Command to execute
    #[command(subcommand)]
    pub command: Command,

    /// Tracing configuration options
    #[command(flatten)]
    pub tracing_options: TracingOptions,
}

/// Commands supported by nmcp
#[derive(Debug, Clone, Parser)]
pub enum Command {
    /// Run the Kubernetes operator for managing MCP servers and pools
    #[command(name = "operator")]
    Operator {
        #[command(flatten)]
        controller_options: ControllerOptions,
    },

    /// Run only the API server without the operator
    #[command(name = "gateway")]
    Gateway {
        #[command(flatten)]
        controller_options: ControllerOptions,

        #[command(flatten)]
        gateway_options: GatewayOptions,
    },

    /// Run the manager API server
    #[command(name = "manager")]
    Manager {
        #[command(flatten)]
        controller_options: ControllerOptions,

        #[command(flatten)]
        manager_options: ManagerOptions,
    },

    /// Export CRD or schema definitions
    #[command(name = "export")]
    Export {
        /// Type of resource to export: crd or schema
        #[arg(short, long, value_parser = ["crd", "schema"])]
        r#type: String,

        /// Resource to export: pool or server
        #[arg(short, long, value_parser = ["pool", "server"])]
        resource: String,

        /// Output format: json or yaml
        #[arg(short, long, default_value = "yaml", value_parser = ["json", "yaml"])]
        format: String,

        /// Output file (optional, defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}
