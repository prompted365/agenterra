//! Schema parsing and generation functionality for OpenAPI specs

use std::path::Path;
use serde_json::{Map, Value as JsonValue, json};
use crate::openapi::types::PropertyInfo;
use anyhow::Result;

/// Build a properties schema from a slice of PropertyInfo
pub fn build_properties_schema(properties: &[PropertyInfo]) -> Map<String, JsonValue> {
    let mut schema = Map::new();
    
    for prop in properties {
        let mut property_schema = Map::new();
        property_schema.insert("type".to_string(), JsonValue::String(prop.openapi_type.clone()));
        
        if let Some(format) = &prop.format {
            property_schema.insert("format".to_string(), JsonValue::String(format.clone()));
        }
        
        if !prop.title.is_empty() {
            property_schema.insert("title".to_string(), JsonValue::String(prop.title.clone()));
        }
        
        if !prop.description.is_empty() {
            property_schema.insert("description".to_string(), JsonValue::String(prop.description.clone()));
        }
        
        if !prop.example.is_empty() {
            property_schema.insert("example".to_string(), JsonValue::String(prop.example.clone()));
        }
        
        schema.insert(prop.name.clone(), JsonValue::Object(property_schema));
    }
    
    schema
}

/// Build an OpenAPI response schema
pub fn build_response_schema(properties_struct_name: &str) -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "meta": {
                "type": "object",
                "properties": {
                    "total": { "type": "integer" },
                    "count": { "type": "integer" },
                    "limit": { "type": "integer" },
                    "offset": { "type": "integer" }
                }
            },
            "data": {
                "type": "array",
                "items": {
                    "$ref": format!("#/components/schemas/{}", properties_struct_name)
                }
            },
            "totals": {
                "type": "object",
                "description": "Optional aggregate totals"
            }
        }
    })
}

/// Generate endpoint schema files for agent/resource introspection
pub async fn generate_endpoint_schema_files(endpoints: &[JsonValue], output_dir: &Path) -> Result<()> {
    use tokio::fs;
    
    // Create schemas directory if it doesn't exist
    let schemas_dir = output_dir.join("schemas");
    fs::create_dir_all(&schemas_dir).await?;
    
    for endpoint in endpoints {
        let path = endpoint["path"].as_str().unwrap_or_default();
        if path.is_empty() {
            continue;
        }
        
        // Generate schema file name from path
        let schema_name = path
            .trim_start_matches('/')
            .replace('/', "_")
            .replace('{', "")
            .replace('}', "");
            
        let schema_path = schemas_dir.join(format!("{}.json", schema_name));
        
        // Build schema
        let schema = build_response_schema(&endpoint["struct_name"].as_str().unwrap_or_default());
        
        // Write schema file
        fs::write(schema_path, serde_json::to_string_pretty(&schema)?).await?;
    }
    
    Ok(())
}

/// Generate OpenAPI JSON file
pub async fn generate_openapi_json(
    endpoints: &[JsonValue],
    schemas: &Map<String, JsonValue>,
    output_path: &Path,
) -> Result<()> {
    use tokio::fs;
    
    let mut paths = Map::new();
    
    // Build paths object from endpoints
    for endpoint in endpoints {
        let path = endpoint["path"].as_str().unwrap_or_default();
        if path.is_empty() {
            continue;
        }
        
        let method = endpoint["method"].as_str().unwrap_or_default().to_lowercase();
        if method.is_empty() {
            continue;
        }
        
        let mut path_item = Map::new();
        let mut operation = Map::new();
        
        // Add operation metadata
        if let Some(summary) = endpoint.get("summary") {
            operation.insert("summary".to_string(), summary.clone());
        }
        if let Some(description) = endpoint.get("description") {
            operation.insert("description".to_string(), description.clone());
        }
        if let Some(tags) = endpoint.get("tags") {
            operation.insert("tags".to_string(), tags.clone());
        }
        
        // Add parameters
        if let Some(parameters) = endpoint.get("parameters") {
            operation.insert("parameters".to_string(), parameters.clone());
        }
        
        // Add responses
        let mut responses = Map::new();
        let mut ok_response = Map::new();
        ok_response.insert(
            "description".to_string(),
            JsonValue::String("Successful response".to_string()),
        );
        ok_response.insert(
            "content".to_string(),
            json!({
                "application/json": {
                    "schema": {
                        "$ref": format!("#/components/schemas/{}_response", endpoint["struct_name"].as_str().unwrap_or_default())
                    }
                }
            }),
        );
        responses.insert("200".to_string(), JsonValue::Object(ok_response));
        operation.insert("responses".to_string(), JsonValue::Object(responses));
        
        // Add operation to path item
        path_item.insert(method, JsonValue::Object(operation));
        
        // Add path item to paths
        paths.insert(path.to_string(), JsonValue::Object(path_item));
    }
    
    // Build final OpenAPI document
    let openapi_doc = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "MCP Server API",
            "version": "1.0.0"
        },
        "paths": paths,
        "components": {
            "schemas": schemas
        }
    });
    
    // Write OpenAPI JSON file
    fs::write(output_path, serde_json::to_string_pretty(&openapi_doc)?).await?;
    
    Ok(())
}
