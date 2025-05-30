//! Property parsing functionality for OpenAPI specs

use crate::openapi::types::PropertyInfo;
use serde_yaml::{Mapping, Value as YamlValue};

/// Extract property information from a YAML value
pub fn extract_property_info(name: &str, prop: &YamlValue) -> PropertyInfo {
    let default_mapping = Mapping::new();
    let map = prop.as_mapping().unwrap_or(&default_mapping);

    // Get basic fields with defaults
    let openapi_type = get_str_field(map, "type");
    let format = map.get("format").and_then(|v| v.as_str()).map(String::from);
    let title = get_str_field(map, "title");
    let description = get_str_field(map, "description");
    let example = map
        .get("example")
        .map(|v| v.as_str().unwrap_or("").to_string())
        .unwrap_or_default();
    let required = get_bool_field(map, "required");

    // Convert schema to JSON for storage
    let schema = serde_json::to_value(map).unwrap_or(serde_json::Value::Null);

    // Get elastic type if present
    let elastic_type = map
        .get("x-elastic-type")
        .and_then(|v| v.as_str())
        .map(String::from);

    // Map OpenAPI type to Rust type
    let rust_type = openapi_type_to_rust_type(&openapi_type, format.as_deref());

    PropertyInfo {
        name: name.to_string(),
        rust_type: rust_type.to_string(),
        openapi_type,
        format,
        title,
        description,
        example,
        required,
        schema: schema.into(),
        elastic_type,
    }
}

/// Extract properties from a YAML mapping
pub fn extract_properties(mapping: &serde_yaml::Mapping) -> Vec<PropertyInfo> {
    mapping
        .iter()
        .map(|(k, v)| {
            let name = k.as_str().unwrap_or_default();
            extract_property_info(name, v)
        })
        .collect()
}

/// Extract row properties from a YAML document
pub fn extract_row_properties(doc: &serde_yaml::Value) -> Vec<PropertyInfo> {
    // First try to get properties.data.properties
    if let Some(props) = doc
        .get("properties")
        .and_then(|p| p.get("data"))
        .and_then(|d| d.get("properties"))
        .and_then(|p| p.as_mapping())
    {
        return extract_properties(props);
    }

    // Fall back to top-level properties
    doc.get("properties")
        .and_then(|p| p.as_mapping())
        .map(extract_properties)
        .unwrap_or_default()
}

// Helper functions
fn get_str_field(map: &serde_yaml::Mapping, key: &str) -> String {
    map.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

fn get_bool_field(map: &serde_yaml::Mapping, key: &str) -> bool {
    map.get(key).and_then(|v| v.as_bool()).unwrap_or_default()
}

/// Maps OpenAPI types and formats to Rust types
pub fn openapi_type_to_rust_type(openapi_type: &str, format: Option<&str>) -> &'static str {
    match (openapi_type, format) {
        ("string", _) => "String",
        ("integer", Some("int64")) => "i64",
        ("integer", _) => "i32",
        ("number", Some("double")) => "f64",
        ("number", _) => "f32",
        ("boolean", _) => "bool",
        _ => "String",
    }
}
