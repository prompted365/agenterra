//! Template system for code generation

// Internal imports (std, crate)
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{
    builders::EndpointContext,
    config::Config,
    error::Result,
    manifest::TemplateManifest,
    openapi::{OpenApiContext, OpenApiOperation},
    template::Template,
    template_options::TemplateOptions,
    utils::to_snake_case,
};

// External imports (alphabetized)
use serde::Serialize;
use serde_json::{Map, Value as JsonValue, json};
use tera::{Context, Tera};
use tokio::{fs, task};

type TeraCache = std::collections::HashMap<String, Arc<Tera>>;

/// Manages loading and rendering of code generation templates
#[derive(Debug, Clone)]
pub struct TemplateManager {
    /// Cached Tera template engine instance
    tera: Arc<Tera>,
    /// Path to the template directory
    template_dir: PathBuf,
    /// The template kind (language/framework)
    template_kind: Template,
    /// The template manifest
    manifest: TemplateManifest,
}

impl TemplateManager {
    /// Create a new TemplateManager for the given template kind.
    /// If `template_dir` is provided, it will be used directly. Otherwise, the template
    /// directory will be discovered based on the language and framework.
    /// Creates a new TemplateManager with a cached Tera instance
    pub async fn new(template_kind: Template, template_dir: Option<PathBuf>) -> Result<Self> {
        let template_dir = if let Some(dir) = template_dir {
            // Use the provided template directory directly
            if !dir.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Template directory not found: {}", dir.display()),
                )
                .into());
            }
            tokio::fs::canonicalize(&dir).await.map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Failed to canonicalize template directory: {}", e),
                )
            })?
        } else {
            // Discover the template directory based on the template kind
            Self::discover_template_dir(&template_kind).await?
        };

        // Convert template_dir to string for caching
        let template_dir_str = template_dir.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Template path contains invalid UTF-8",
            )
        })?;

        // Get or initialize the cached Tera instance
        let tera = {
            use once_cell::sync::Lazy;
            use std::sync::Mutex;

            static TERA_CACHE: Lazy<Mutex<TeraCache>> = Lazy::new(|| Mutex::new(TeraCache::new()));

            let mut cache = TERA_CACHE.lock().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to acquire Tera cache lock: {}", e),
                )
            })?;

            // Check if we have a cached Tera instance for this template directory
            if let Some(cached_tera) = cache.get(template_dir_str) {
                cached_tera.clone()
            } else {
                // Initialize a new Tera instance and cache it
                let mut tera =
                    Tera::new(&format!("{}/**/*.tera", template_dir_str)).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Failed to initialize Tera: {}", e),
                        )
                    })?;

                // Auto-escape all files
                tera.autoescape_on(vec![".html", ".htm", ".xml", ".md"]);

                let tera_arc = Arc::new(tera);
                cache.insert(template_dir_str.to_string(), tera_arc.clone());
                tera_arc
            }
        };

        // Load the template manifest
        let manifest_path = template_dir.join("manifest.yaml");
        let manifest = if manifest_path.exists() {
            let manifest_content = tokio::fs::read_to_string(&manifest_path).await?;
            serde_yaml::from_str(&manifest_content).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to parse manifest: {}", e),
                )
            })?
        } else {
            // Default manifest if none exists
            TemplateManifest::default()
        };

        Ok(Self {
            tera,
            template_dir,
            template_kind,
            manifest,
        })
    }

    /// Discover the template directory based on the template kind
    async fn discover_template_dir(template_kind: &Template) -> Result<PathBuf> {
        // Try to get the template directory from the template kind
        let template_dir = template_kind.template_dir().await.map_err(|e| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Failed to discover template directory: {}", e),
            )
        })?;

        // Verify the directory exists
        if !template_dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Template directory not found: {}", template_dir.display()),
            )
            .into());
        }

        // Convert to absolute path
        tokio::fs::canonicalize(&template_dir).await.map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to canonicalize template path: {}", e),
            )
            .into()
        })
    }

    /// Get the template kind this template manager is configured for
    pub fn template_kind(&self) -> Template {
        self.template_kind
    }

    /// Get the path to the template directory
    pub fn template_dir(&self) -> &Path {
        &self.template_dir
    }

    /// Get a reference to the Tera template engine
    pub fn tera(&self) -> &Tera {
        &self.tera
    }

    /// Reload all templates from the template directory.
    /// This is a no-op in the cached implementation since templates are loaded on demand.
    pub async fn reload_templates(&self) -> Result<()> {
        // No-op in the cached implementation
        // Templates are loaded on demand and cached automatically
        Ok(())
    }

    /// Discovers all template files in the given directory and its subdirectories.
    ///
    /// This function uses `spawn_blocking` to avoid blocking the async runtime
    /// during filesystem operations.
    ///
    /// # Arguments
    /// * `dir` - The directory to search for template files
    ///
    /// # Returns
    /// A `Result` containing a vector of paths to template files with the `.tera` extension
    pub async fn discover_template_files(dir: &Path) -> Result<Vec<PathBuf>> {
        let dir_buf = dir.to_path_buf();

        task::spawn_blocking(move || {
            let mut templates = Vec::new();

            fn walk_dir(dir: &Path, templates: &mut Vec<PathBuf>) -> std::io::Result<()> {
                for entry in std::fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_dir() {
                        walk_dir(&path, templates)?;
                    } else if path.extension().and_then(|s| s.to_str()) == Some("tera") {
                        templates.push(path);
                    }
                }
                Ok(())
            }

            walk_dir(&dir_buf, &mut templates)?;
            Ok(templates)
        })
        .await
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to join blocking task: {}", e),
            )
        })?
    }

    /// Generate a handler file from a template
    pub async fn generate_handler<T: Serialize>(
        &self,
        template_name: &str,
        context: &T,
        output_path: impl AsRef<Path>,
    ) -> Result<()> {
        self.generate_with_context(template_name, context, output_path)
            .await
    }

    /// Get a reference to the template manifest
    pub fn manifest(&self) -> &TemplateManifest {
        &self.manifest
    }

    /// Generate a file from a template with a custom context
    pub async fn generate_with_context<T: Serialize>(
        &self,
        template_name: &str,
        context: &T,
        output_path: impl AsRef<Path>,
    ) -> Result<()> {
        let output_path = output_path.as_ref();

        // First validate required context variables
        let context_value = serde_json::to_value(context).map_err(|e| {
            crate::error::Error::template(format!("Failed to serialize context: {}", e))
        })?;

        let context_map = context_value.as_object().ok_or_else(|| {
            crate::error::Error::template("Context must be a JSON object".to_string())
        })?;

        // Define required variables per template type
        let required_vars: &[&str] = match template_name {
            // Add template-specific required variables here
            // Example: "handlers/endpoint.rs" => &["endpoint", "parameters_type"],
            _ => &[],
        };

        Self::validate_context(template_name, context_map, required_vars)?;

        let parent = output_path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid output path: {}", output_path.display()),
            )
        })?;

        tokio::fs::create_dir_all(parent).await?;

        // Log the template being rendered
        log::debug!("Rendering template: {}", template_name);
        log::debug!("Output path: {}", output_path.display());
        log::debug!("Parent directory: {}", parent.display());

        // Build Tera Context from the already parsed context_map
        let mut tera_context = Context::new();
        for (k, v) in context_map {
            tera_context.insert(k, v);
        }

        // Verify template exists
        log::debug!("Checking if template exists: {}", template_name);
        self.tera.get_template(template_name).map_err(|e| {
            crate::error::Error::template(format!("Template not found: {} - {}", template_name, e))
        })?;

        log::debug!("Found template: {}", template_name);
        log::debug!(
            "Available templates: {:?}",
            self.tera.get_template_names().collect::<Vec<_>>()
        );

        // Render the template with detailed error reporting
        let content = match self.tera.render(template_name, &tera_context) {
            Ok(content) => content,
            Err(e) => {
                // Get the template source for better error reporting
                let template_source =
                    match std::fs::read_to_string(self.template_dir.join(template_name)) {
                        Ok(source) => source,
                        Err(_) => "<unable to read template file>".to_string(),
                    };

                return Err(crate::error::Error::template(format!(
                    "Failed to render template '{}': {}\nTemplate source:\n{}",
                    template_name, e, template_source
                )));
            }
        };

        log::debug!(
            "Rendered content for {} ({} bytes):\n{}",
            template_name,
            content.len(),
            if content.len() > 200 {
                format!("{}... (truncated)", &content[..200])
            } else {
                content.clone()
            }
        );

        // Ensure the parent directory exists
        log::debug!("Ensuring parent directory exists: {}", parent.display());
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            log::error!("Failed to create directory: {}", e);
            return Err(crate::error::Error::Io(e));
        }

        // Write the output file
        log::debug!("Writing to output file: {}", output_path.display());
        tokio::fs::write(&output_path, &content).await?;

        log::debug!("Successfully wrote template to: {}", output_path.display());
        Ok(())
    }

    /// List all available templates
    pub fn list_templates(&self) -> Vec<String> {
        self.tera
            .get_template_names()
            .map(|s| s.to_string())
            .collect()
    }

    /// Check if a template exists
    pub fn has_template(&self, name: &str) -> bool {
        self.tera.get_template(name).is_ok()
    }

    /// Generate code from loaded templates based on the OpenAPI spec and options
    pub async fn generate(
        &self,
        spec: &OpenApiContext,
        config: &Config,
        template_opts: Option<TemplateOptions>,
    ) -> Result<()> {
        // Create output directory
        let output_path = PathBuf::from(&config.output_dir);
        fs::create_dir_all(&output_path).await?;

        // Build the context for template rendering
        let (context, operations) = self.build_context(spec, &template_opts).await?;

        log::debug!("Starting template processing with context: {:#?}", context);
        // Process all template files
        self.process_template_files(&context, &output_path, &template_opts, spec, operations)
            .await
            .map_err(|e| {
                log::error!("Template processing failed: {}", e);
                e
            })?;

        // Run post-generation hooks if any
        self.execute_post_generation_hooks(&output_path).await?;

        Ok(())
    }

    /// Build the complete template context from OpenAPI spec
    async fn build_context(
        &self,
        spec: &OpenApiContext,
        template_opts: &Option<TemplateOptions>,
    ) -> Result<(serde_json::Value, Vec<OpenApiOperation>)> {
        let mut base_map = serde_json::Map::new();

        // Add project name from spec title
        if let Some(title) = spec
            .json
            .get("info")
            .and_then(|info| info.get("title"))
            .and_then(|t| t.as_str())
        {
            // Sanitize the title to be a valid Rust package name
            let sanitized_name = to_snake_case(title);
            base_map.insert("project_name".to_string(), json!(sanitized_name));
            base_map.insert("project_title".to_string(), json!(title));
        }

        // Add API version from spec
        if let Some(api_version) = spec
            .json
            .get("info")
            .and_then(|info| info.get("version"))
            .and_then(|v| v.as_str())
        {
            base_map.insert("api_version".to_string(), json!(api_version));
        }

        // Add MCP Agent instructions if provided
        if let Some(opts) = template_opts {
            if let Some(instructions) = &opts.agent_instructions {
                base_map.insert("agent_instructions".to_string(), instructions.clone());
            }
        }

        // Add the full spec to the context if needed
        if let Ok(spec_value) = serde_json::to_value(spec) {
            base_map.insert("spec".to_string(), spec_value);
        }

        // Add spec file name for reference in templates
        base_map.insert("spec_file_name".to_string(), json!("openapi.json"));

        // Extract operations from the OpenAPI spec
        let operations = spec.parse_operations().await?;

        // Transform endpoints using language-specific builder
        let endpoints =
            EndpointContext::transform_endpoints(self.template_kind, operations.clone())?;
        base_map.insert("endpoints".to_string(), json!(endpoints));

        // Add server configuration variables needed by templates
        base_map.insert("log_file".to_string(), json!("mcpgen"));
        base_map.insert("server_port".to_string(), json!(8080));

        // Add any template options to the context if provided
        if let Some(opts) = template_opts {
            // Override defaults with template options if provided
            if let Some(port) = opts.server_port {
                base_map.insert("server_port".to_string(), json!(port));
            }
            if let Some(log_file) = &opts.log_file {
                base_map.insert("log_file".to_string(), json!(log_file));
            }
        }

        // For debugging, log the context keys
        let keys_str: Vec<String> = base_map.keys().map(|k| k.to_string()).collect();
        log::debug!("Template context keys: {}", keys_str.join(", "));

        Ok((serde_json::Value::Object(base_map), operations))
    }

    /// Process all template files with the given context
    async fn process_template_files(
        &self,
        base_context: &serde_json::Value,
        output_path: &Path,
        template_opts: &Option<TemplateOptions>,
        spec: &OpenApiContext,
        operations: Vec<OpenApiOperation>,
    ) -> Result<()> {
        // Pre-load endpoint contexts for operation templates if needed
        let needs_endpoints = self
            .manifest
            .files
            .iter()
            .any(|f| f.for_each.as_deref() == Some("endpoint"));

        let endpoint_contexts = if needs_endpoints {
            operations
        } else {
            Vec::new()
        };

        for file in &self.manifest.files {
            // Handle per-endpoint generation if specified
            if file.for_each.as_deref() == Some("endpoint") {
                // Convert base_context to Context for operation processing
                let context = if let serde_json::Value::Object(obj) = base_context {
                    Context::from_value(serde_json::Value::Object(obj.clone()))
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                } else {
                    Context::new()
                };

                self.process_operation_file(
                    file,
                    &context,
                    output_path,
                    &endpoint_contexts,
                    &template_opts,
                    spec,
                )
                .await?;
            } else {
                self.process_single_file(file, base_context, output_path)
                    .await?;
            }
        }

        Ok(())
    }

    /// Process a single template file
    async fn process_single_file(
        &self,
        file: &crate::manifest::TemplateFile,
        base_context: &serde_json::Value,
        output_path: &Path,
    ) -> Result<()> {
        let dest_path = output_path.join(&file.destination);

        // Create parent directories if they don't exist
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Create file-specific context
        let file_context = self.create_file_context(base_context, file).await?;

        // Convert file_context to Context
        let tera_context = if let serde_json::Value::Object(obj) = file_context {
            Context::from_value(serde_json::Value::Object(obj))
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        } else {
            Context::new()
        };

        // Log the template context for debugging
        log::debug!(
            "Rendering template: {} with context: {:#?}",
            file.source,
            tera_context
        );

        // Render the template
        let rendered = match self.tera.render(&file.source, &tera_context) {
            Ok(r) => r,
            Err(e) => {
                // Convert tera::Context to a serializable Map before serializing
                let context_map: std::collections::HashMap<String, serde_json::Value> =
                    tera_context
                        .into_json()
                        .as_object()
                        .map(|obj| obj.clone().into_iter().collect())
                        .unwrap_or_default();
                let context_json = serde_json::to_string_pretty(&context_map)
                    .unwrap_or_else(|_| "Failed to serialize context".to_string());
                log::error!(
                    "Failed to render template {}: {}\nTemplate context: {}",
                    file.source,
                    e,
                    context_json
                );
                return Err(crate::error::Error::Io(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Template rendering failed for {}: {}", file.source, e),
                )));
            }
        };

        // Write the output file
        fs::write(&dest_path, rendered).await.map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to write file {}: {}", dest_path.display(), e),
            )
        })?;

        Ok(())
    }

    /// Process a template file for each operation
    async fn process_operation_file(
        &self,
        file: &crate::manifest::TemplateFile,
        base_context: &Context,
        output_path: &Path,
        operations: &[OpenApiOperation],
        template_opts: &Option<TemplateOptions>,
        spec: &OpenApiContext,
    ) -> Result<()> {
        // Create schemas directory
        let schemas_dir = output_path.join("schemas");
        fs::create_dir_all(&schemas_dir).await?;

        for operation in operations {
            // Language-specific fields like fn_name must be injected by a builder; OpenApiOperation is language-agnostic.
            let include = template_opts
                .as_ref()
                .map(|opts| {
                    opts.all_operations
                        || opts.include_operations.is_empty()
                        || opts.include_operations.contains(&operation.id)
                })
                .unwrap_or(true);
            let exclude = template_opts
                .as_ref()
                .map(|opts| opts.exclude_operations.contains(&operation.id))
                .unwrap_or(false);

            if include && !exclude {
                let mut context = base_context.clone();

                let builder = EndpointContext::get_builder(self.template_kind);
                let endpoint_context = builder.build(operation)?;

                // Merge the endpoint context into the template context
                if let Some(obj) = endpoint_context.as_object() {
                    for (key, value) in obj {
                        context.insert(key, value);
                    }
                }

                // Add operation metadata
                context.insert("operation_id", &operation.id);
                context.insert("method", &"get"); // TODO: Get actual method
                context.insert("path", &format!("/{}", &operation.id)); // TODO: Get actual path

                // Insert OpenAPI-native fields
                context.insert("operation_id", &operation.id);

                // Sanitize and add text fields
                let sanitized_summary = operation.summary.as_deref().map(|s| {
                    s.chars()
                        .filter(|c| c.is_ascii_alphanumeric() || c.is_whitespace())
                        .collect::<String>()
                        .trim()
                        .to_string()
                });

                let sanitized_description = operation.description.as_deref().map(|s| {
                    s.chars()
                        .filter(|c| {
                            c.is_ascii_alphanumeric() || c.is_whitespace() || *c == '.' || *c == ','
                        })
                        .collect::<String>()
                        .trim()
                        .to_string()
                });

                context.insert("summary", &sanitized_summary);
                context.insert("description", &sanitized_description);
                context.insert("deprecated", &operation.deprecated);

                // Add tags with proper sanitization
                let sanitized_tags: Vec<String> = operation
                    .tags
                    .as_ref()
                    .map(|tags| {
                        tags.iter()
                            .map(|t| t.trim().replace("\n", " ").replace("\r", " "))
                            .collect()
                    })
                    .unwrap_or_default();
                context.insert("tags", &sanitized_tags);

                // Extract and process parameters with proper error handling
                let parameter_info: Vec<serde_json::Value> = operation
                    .parameters
                    .as_ref()
                    .map(|params| {
                        params
                            .iter()
                            .map(|p| {
                                let mut param_obj = serde_json::Map::new();

                                // Required fields
                                param_obj.insert("name".to_string(), json!(&p.name));
                                param_obj.insert("in".to_string(), json!(&p.in_));

                                // Optional fields with their correct names
                                if let Some(desc) = &p.description {
                                    param_obj.insert("description".to_string(), json!(desc));
                                }

                                // Handle required field with path parameter default
                                let is_required = p.required.unwrap_or_else(|| p.in_ == "path");
                                param_obj.insert("required".to_string(), json!(is_required));

                                // Add schema if available
                                if let Some(schema) = &p.schema {
                                    param_obj.insert("schema".to_string(), schema.clone());
                                }

                                // Add content if available (for complex parameters)
                                if let Some(content) = &p.content {
                                    param_obj.insert("content".to_string(), json!(content));
                                }

                                // Add examples if available
                                if let Some(examples) = &p.examples {
                                    param_obj.insert("examples".to_string(), json!(examples));
                                }

                                // Add other optional fields
                                if let Some(deprecated) = p.deprecated {
                                    param_obj.insert("deprecated".to_string(), json!(deprecated));
                                }

                                if let Some(style) = &p.style {
                                    param_obj.insert("style".to_string(), json!(style));
                                }

                                if let Some(explode) = p.explode {
                                    param_obj.insert("explode".to_string(), json!(explode));
                                }

                                // Add allow_empty_value with correct serialization name
                                if let Some(allow_empty) = p.allow_empty_value {
                                    param_obj
                                        .insert("allowEmptyValue".to_string(), json!(allow_empty));
                                }

                                // Add allow_reserved with correct serialization name
                                if let Some(allow_reserved) = p.allow_reserved {
                                    param_obj
                                        .insert("allowReserved".to_string(), json!(allow_reserved));
                                }

                                // Add any vendor extensions
                                if !p.vendor_extensions.is_empty() {
                                    for (key, value) in &p.vendor_extensions {
                                        if key.starts_with("x-") {
                                            param_obj.insert(key.clone(), value.clone());
                                        }
                                    }
                                }

                                json!(param_obj)
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                // Add parameters to context
                context.insert(
                    "parameters",
                    &operation.parameters.clone().unwrap_or_default(),
                );
                context.insert("parameter_info", &parameter_info);

                // Process responses
                context.insert("responses", &operation.responses);

                // Add request body if present with sanitized properties
                if let Some(request_body) = &operation.request_body {
                    context.insert("has_request_body", &true);
                    context.insert("request_body", request_body);

                    // Use the operation's method to extract request body properties
                    match spec.extract_request_body_properties(operation) {
                        Ok((props, _)) if !props.is_null() => {
                            let property_info = OpenApiContext::extract_property_info(&props);
                            context.insert("request_properties", &property_info);
                        }
                        _ => {
                            // Fallback to basic property extraction if the above fails
                            if let Some(content) = request_body
                                .get("content")
                                .and_then(serde_json::Value::as_object)
                            {
                                for (_content_type, media_type) in content {
                                    if let Some(schema) = media_type.get("schema") {
                                        let property_info =
                                            OpenApiContext::extract_property_info(schema);
                                        context.insert("request_properties", &property_info);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    context.insert("has_request_body", &false);
                }

                // Add security requirements if present
                if let Some(security) = &operation.security {
                    context.insert("security", security);
                }

                // Add sanitized names for use in generated code
                let sanitized_operation_name = operation
                    .id
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect::<String>();
                context.insert("sanitized_operation_name", &sanitized_operation_name);

                let endpoint_fs = if let Some(endpoint_val) = endpoint_context.get("endpoint_fs") {
                    endpoint_val.as_str().unwrap_or(&operation.id)
                } else {
                    &operation.id
                };

                let endpoint_name = if let Some(endpoint_val) = endpoint_context.get("endpoint") {
                    endpoint_val.as_str().unwrap_or(&operation.id)
                } else {
                    &operation.id
                };

                let sanitized_filename = to_snake_case(endpoint_fs);
                context.insert("sanitized_filename", &sanitized_filename);

                log::debug!("Processing template for operation: {}", operation.id);

                // Generate schema file with proper schema extraction
                // Use snake_case for the filename to match MCP conventions
                let schema_filename = to_snake_case(&operation.id);
                let schema_path = schemas_dir.join(format!("{}.json", schema_filename));
                let mut schema_value = serde_json::to_value(&operation)?;

                // Dereference all $ref in the schema
                self.dereference_schema_refs(&mut schema_value, spec)?;

                // Remove null values from the schema
                schema_value
                    .as_object_mut()
                    .unwrap()
                    .retain(|_, v| v != &json!(null));

                let schema_json = serde_json::to_string_pretty(&schema_value)?;
                fs::write(&schema_path, schema_json).await.map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "Failed to write schema file {}: {}",
                            schema_path.display(),
                            e
                        ),
                    )
                })?;

                // Generate the output path with sanitized operation_id
                let output_file = file
                    .destination
                    .replace("{{operation_id}}", endpoint_fs)
                    .replace("{operation_id}", endpoint_fs)
                    .replace("{{endpoint}}", endpoint_name)
                    .replace("{endpoint}", endpoint_name);
                let output_path = output_path.join(&output_file);

                // Create parent directories if they don't exist
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent).await?;
                }

                // Render the template
                let rendered = self
                    .tera
                    .render(file.source.as_str(), &context)
                    .map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to render template {}: {}", file.source, e),
                        )
                    })?;

                // Write the file
                fs::write(&output_path, rendered).await.map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to write file {}: {}", output_path.display(), e),
                    )
                })?;
            }
        }
        Ok(())
    }

    /// Validates that all required context variables are present
    fn validate_context(
        template: &str,
        context: &Map<String, JsonValue>,
        required_vars: &[&str],
    ) -> crate::Result<()> {
        let mut missing = Vec::new();

        for var in required_vars {
            if !context.contains_key(*var) {
                missing.push(var.to_string());
            }
        }

        if !missing.is_empty() {
            return Err(crate::Error::template(format!(
                "Missing required context variables for template '{}': {}",
                template,
                missing.join(", ")
            )));
        }
        Ok(())
    }

    /// Execute post-generation hooks from the manifest
    pub async fn execute_post_generation_hooks(
        &self,
        output_path: &std::path::Path,
    ) -> crate::Result<()> {
        for hook in &self.manifest.hooks.post_generate {
            match hook.as_str() {
                "cargo_fmt" => {
                    if let Ok(mut cmd) = std::process::Command::new("cargo")
                        .args(["fmt", "--"])
                        .current_dir(output_path)
                        .spawn()
                    {
                        let _ = cmd.wait();
                    }
                }
                _ => log::warn!("Unknown post-generation hook: {}", hook),
            }
        }
        Ok(())
    }

    /// Merge base context with file context, giving precedence to file context keys
    pub async fn create_file_context(
        &self,
        base_context: &serde_json::Value,
        file: &crate::manifest::TemplateFile,
    ) -> crate::Result<serde_json::Value> {
        let mut context = if let serde_json::Value::Object(file_ctx) = &file.context {
            file_ctx.clone()
        } else {
            serde_json::Map::new()
        };
        if let serde_json::Value::Object(base_map) = base_context {
            for (k, v) in base_map {
                if !context.contains_key(k) {
                    context.insert(k.clone(), v.clone());
                }
            }
        }
        Ok(serde_json::Value::Object(context))
    }

    /// Dereference all $ref in a JSON value by replacing them with actual schema definitions
    fn dereference_schema_refs(
        &self,
        value: &mut serde_json::Value,
        spec: &OpenApiContext,
    ) -> Result<()> {
        match value {
            serde_json::Value::Object(map) => {
                // Check if this object contains a $ref
                if let Some(ref_value) = map.get("$ref") {
                    if let Some(ref_str) = ref_value.as_str() {
                        if ref_str.starts_with("#/components/schemas/") {
                            let schema_name = ref_str.trim_start_matches("#/components/schemas/");

                            // Get the actual schema definition
                            if let Some(components) = spec.json.get("components") {
                                if let Some(schemas) = components.get("schemas") {
                                    if let Some(schema_def) = schemas.get(schema_name) {
                                        // Replace the entire object with the dereferenced schema
                                        *value = schema_def.clone();
                                        // Continue dereferencing in the new value
                                        self.dereference_schema_refs(value, spec)?;
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }

                // Recursively process all values in the object
                for (_, v) in map.iter_mut() {
                    self.dereference_schema_refs(v, spec)?;
                }
            }
            serde_json::Value::Array(arr) => {
                // Recursively process all items in the array
                for item in arr.iter_mut() {
                    self.dereference_schema_refs(item, spec)?;
                }
            }
            _ => {} // Other types don't need processing
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Map, json};

    #[test]
    fn test_validate_context() {
        let mut context = Map::new();
        context.insert("foo".to_string(), json!("bar"));
        context.insert("baz".to_string(), json!(123));

        // Test with no required vars
        assert!(TemplateManager::validate_context("test_template", &context, &[]).is_ok());

        // Test with required vars that exist
        assert!(TemplateManager::validate_context("test_template", &context, &["foo"]).is_ok());
        assert!(TemplateManager::validate_context("test_template", &context, &["baz"]).is_ok());
        assert!(
            TemplateManager::validate_context("test_template", &context, &["foo", "baz"]).is_ok()
        );

        // Test with missing required var
        let result = TemplateManager::validate_context("test_template", &context, &["missing"]);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Missing required context variables"));
            assert!(e.to_string().contains("missing"));
        }

        // Test with mix of existing and missing vars
        let result =
            TemplateManager::validate_context("test_template", &context, &["foo", "missing"]);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_template_manager() -> Result<()> {
        use tempfile::TempDir;

        // Create a test directory with template files
        let temp = TempDir::new().unwrap();
        let td = temp.path();
        // Write a simple template file
        let tera_content = "Message: {{message}}";
        tokio::fs::write(td.join("foo.tera"), tera_content)
            .await
            .unwrap();
        // Write the manifest.yaml
        let manifest = r#"
name: test-template
description: Test template
version: 0.1.0
language: rust
files:
  - source: foo.tera
    destination: foo.txt
    context:
      message: hello
"#;
        tokio::fs::write(td.join("manifest.yaml"), manifest)
            .await
            .unwrap();
        // Create a minimal OpenAPI spec file
        let spec_json = json!({"paths": {}});
        let spec_file = td.join("spec.json");
        tokio::fs::write(&spec_file, spec_json.to_string())
            .await
            .unwrap();
        // Prepare config pointing to output directory
        let out_dir = temp.path().join("out");
        let config = Config::new(spec_file.to_str().unwrap(), out_dir.to_str().unwrap());
        // Initialize TemplateManager with our temp dir
        let manager = TemplateManager::new(Template::RustAxum, Some(td.to_path_buf())).await?;
        // Load spec and generate
        let spec = OpenApiContext::from_file(&spec_file).await?;
        manager.generate(&spec, &config, None).await?;
        // Read and verify the generated file
        let result = tokio::fs::read_to_string(out_dir.join("foo.txt"))
            .await
            .unwrap();
        assert_eq!(result.trim(), "Message: hello");
        Ok(())
    }
}
