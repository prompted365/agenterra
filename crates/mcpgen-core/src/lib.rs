//! Core library for generating MCP servers from OpenAPI specifications.

use std::path::Path;
use thiserror::Error;

pub mod generator;
pub mod openapi;
pub mod template;

/// Errors that can occur during MCP generation
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("OpenAPI error: {0}")]
    OpenApi(String),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Template engine error: {0}")]
    Tera(#[from] tera::Error),
}

/// Result type for MCP generation operations
pub type Result<T> = std::result::Result<T, Error>;

/// Configuration for MCP server generation
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    /// Path to the OpenAPI specification file
    pub openapi_spec: String,

    /// Output directory for generated code
    pub output_dir: String,

    /// Template to use for code generation
    #[serde(default = "default_template")]
    pub template: String,

    /// Whether to include all operations (default: false)
    #[serde(default)]
    pub include_all: bool,

    /// List of operations to include (if include_all is false)
    #[serde(default)]
    pub include_operations: Vec<String>,

    /// List of operations to exclude (if include_all is true)
    #[serde(default)]
    pub exclude_operations: Vec<String>,
}

fn default_template() -> String {
    "axum-basic".to_string()
}

impl Config {
    /// Create a new Config with default values
    pub fn new(openapi_spec: impl Into<String>, output_dir: impl Into<String>) -> Self {
        Self {
            openapi_spec: openapi_spec.into(),
            output_dir: output_dir.into(),
            template: default_template(),
            include_all: false,
            include_operations: Vec::new(),
            exclude_operations: Vec::new(),
        }
    }

    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = std::fs::read_to_string(path_ref)?;
        let config = if path_ref.extension().map_or(false, |ext| ext == "json") {
            serde_json::from_str(&content)?
        } else {
            serde_yaml::from_str(&content)?
        };
        Ok(config)
    }

    /// Save configuration to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = if path.as_ref().extension().map_or(false, |ext| ext == "json") {
            serde_json::to_string_pretty(self)?
        } else {
            serde_yaml::to_string(self)?
        };
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Generate MCP server code from a configuration
pub async fn generate(config: &Config) -> Result<()> {
    log::info!(
        "Generating MCP server from OpenAPI spec: {}",
        config.openapi_spec
    );
    log::debug!("Using template: {}", config.template);

    // 1. Load and parse OpenAPI spec
    let spec = openapi::parser::parse_spec(Path::new(&config.openapi_spec)).await?;
    // .as_json() removed; use spec directly or convert as needed.

    // 2. Set up template manager
    let template_dir = Path::new(&config.output_dir).join("templates");
    let template_manager = template::TemplateManager::new(&template_dir)?;

    // 3. Extract endpoints and generate handlers
    let mut endpoints = Vec::new();

    if let Some(paths) = spec.as_json().get("paths") {
        if let Some(paths_obj) = paths.as_object() {
            for (path, methods) in paths_obj {
                if let Some(methods_obj) = methods.as_object() {
                    for (method, details) in methods_obj {
                        // Skip if not included or explicitly excluded
                        let operation_id = format!("{} {}", method.to_uppercase(), path);
                        if !config.include_all && !config.include_operations.contains(&operation_id)
                            || config.exclude_operations.contains(&operation_id)
                        {
                            continue;
                        }

                        // Extract endpoint info
                        let endpoint = path
                            .trim_start_matches('/')
                            .replace('/', "_")
                            .replace('{', "")
                            .replace('}', "");

                        let mut endpoint_info = std::collections::HashMap::new();
                        endpoint_info.insert("endpoint".to_string(), endpoint.clone());
                        endpoint_info.insert("method".to_string(), method.to_uppercase());
                        endpoint_info.insert("path".to_string(), path.to_string());
                        endpoint_info
                            .insert("fn_name".to_string(), format!("{}_handler", endpoint));

                        if let Some(details_obj) = details.as_object() {
                            if let Some(summary) = details_obj.get("summary") {
                                endpoint_info.insert(
                                    "summary".to_string(),
                                    summary.as_str().unwrap_or("").to_string(),
                                );
                            }
                            if let Some(description) = details_obj.get("description") {
                                endpoint_info.insert(
                                    "description".to_string(),
                                    description.as_str().unwrap_or("").to_string(),
                                );
                            }
                            if let Some(tags) = details_obj.get("tags") {
                                if let Some(tags_arr) = tags.as_array() {
                                    if let Some(first_tag) = tags_arr.first() {
                                        endpoint_info.insert(
                                            "tag".to_string(),
                                            first_tag.as_str().unwrap_or("").to_string(),
                                        );
                                    }
                                }
                            }
                        }

                        endpoints.push(endpoint_info);
                    }
                }
            }
        }
    }

    // 4. Generate handlers module
    let handlers_dir = Path::new(&config.output_dir).join("src/handlers");
    tokio::fs::create_dir_all(&handlers_dir).await?;

    template_manager
        .generate_handlers_mod(endpoints.clone(), handlers_dir.join("mod.rs"))
        .await?;

    // 5. Generate individual handler files
    for endpoint_info in endpoints {
        template_manager
            .generate_handler(
                "handler.rs",
                &endpoint_info,
                handlers_dir.join(format!("{}.rs", endpoint_info["endpoint"])),
            )
            .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::NamedTempFile;

    #[test]
    fn test_config_roundtrip() -> Result<()> {
        let mut config = Config::new("api.yaml", "output");
        config.include_all = true;
        config.exclude_operations = vec!["unwanted.endpoint".to_string()];

        let file = NamedTempFile::new()?;
        let path = file.path().to_path_buf();

        // Test YAML
        config.save(&path)?;
        let loaded = Config::from_file(&path)?;
        assert_eq!(config.openapi_spec, loaded.openapi_spec);
        assert_eq!(config.include_all, loaded.include_all);

        // Test JSON
        let json_path = path.with_extension("json");
        config.save(&json_path)?;
        let loaded_json = Config::from_file(&json_path)?;
        assert_eq!(config.openapi_spec, loaded_json.openapi_spec);

        Ok(())
    }
}
