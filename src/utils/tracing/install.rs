use clap::{Parser, ValueEnum};
use tracing::{level_filters::LevelFilter, Level};
use tracing_subscriber::{
    filter,
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

use super::FormatterDetailed;

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum TracingFormat {
    /// JSON format for machine processing
    Json,

    /// Pretty format for human-readable logs
    Pretty,

    /// Detailed format with additional context and debugging. Will display more information about errors.
    Detailed,

    /// Compact format for minimal output
    Compact,
}

/// Tracing configuration options
#[derive(Debug, Clone, Parser)]
pub struct TracingOptions {
    /// Set the tracing level
    #[arg(long, global = true, default_value = "info", value_parser = ["off", "error", "warn", "info", "debug", "trace"])]
    pub log_level: String,

    /// Filter logs to only show entries from specific modules (comma-separated)
    #[arg(long, global = true)]
    pub log_filter: Option<String>,

    /// The format for logs
    #[arg(long, global = true, value_enum, default_value = "pretty")]
    pub log_format: Option<TracingFormat>,

    /// Show backtraces for errors
    #[arg(long, global = true)]
    pub show_backtrace: bool,
}

/// Install tracing with the provided configuration options
pub fn install_tracing(options: &TracingOptions) {
    // Convert log level string to LevelFilter
    let level = match options.log_level.to_lowercase().as_str() {
        "off" => LevelFilter::OFF,
        "error" => LevelFilter::ERROR,
        "warn" => LevelFilter::WARN,
        "info" => LevelFilter::INFO,
        "debug" => LevelFilter::DEBUG,
        "trace" => LevelFilter::TRACE,
        _ => LevelFilter::INFO, // Default to INFO
    };

    // Create filter based on options
    let filter = EnvFilter::from_default_env()
        .add_directive(level.into())
        .add_directive("nmcp=info".parse().unwrap());

    // Create the formatting layer based on the options
    let formatter = match options.log_format {
        Some(TracingFormat::Json) => fmt::layer()
            .json()
            .with_ansi(true)
            .with_file(true)
            .with_level(true)
            .with_target(true)
            .with_span_list(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_line_number(true)
            .boxed(),
        Some(TracingFormat::Pretty) => fmt::layer()
            .pretty()
            .with_ansi(true)
            .with_file(true)
            .with_level(true)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_line_number(true)
            .boxed(),
        Some(TracingFormat::Detailed) => fmt::layer()
            .event_format(FormatterDetailed)
            .with_ansi(true)
            .boxed(),
        _ => fmt::layer()
            .compact()
            .with_ansi(true)
            .with_file(true)
            .with_level(true)
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_line_number(true)
            .boxed(),
    };

    let targets = filter::Targets::new().with_target("nmcp", Level::TRACE);

    // Initialize the tracing subscriber with the configured layers
    tracing_subscriber::registry()
        .with(formatter)
        .with(targets)
        .with(filter)
        .init();
}
