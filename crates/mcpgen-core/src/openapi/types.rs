//! Core types for OpenAPI specification processing

use super::schema::SchemaValue;
use serde::{Deserialize, Serialize};

/// Represents a property in an OpenAPI schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyInfo {
    /// Name of the property
    pub name: String,
    /// Rust type for this property
    pub rust_type: String,
    /// OpenAPI type (string, number, etc.)
    pub openapi_type: String,
    /// Optional format specifier (e.g., "date-time")
    pub format: Option<String>,
    /// Title from OpenAPI schema
    pub title: String,
    /// Description from OpenAPI schema
    pub description: String,
    /// Example value as a string
    pub example: String,
    /// Whether this property is required
    pub required: bool,
    /// Full OpenAPI schema for this property
    pub schema: SchemaValue,
    /// Optional Elasticsearch type mapping
    pub elastic_type: Option<String>,
}

/// Represents a parameter in an OpenAPI operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    /// Name of the parameter
    pub name: String,
    /// Rust type for this parameter
    pub rust_type: String,
    /// OpenAPI type (string, number, etc.)
    pub openapi_type: String,
    /// Optional format specifier
    pub format: Option<String>,
    /// Processed description (with markdown sanitized)
    pub description: String,
    /// Raw description from OpenAPI
    pub description_raw: String,
    /// Processed example value
    pub example: String,
    /// Raw example from OpenAPI
    pub example_raw: String,
    /// Whether this parameter is required
    pub required: bool,
    /// Full OpenAPI schema for this parameter
    pub schema: SchemaValue,
}

impl PropertyInfo {
    /// Create a new PropertyInfo
    pub fn new(
        name: impl Into<String>,
        openapi_type: impl Into<String>,
        rust_type: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            openapi_type: openapi_type.into(),
            rust_type: rust_type.into(),
            format: None,
            title: String::new(),
            description: String::new(),
            example: String::new(),
            required: false,
            schema: serde_json::Value::Null.into(),
            elastic_type: None,
        }
    }
}

impl ParameterInfo {
    /// Create a new ParameterInfo
    pub fn new(
        name: impl Into<String>,
        openapi_type: impl Into<String>,
        rust_type: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            openapi_type: openapi_type.into(),
            rust_type: rust_type.into(),
            format: None,
            description: String::new(),
            description_raw: String::new(),
            example: String::new(),
            example_raw: String::new(),
            required: false,
            schema: serde_json::Value::Null.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_info_new() {
        let prop = PropertyInfo::new("test", "string", "String");
        assert_eq!(prop.name, "test");
        assert_eq!(prop.openapi_type, "string");
        assert_eq!(prop.rust_type, "String");
        assert!(!prop.required);
    }

    #[test]
    fn test_parameter_info_new() {
        let param = ParameterInfo::new("test", "string", "String");
        assert_eq!(param.name, "test");
        assert_eq!(param.openapi_type, "string");
        assert_eq!(param.rust_type, "String");
        assert!(!param.required);
    }
}
