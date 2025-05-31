//! Template system for code generation

use crate::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

/// Template manager for code generation
pub struct TemplateManager {
    tera: Tera,
    template_dir: PathBuf,
}

impl TemplateManager {
    /// Reload all templates from the template directory.
    pub fn reload_templates(&mut self) -> crate::Result<()> {
        self.tera = Tera::default();
        self.tera.add_template_files(vec![
            (
                self.template_dir.join("handler.rs.tera"),
                Some("handler.rs"),
            ),
            (
                self.template_dir.join("handlers_mod.rs.tera"),
                Some("handlers_mod.rs"),
            ),
        ])?;
        Ok(())
    }

    /// Create a new template manager with the given template directory
    pub fn new(template_dir: impl AsRef<Path>) -> crate::Result<Self> {
        let template_dir = template_dir.as_ref().to_path_buf();
        let _template_pattern = template_dir
            .join("**/*.tera")
            .to_string_lossy()
            .into_owned();

        let mut tera = Tera::default();
        tera.add_template_files(vec![
            (template_dir.join("handler.rs.tera"), Some("handler.rs")),
            (
                template_dir.join("handlers_mod.rs.tera"),
                Some("handlers_mod.rs"),
            ),
        ])?;

        Ok(Self { tera, template_dir })
    }

    /// Generate a handler file from a template
    pub async fn generate_handler<T: Serialize>(
        &self,
        template_name: &str,
        context: &T,
        output_path: impl AsRef<Path>,
    ) -> Result<()> {
        let tera_context = Context::from_serialize(context)?;
        let rendered = self.tera.render(template_name, &tera_context)?;

        tokio::fs::write(output_path, rendered).await?;
        Ok(())
    }

    /// Generate multiple handler files from a template
    pub async fn generate_handlers<T: Serialize>(
        &self,
        template_name: &str,
        contexts: &[T],
        output_dir: impl AsRef<Path>,
    ) -> crate::Result<()> {
        for context in contexts {
            let tera_context = Context::from_serialize(context)?;
            let rendered = self.tera.render(template_name, &tera_context)?;

            // Create output directory if it doesn't exist
            tokio::fs::create_dir_all(&output_dir).await?;

            // Generate the output file path based on context
            let file_name = if let Some(name) = tera_context.get("endpoint") {
                format!("{}.rs", name.as_str().unwrap_or("handler"))
            } else {
                "handler.rs".to_string()
            };

            let output_path = output_dir.as_ref().join(file_name);
            tokio::fs::write(output_path, rendered).await?;
        }
        Ok(())
    }

    /// Generate the handlers module file
    pub async fn generate_handlers_mod(
        &self,
        endpoints: Vec<HashMap<String, String>>,
        output_path: impl AsRef<Path>,
    ) -> Result<()> {
        let mut context = Context::new();
        context.insert("endpoints", &endpoints);

        let rendered = self.tera.render("handlers_mod.rs", &context)?;
        tokio::fs::write(output_path, rendered).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_template_manager() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let template_dir = temp_dir.path().join("templates");
        tokio::fs::create_dir(&template_dir).await?;

        // Create test template
        let test_handler_template = r#"
        // Generated handler for {{ endpoint }}
        pub fn {{ endpoint }}_handler() {
            println!("{{ description }}");
        }
        "#;

        let test_handlers_mod_template = r#"
        // Generated handlers module
        pub mod handlers {
            {{ handlers }}
        }
        "#;

        tokio::fs::write(template_dir.join("handler.rs.tera"), test_handler_template).await?;
        tokio::fs::write(
            template_dir.join("handlers_mod.rs.tera"),
            test_handlers_mod_template,
        )
        .await?;

        let manager = TemplateManager::new(&template_dir)?;

        let context = json!({
            "endpoint": "test",
            "description": "Test handler"
        });

        let output_dir = temp_dir.path().join("output");
        tokio::fs::create_dir(&output_dir).await?;

        manager
            .generate_handler("handler.rs", &context, output_dir.join("test_handler.rs"))
            .await?;

        let generated = tokio::fs::read_to_string(output_dir.join("test_handler.rs")).await?;
        assert!(generated.contains("Test handler"));
        assert!(generated.contains("test_handler"));

        Ok(())
    }
}
