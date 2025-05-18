use clap::Parser;
use kube::CustomResourceExt;
use nmcp::{serialize, Controller, ControllerOptions, Error, Gateway, GatewayOptions, Result};
use nmcp::{MCPPool, MCPServer};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{stdout, AsyncWriteExt};
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Command-line options for nmcp
#[derive(Debug, Clone, Parser)]
#[command(
    name = "nmcp",
    version,
    about = "Kubernetes operator for managing MCP servers",
    after_help = "For more information, visit https://github.com/shorwood/nmcp"
)]
pub enum Command {
    /// Run the Kubernetes operator for managing MCP servers
    #[command(name = "operator")]
    Operator(ControllerOptions),

    /// Run only the API server without the operator
    #[command(name = "gateway")]
    Gateway {
        #[command(flatten)]
        controller_options: ControllerOptions,

        #[command(flatten)]
        gateway_options: GatewayOptions,
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

/// Main entry point for the operator
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_file(true)
                .with_level(true)
                .with_target(false)
                .with_line_number(true),
        )
        .with(filter_fn(|metadata| {
            metadata.module_path().unwrap_or_default().contains("nmcp")
        }))
        .init();

    match Command::parse() {
        // Start the operator.
        Command::Operator(options) => {
            let controller = Controller::new(&options).await?;
            controller.start_server_operator().await?;
        }

        // Start the API server.
        Command::Gateway {
            controller_options,
            gateway_options,
        } => {
            let controller = Controller::new(&controller_options).await?;
            let server = Gateway::new(gateway_options, controller).await?;
            let _ = server.start().await;
        }

        // Export CRD or schema
        Command::Export {
            r#type,
            resource,
            format,
            output,
        } => {
            let serialized = match (r#type.as_str(), resource.as_str()) {
                ("crd", "pool") => serialize(&MCPPool::crd(), &format)?,
                ("crd", "server") => serialize(&MCPServer::crd(), &format)?,
                ("schema", "pool") => serialize(&schemars::schema_for!(MCPPool), &format)?,
                ("schema", "server") => serialize(&schemars::schema_for!(MCPServer), &format)?,
                _ => {
                    return Err(Error::Internal(format!(
                        "Invalid type/resource combination: {type}/{resource}"
                    )));
                }
            };

            // --- Write to file or stdout
            match output {
                Some(path) => {
                    let mut file = File::create(path).await.map_err(Error::WriteError)?;
                    file.write_all(serialized.as_bytes())
                        .await
                        .map_err(Error::WriteError)?;
                }
                None => {
                    stdout()
                        .write_all(serialized.as_bytes())
                        .await
                        .map_err(Error::WriteError)?;
                }
            }
        }
    }
    Ok(())
}
