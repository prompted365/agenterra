//! Core library for generating MCP servers from OpenAPI specifications.

use std::path::Path;
use thiserror::Error;

pub mod openapi;
pub mod generator;

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
    // TODO: Implement actual code generation
    // 1. Load and parse OpenAPI spec
    // 2. Apply filters based on config
    // 3. Generate code using the specified template
    // 4. Write output files
    
    log::info!("Generating MCP server from OpenAPI spec: {}", config.openapi_spec);
    log::debug!("Using template: {}", config.template);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_roundtrip() -> Result<()> {
        let mut config = Config::new("api.yaml", "output");
        config.include_all = true;
        config.exclude_operations = vec!["unwanted.endpoint".to_string()];
        
        let mut file = NamedTempFile::new()?;
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
