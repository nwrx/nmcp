use std::path::PathBuf;
use tokio::time::Duration;
use structopt::StructOpt;

use crate::utils::SerializeFormat;

/// Command line arguments for the UNMCP operator
#[derive(StructOpt, Debug)]
#[structopt(name = "unmcp", about = "Unified MCP operator for Kubernetes")]
pub struct ProgramArgs {
    #[structopt(subcommand)]
    pub cmd: Command,
}

/// Subcommands for the UNMCP operator
#[derive(StructOpt, Debug)]
pub enum Command {
    /// Start the MCP operator to manage the MCP resources
    #[structopt(name = "operator")]
    Operator(OperatorCommand),
    
    /// Get CRD schema information
    #[structopt(name = "schema")]
    Schema(SchemaCommand),
    
    /// Get CRD definition
    #[structopt(name = "crd")]
    Crd(CrdCommand),
}

/// Operator subcommands
#[derive(StructOpt, Debug)]
pub enum OperatorCommand {
    /// Start the operator
    #[structopt(name = "start")]
    Start(OperatorStartArgs),
}

/// Arguments for the operator start command
#[derive(StructOpt, Debug)]
pub struct OperatorStartArgs {
    /// Path to kubeconfig file
    #[structopt(short, long, parse(from_os_str))]
    pub kubeconfig: Option<PathBuf>,

    /// Namespace to watch (watches all namespaces if not specified)
    #[structopt(short, long)]
    pub namespace: Option<String>,

    /// Default reconciliation interval in seconds
    #[structopt(long, default_value = "60")]
    pub reconciliation_interval: u64,

    /// Server controller reconciliation interval in seconds
    #[structopt(long)]
    pub reconciliation_interval_server: Option<u64>,

    /// Pool controller reconciliation interval in seconds
    #[structopt(long)]
    pub reconciliation_interval_pool: Option<u64>,

    /// Default pool name for servers
    #[structopt(long)]
    pub default_pool: Option<String>,
}

/// Schema subcommands
#[derive(StructOpt, Debug)]
pub enum SchemaCommand {
    /// Get Pool schema
    #[structopt(name = "pool")]
    Pool(FormatOption),
    
    /// Get Server schema
    #[structopt(name = "server")]
    Server(FormatOption),
}

/// CRD subcommands
#[derive(StructOpt, Debug)]
pub enum CrdCommand {
    /// Get Pool CRD
    #[structopt(name = "pool")]
    Pool(FormatOption),
    
    /// Get Server CRD
    #[structopt(name = "server")]
    Server(FormatOption),
}

/// Format options for output
#[derive(StructOpt, Debug)]
pub struct FormatOption {
    /// Output format (yaml or json)
    #[structopt(long, default_value = "yaml", possible_values = &["yaml", "json"])]
    pub format: SerializeFormat,
}

/// Program struct that encapsulates the configuration and runtime of the UNMCP operator
pub struct Program {
    pub kubeconfig: Option<PathBuf>,
    pub reconciliation_interval: Duration,
    pub reconciliation_interval_server: Duration,
    pub reconciliation_interval_pool: Duration,
    pub namespace: Option<String>,
    pub default_pool: Option<String>,
}

impl Program {
    /// Create a new Program instance with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new Program instance from command line arguments
    pub fn from_args(args: ProgramArgs) -> Self {
        match &args.cmd {
            Command::Operator(OperatorCommand::Start(start_args)) => {
                let default_interval = Duration::from_secs(start_args.reconciliation_interval);
                Self {
                    kubeconfig: start_args.kubeconfig.clone(),
                    reconciliation_interval: default_interval,
                    reconciliation_interval_server: start_args.reconciliation_interval_server
                        .map(Duration::from_secs)
                        .unwrap_or(default_interval),
                    reconciliation_interval_pool: start_args.reconciliation_interval_pool
                        .map(Duration::from_secs)
                        .unwrap_or(default_interval),
                    namespace: start_args.namespace.clone(),
                    default_pool: start_args.default_pool.clone(),
                }
            },
            _ => Self::default(),
        }
    }
}

impl Default for Program {
    fn default() -> Self {
        Self {
            kubeconfig: None,
            reconciliation_interval: Duration::from_secs(60),
            reconciliation_interval_server: Duration::from_secs(60),
            reconciliation_interval_pool: Duration::from_secs(120),
            namespace: None,
            default_pool: None,
        }
    }
}