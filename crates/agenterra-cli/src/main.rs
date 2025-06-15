//! agenterra CLI entrypoint
//! Parses command-line arguments and dispatches to the core generator.

// Internal imports (std, crate)
use reqwest::Url;
use std::path::{Path, PathBuf};

use dialoguer::{theme::ColorfulTheme, Input, Select};
use notify::{recommended_watcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

// External imports (alphabetized)
use agenterra_core::{TemplateKind, TemplateManager, TemplateOptions};
use anyhow::Context;
use clap::Parser;
use tokio::fs;

#[derive(Parser)]
#[command(name = "agenterra")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    /// Scaffold a new MCP server from an OpenAPI spec
    Scaffold {
        /// Project name
        #[arg(long, default_value = "agenterra_mcp_server")]
        project_name: String,
        /// Path or URL to OpenAPI schema (YAML or JSON)
        ///
        /// Can be a local file path or an HTTP/HTTPS URL
        /// Example: --schema-path path/to/schema.yaml
        /// Example: --schema-path https://example.com/openapi.json
        #[arg(long)]
        schema_path: String,
        /// Template to use for code generation (e.g., rust_axum, python_fastapi)
        #[arg(long, default_value = "rust_axum")]
        template_kind: String,
        /// Custom template directory (only used with --template-kind=custom)
        #[arg(long)]
        template_dir: Option<PathBuf>,
        /// Output directory for generated code
        #[arg(long)]
        output_dir: Option<PathBuf>,
        /// Log file name without extension (default: mcp-server)
        #[arg(long)]
        log_file: Option<String>,
        /// Server port (default: 3000)
        #[arg(long)]
        port: Option<u16>,
        /// Base URL of the OpenAPI specification (Optional)
        #[arg(long)]
        base_url: Option<Url>,
        /// Watch schema file for changes and rebuild automatically
        #[arg(long)]
        watch: bool,
    },
    /// Interactive scaffolding flow
    Init,
    /// List available template kinds
    ListTemplates,
}

/// Arguments needed to scaffold a project
#[derive(Clone, Debug)]
struct ScaffoldArgs {
    project_name: String,
    schema_path: String,
    template_kind: String,
    template_dir: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    log_file: Option<String>,
    port: Option<u16>,
    base_url: Option<Url>,
    watch: bool,
}

/// Execute the scaffold flow with the provided arguments
async fn run_scaffold(args: &ScaffoldArgs) -> anyhow::Result<()> {
    // Parse template
    let template_kind_enum: TemplateKind = args
        .template_kind
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid template '{}' : {e}", args.template_kind))?;

    // Resolve output directory - use project_name if not specified
    let output_path = args
        .output_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(&args.project_name));

    // Debug log template and paths
    println!(
        "Scaffolding with template: {}, template_dir: {:?}, output_dir: {:?}",
        template_kind_enum.as_str(),
        args.template_dir,
        output_path
    );

    // Initialize the template manager using the resolved template directory
    let template_manager = TemplateManager::new(template_kind_enum, args.template_dir.clone())
        .await
        .context("Failed to initialize template manager")?;

    // Create output directory if it doesn't exist
    if !output_path.exists() {
        println!("Creating output directory: {}", output_path.display());
        fs::create_dir_all(&output_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create output directory: {}", e))?;
    }

    // List available templates for debugging
    println!("Available templates:");
    for template in template_manager.list_templates() {
        println!("Source: {} -> Destination: {}", template.0, template.1);
    }

    println!(
        "Using templates from: {}",
        template_manager.template_dir().display()
    );

    // Create directories for all template file destinations
    for file in &template_manager.manifest().files {
        if let Some(parent) = Path::new(&file.destination).parent() {
            let dir = output_path.join(parent);
            if !dir.exists() {
                println!("Creating directory: {}", dir.display());
                fs::create_dir_all(&dir).await.map_err(|e| {
                    anyhow::anyhow!("Failed to create directory {}: {}", dir.display(), e)
                })?;
            }
        }
    }

    // Load the OpenAPI schema from either a file or URL
    let schema_path = &args.schema_path;
    println!("Loading OpenAPI schema from: {}", schema_path);

    // Check if the schema_path is a URL or a file path
    let schema_obj = if schema_path.starts_with("http://") || schema_path.starts_with("https://") {
        // It's a URL, use from_url
        let response = reqwest::get(schema_path.as_str()).await.map_err(|e| {
            anyhow::anyhow!("Failed to fetch OpenAPI schema from {}: {}", schema_path, e)
        })?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch OpenAPI schema from {}: HTTP {}",
                schema_path,
                response.status()
            ));
        }

        let content = response
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read response from {}: {}", schema_path, e))?;

        // Parse the content as OpenAPI schema
        let temp_dir = tempfile::tempdir()?;
        let temp_file = temp_dir.path().join("openapi_schema.json");
        tokio::fs::write(&temp_file, &content).await?;

        agenterra_core::openapi::OpenApiContext::from_file(&temp_file)
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed to parse OpenAPI schema from {}: {}\nSee docs/CONFIGURATION.md#troubleshooting", schema_path, e)
            })?
    } else {
        // It's a file path
        agenterra_core::openapi::OpenApiContext::from_file(schema_path)
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to load OpenAPI schema: {}\nSee docs/CONFIGURATION.md#troubleshooting",
                    e
                )
            })?
    };

    // Create config with template
    let config = agenterra_core::Config {
        project_name: args.project_name.clone(),
        openapi_schema_path: schema_path.to_string(),
        output_dir: output_path.to_string_lossy().to_string(),
        template_kind: args.template_kind.clone(),
        template_dir: args
            .template_dir
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        include_all: true,
        include_operations: Vec::new(),
        exclude_operations: Vec::new(),
        base_url: args.base_url.clone(),
    };

    // Create template options
    let template_opts = TemplateOptions {
        server_port: args.port,
        log_file: args.log_file.clone(),
        ..Default::default()
    };

    // Generate the server using the template manager
    template_manager
        .generate(&schema_obj, &config, Some(template_opts))
        .await?;

    println!(
        "✅ Successfully generated server in: {}",
        output_path.display()
    );
    Ok(())
}

