//! mcpgen CLI entrypoint
//! Parses command-line arguments and dispatches to the core generator.

use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber;

/// CLI for mcpgen: Generate MCP servers from OpenAPI specs
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to OpenAPI spec file (YAML or JSON)
    #[arg(short, long, default_value = "api.yaml")]
    pub spec: PathBuf,

    /// Output target (e.g. rust-axum)
    #[arg(short, long, default_value = "rust-axum")]
    pub target: String,

    /// Comma-separated list of policy plugins
    #[arg(long)]
    pub policy_plugins: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse CLI arguments
    let cli = Cli::parse();
    tracing::info!(?cli, "Parsed CLI arguments");
    println!("Parsed CLI args: {cli:#?}");

    // TODO: Wire to core generator logic
    Ok(())
}
