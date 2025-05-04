use structopt::StructOpt;
use unmcp_operator::{Controller, ControllerOptions, Result, Server, ServerOptions};

/// Command-line options for unmcp-operator
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
}

/// Main entry point for the operator
#[tokio::main]
async fn main() -> Result<()> {
    match Command::from_args() {
        // Start the operator.
        Command::Operator(options) => {
            let controller = Controller::new(&options).await?;
            controller.start_server_operator().await?;
        }

        // Start the API server.
        Command::Server { global, options } => {
            let controller = Controller::new(&global).await?;
            let server = Server::new(options, controller).await?;
            let _ = server.start().await;
        }
    }
    Ok(())
}