async fn watch_and_scaffold(args: ScaffoldArgs) -> anyhow::Result<()> {
    if args.schema_path.starts_with("http://") || args.schema_path.starts_with("https://") {
        println!("--watch is only supported for local schema files");
        return run_scaffold(&args).await;
    }

    let (tx, mut rx) = mpsc::channel(1);
    let schema = args.schema_path.clone();
    let mut watcher = recommended_watcher(move |res| {
        let _ = tx.blocking_send(res);
    })?;
    watcher.watch(Path::new(&schema), RecursiveMode::NonRecursive)?;

    run_scaffold(&args).await?;
    println!("Watching {} for changes...", schema);

    while let Some(res) = rx.recv().await {
        match res {
            Ok(_event) => {
                println!("Change detected. Regenerating...");
                if let Err(e) = run_scaffold(&args).await {
                    eprintln!("Generation failed: {e:#}");
                }
                let output_dir = args
                    .output_dir
                    .clone()
                    .unwrap_or_else(|| PathBuf::from(&args.project_name));
                let build = tokio::process::Command::new("cargo")
                    .arg("check")
                    .current_dir(&output_dir)
                    .output()
                    .await?;
                if !build.status.success() {
                    eprintln!("Build errors:\n{}", String::from_utf8_lossy(&build.stderr));
                } else {
                    println!("Build succeeded.");
                }
            }
            Err(e) => eprintln!("Watch error: {e:?}"),
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match &cli.command {
        Commands::Scaffold {
            project_name,
            schema_path,
            template_kind,
            template_dir,
            output_dir,
            log_file,
            port,
            base_url,
            watch,
        } => {
            let args = ScaffoldArgs {
                project_name: project_name.clone(),
                schema_path: schema_path.clone(),
                template_kind: template_kind.clone(),
                template_dir: template_dir.clone(),
                output_dir: output_dir.clone(),
                log_file: log_file.clone(),
                port: *port,
                base_url: base_url.clone(),
                watch: *watch,
            };
            if args.watch {
                watch_and_scaffold(args).await?;
            } else {
                run_scaffold(&args).await?;
            }
        }
        Commands::Init => {
            let theme = ColorfulTheme::default();
            let project_name: String = Input::with_theme(&theme)
                .with_prompt("Project name")
                .default("agenterra_mcp_server".into())
                .interact_text()?;

            let schema_path: String = Input::with_theme(&theme)
                .with_prompt("Path or URL to OpenAPI schema")
                .default("tests/fixtures/openapi/petstore.openapi.v3.json".into())
                .interact_text()?;
            if !schema_path.starts_with("http://")
                && !schema_path.starts_with("https://")
                && tokio::fs::metadata(&schema_path).await.is_err()
            {
                return Err(anyhow::anyhow!(
                    "Schema path does not exist: {}",
                    schema_path
                ));
            }

            let templates: Vec<String> = TemplateKind::all()
                .map(|k| k.as_str().to_string())
                .collect();
            let selection = Select::with_theme(&theme)
                .with_prompt("Template kind")
                .items(&templates)
                .default(0)
                .interact()?;
            let template_kind = templates[selection].clone();

            let default_output = project_name.clone();
            let output_dir_str: String = Input::with_theme(&theme)
                .with_prompt("Output directory")
                .default(default_output)
                .interact_text()?;

            let args = ScaffoldArgs {
                project_name,
                schema_path,
                template_kind,
                template_dir: None,
                output_dir: Some(PathBuf::from(output_dir_str)),
                log_file: None,
                port: None,
                base_url: None,
                watch: false,
            };
            if args.watch {
                watch_and_scaffold(args).await?;
            } else {
                run_scaffold(&args).await?;
            }
        }
        Commands::ListTemplates => {
            println!("Available template kinds:");
            for kind in TemplateKind::all() {
                println!("- {}", kind.as_str());
            }
        }
    }
    Ok(())
}
