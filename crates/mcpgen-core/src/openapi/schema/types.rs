//! Common OpenAPI schema types

use serde::{Deserialize, Serialize};
use super::SchemaValue;

/// OpenAPI schema type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    /// String type
    String,
    /// Number type (float/double)
    Number,
    /// Integer type
    Integer,
    /// Boolean type
    Boolean,
    /// Array type
    Array,
    /// Object type
    Object,
}

/// Common schema formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchemaFormat {
    /// Date format (e.g., 2025-05-29)
    Date,
    /// Date-time format (e.g., 2025-05-29T20:07:42-04:00)
    DateTime,
    /// Time format (e.g., 20:07:42)
    Time,
    /// Duration format (e.g., P1DT2H)
    Duration,
    /// Email format
    Email,
    /// Hostname format
    Hostname,
    /// IPv4 format
    Ipv4,
    /// IPv6 format
    Ipv6,
    /// URI format
    Uri,
    /// URI reference format
    UriRef,
    /// UUID format
    Uuid,
}

/// Common schema metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMetadata {
    /// Schema title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    
    /// Schema description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<SchemaValue>,
    
    /// Example value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<SchemaValue>,
    
    /// Whether this schema is deprecated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schema_type_serde() {
        let json = serde_json::to_string(&SchemaType::String).unwrap();
        assert_eq!(json, r#""string""#);
        
        let schema_type: SchemaType = serde_json::from_str(r#""integer""#).unwrap();
        assert_eq!(schema_type, SchemaType::Integer);
    }
    
    #[test]
    fn test_schema_format_serde() {
        let json = serde_json::to_string(&SchemaFormat::DateTime).unwrap();
        assert_eq!(json, r#""date-time""#);
        
        let format: SchemaFormat = serde_json::from_str(r#""uri-ref""#).unwrap();
        assert_eq!(format, SchemaFormat::UriRef);
    }
}