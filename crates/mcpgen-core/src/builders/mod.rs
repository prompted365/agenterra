//! Context builder traits and adapters for language-specific codegen.
pub mod rust;

use crate::openapi::OpenApiOperation;
use crate::template::Template;
use serde_json::Value as JsonValue;

/// Trait for converting an OpenApiOperation into a language-specific context.
pub trait EndpointContextBuilder {
    fn build(&self, op: &OpenApiOperation) -> crate::Result<JsonValue>;
}

pub struct EndpointContext;

impl EndpointContext {
    /// Transform a list of OpenAPI operations into language-specific endpoint contexts
    pub fn transform_endpoints(
        template: Template,
        operations: Vec<OpenApiOperation>,
    ) -> crate::Result<Vec<JsonValue>> {
        let builder = Self::get_builder(template);
        let mut contexts = Vec::new();
        for op in operations {
            contexts.push(builder.build(&op)?);
        }
        Ok(contexts)
    }

    pub fn get_builder(template: Template) -> Box<dyn EndpointContextBuilder> {
        match template {
            Template::RustAxum => Box::new(rust::RustEndpointContextBuilder),
            _ => unimplemented!("Builder not implemented for template: {:?}", template),
        }
    }
}
