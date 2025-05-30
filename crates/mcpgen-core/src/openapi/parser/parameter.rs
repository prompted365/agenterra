//! Parameter parsing functionality for OpenAPI specs

use serde_yaml::{Mapping, Value as YamlValue};
use crate::openapi::types::ParameterInfo;

use super::property::openapi_type_to_rust_type;

/// Extract parameter information from an OpenAPI parameter object
pub fn extract_parameter_info(param: &YamlValue) -> Option<ParameterInfo> {
    let map = param.as_mapping()?;
    
    // Get name and basic fields
    let name = get_str_field(map, "name");
    if name.is_empty() {
        return None;
    }

    // Get schema info
    let schema = map.get("schema").unwrap_or(&YamlValue::Null);
    let default_mapping = Mapping::new();
    let schema_map = schema.as_mapping().unwrap_or(&default_mapping);
    
    // Extract type information
    let openapi_type = get_str_field(schema_map, "type");
    let format = schema_map
        .get("format")
        .and_then(|v| v.as_str())
        .map(String::from);

    // Get descriptions
    let description = get_str_field(map, "description");
    let description_raw = description.clone();

    // Get examples
    let example = map
        .get("example")
        .map(|v| v.as_str().unwrap_or("").to_string())
        .unwrap_or_default();
    let example_raw = example.clone();

    // Required field
    let required = get_bool_field(map, "required");

    // Convert schema to JSON
    let schema_json = serde_json::to_value(schema).unwrap_or(serde_json::Value::Null);

    // Map to Rust type
    let rust_type = openapi_type_to_rust_type(&openapi_type, format.as_deref());

    Some(ParameterInfo {
        name,
        rust_type: rust_type.to_string(),
        openapi_type,
        format,
        description,
        description_raw,
        example,
        example_raw,
        required,
        schema: schema_json.into(),
    })
}

/// Extract parameters from a path object, resolving $ref if present
pub fn extract_parameters_for_handler(swagger_doc: &YamlValue, path_obj: &Mapping) -> Vec<ParameterInfo> {
    let mut params = Vec::new();

    // Get parameters array
    if let Some(parameters) = path_obj.get("parameters").and_then(|p| p.as_sequence()) {
        for param in parameters {
            if let Some(ref_str) = param.get("$ref").and_then(|r| r.as_str()) {
                // Resolve parameter reference
                if let Some(resolved) = resolve_parameter_ref(swagger_doc, ref_str) {
                    if let Some(info) = extract_parameter_info(resolved) {
                        params.push(info);
                    }
                }
            } else if let Some(info) = extract_parameter_info(param) {
                params.push(info);
            }
        }
    }

    params
}

/// Resolve a $ref string into a YAML value
fn resolve_parameter_ref<'a>(swagger_doc: &'a YamlValue, ref_str: &str) -> Option<&'a YamlValue> {
    let mut node = swagger_doc;
    let path = ref_str.trim_start_matches("#/").split('/');
    for key in path {
        node = node.as_mapping().and_then(|m| m.get(key))?;
    }
    Some(node)
}

// Helper functions
fn get_str_field(map: &serde_yaml::Mapping, key: &str) -> String {
    map.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

fn get_bool_field(map: &serde_yaml::Mapping, key: &str) -> bool {
    map.get(key)
        .and_then(|v| v.as_bool())
        .unwrap_or_default()
}
