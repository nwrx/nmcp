use clap::Parser;
use kube::CustomResourceExt;
use nmcp::{install_tracing, serialize};
use nmcp::{Cli, Command, Controller, ErrorInner, Gateway, Result, ResultExt};
use nmcp::{MCPPool, MCPServer};
use tokio::fs::File;
use tokio::io::{stdout, AsyncWriteExt};

/// Main entry point for the operator
#[tokio::main]
async fn main() -> Result<()> {
    let arguments = Cli::parse();
    install_tracing(&arguments.tracing_options);

    // --- Start the operator or API server based on the command
    let result = match arguments.command {
        // Start the operator.
        Command::Operator { controller_options } => {
            let controller = Controller::new(&controller_options).await?;
            controller.start_server_operator().await
        }
        // Start the gateway API server.
        Command::Gateway {
            controller_options,
            gateway_options,
        } => {
            let controller = Controller::new(&controller_options).await?;
            let server = Gateway::new(gateway_options, controller).await?;
            server.start().await
        }
        // Start the manager API server.
        Command::Manager {
            controller_options,
            manager_options,
        } => {
            let controller = Controller::new(&controller_options).await?;
            let server = nmcp::manager::Manager::new(manager_options, controller).await?;
            server.start().await
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
                    return Result::Err(
                        ErrorInner::Generic(format!(
                            "Invalid type/resource combination: {type}/{resource}"
                        ))
                        .into(),
                    )
                }
            };

            // --- Write to file or stdout
            match output {
                None => stdout()
                    .write_all(serialized.as_bytes())
                    .await
                    .with_message("Could not write to stdout"),

                // If an output file is specified, write to that file.
                Some(path) => {
                    let mut file = File::open(path)
                        .await
                        .with_message("Could not open output file")?;

                    file.write_all(serialized.as_bytes())
                        .await
                        .with_message("Could not write to output file")
                }
            }
        }
    };

    if let Err(error) = result {
        error.trace();
    };

    Ok(())
}
