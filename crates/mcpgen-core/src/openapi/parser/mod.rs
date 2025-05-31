//! OpenAPI parser module for handling YAML/JSON specs

mod parameter;
mod property;
mod schema;

pub use parameter::*;
pub use property::*;
pub use schema::*;

use crate::openapi::schema::{SchemaValue, SourceFormat};
use std::path::Path;

/// Parse an OpenAPI specification from a file
pub async fn parse_spec(path: &Path) -> crate::Result<SchemaValue> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(crate::Error::Io)?;
    let format = if content.trim_start().starts_with('{') {
        SourceFormat::Json
    } else {
        SourceFormat::Yaml
    };

    match format {
        SourceFormat::Json => Ok(SchemaValue::from_json(&content)?),
        SourceFormat::Yaml => Ok(SchemaValue::from_yaml(&content)?),
    }
}
