//! Schema value types and conversion traits

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// Format of a schema value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourceFormat {
    /// JSON format
    #[default]
    Json,
    /// YAML format
    Yaml,
}

/// A schema value that can be parsed from either YAML or JSON.
/// Internally stored as JSON since OpenAPI tools typically work with JSON Schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaValue {
    /// The underlying JSON value
    #[serde(flatten)]
    inner: serde_json::Value,

    /// The original format this value was parsed from
    #[serde(skip)]
    format: SourceFormat,
}

impl SchemaValue {
    /// Create a new SchemaValue from a JSON string
    pub fn from_json(json: &str) -> crate::Result<Self> {
        let inner = serde_json::from_str(json)?;
        Ok(Self {
            inner,
            format: SourceFormat::Json,
        })
    }

    /// Create a new SchemaValue from a YAML string
    pub fn from_yaml(yaml: &str) -> crate::Result<Self> {
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml)?;
        let inner = serde_json::to_value(&yaml_value)?;
        Ok(Self {
            inner,
            format: SourceFormat::Yaml,
        })
    }

    /// Get the underlying JSON value
    pub fn into_json(self) -> serde_json::Value {
        self.inner
    }

    /// Get the format this value was originally parsed from
    pub fn format(&self) -> SourceFormat {
        self.format
    }
}

impl From<serde_json::Value> for SchemaValue {
    fn from(value: serde_json::Value) -> Self {
        Self {
            inner: value,
            format: SourceFormat::Json,
        }
    }
}

impl TryFrom<serde_yaml::Value> for SchemaValue {
    type Error = crate::Error;

    fn try_from(value: serde_yaml::Value) -> Result<Self, Self::Error> {
        let inner = serde_json::to_value(&value)?;
        Ok(Self {
            inner,
            format: SourceFormat::Yaml,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_roundtrip() -> crate::Result<()> {
        let json = r#"{"key": "value"}"#;
        let schema = SchemaValue::from_json(json)?;
        assert_eq!(schema.format(), SourceFormat::Json);

        let value = schema.into_json();
        assert_eq!(value["key"], "value");
        Ok(())
    }

    #[test]
    fn test_yaml_conversion() -> crate::Result<()> {
        let yaml = "key: value";
        let schema = SchemaValue::from_yaml(yaml)?;
        assert_eq!(schema.format(), SourceFormat::Yaml);

        let value = schema.into_json();
        assert_eq!(value["key"], "value");
        Ok(())
    }
}
