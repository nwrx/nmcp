use kube::CustomResourceExt;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::fs::File;
use tokio::io::{stdout, AsyncWriteExt};
use unmcp::{
    serialize, Controller, ControllerOptions, Error, MCPPool, MCPServer, Result, Server,
    ServerOptions,
};

/// Command-line options for unmcp
#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "unmcp",
    about = "Kubernetes operator for managing MCP servers",
    after_help = "For more information, visit https://github.com/shorwood/unmcp"
)]
pub enum Command {
    /// Run the Kubernetes operator for managing MCP servers
    #[structopt(name = "operator")]
    Operator(ControllerOptions),

    /// Run only the API server without the operator
    #[structopt(name = "server")]
    Server {
        #[structopt(flatten)]
        global: ControllerOptions,
        #[structopt(flatten)]
        options: ServerOptions,
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
    match Command::from_args() {
        // Start the operator.
        Command::Operator(options) => {
            let controller = Controller::new(&options).await?;
            controller.start_tracing();
            controller.start_server_operator().await?;
        }

        // Start the API server.
        Command::Server { global, options } => {
            let controller = Controller::new(&global).await?;
            controller.start_tracing();
            let server = Server::new(options, controller).await?;
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
