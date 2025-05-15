use kube::CustomResourceExt;
use nmcp::{
    serialize, Controller, ControllerOptions, Error, Gateway, GatewayOptions, MCPPool, MCPServer,
    Result,
};
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::fs::File;
use tokio::io::{stdout, AsyncWriteExt};
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Command-line options for nmcp
#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "nmcp",
    about = "Kubernetes operator for managing MCP servers",
    after_help = "For more information, visit https://github.com/shorwood/nmcp"
)]
pub enum Command {
    /// Run the Kubernetes operator for managing MCP servers
    #[structopt(name = "operator")]
    Operator(ControllerOptions),

    /// Run only the API server without the operator
    #[structopt(name = "gateway")]
    Gateway {
        #[structopt(flatten)]
        controller_options: ControllerOptions,

        #[structopt(flatten)]
        gateway_options: GatewayOptions,
    },

    /// Export CRD or schema definitions
    #[structopt(name = "export")]
    Export {
        /// Type of resource to export: crd or schema
        #[structopt(short, long, possible_values = &["crd", "schema"])]
        r#type: String,

        /// Resource to export: pool or server
        #[structopt(short, long, possible_values = &["pool", "server"])]
        resource: String,

        /// Output format: json or yaml
        #[structopt(short, long, default_value = "yaml", possible_values = &["json", "yaml"])]
        format: String,

        /// Output file (optional, defaults to stdout)
        #[structopt(short, long)]
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

    match Command::from_args() {
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
