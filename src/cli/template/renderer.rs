use super::metadata::TemplateMetadata;
use lpm_core::LpmResult;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Renders a template to a target directory
pub struct TemplateRenderer {
    template_dir: PathBuf,
    metadata: TemplateMetadata,
}

impl TemplateRenderer {
    pub fn new(template_dir: PathBuf) -> LpmResult<Self> {
        let metadata = TemplateMetadata::load(&template_dir)?;
        Ok(Self {
            template_dir,
            metadata,
        })
    }

    /// Render the template to the target directory
    pub fn render(&self, target_dir: &Path, variables: &HashMap<String, String>) -> LpmResult<()> {
        // Validate required variables
        for var in &self.metadata.variables {
            if var.required && !variables.contains_key(&var.name) {
                return Err(lpm_core::LpmError::Config(format!(
                    "Required template variable '{}' not provided",
                    var.name
                )));
            }
        }

        // Create target directory if it doesn't exist
        fs::create_dir_all(target_dir)?;

        // Render all files in the template directory
        self.render_directory(&self.template_dir, target_dir, variables)?;

        Ok(())
    }

    fn render_directory(
        &self,
        source: &Path,
        target: &Path,
        variables: &HashMap<String, String>,
    ) -> LpmResult<()> {
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let source_path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Skip template.yaml and other metadata files
            if file_name_str == "template.yaml" || file_name_str.starts_with('.') {
                continue;
            }

            let target_path = target.join(&file_name);

            if source_path.is_dir() {
                // Recursively render subdirectories
                fs::create_dir_all(&target_path)?;
                self.render_directory(&source_path, &target_path, variables)?;
            } else {
                // Render file with variable substitution
                self.render_file(&source_path, &target_path, variables)?;
            }
        }

        Ok(())
    }

    fn render_file(
        &self,
        source: &Path,
        target: &Path,
        variables: &HashMap<String, String>,
    ) -> LpmResult<()> {
        let content = fs::read_to_string(source)?;
        let rendered = self.substitute_variables(&content, variables);
        fs::write(target, rendered)?;
        Ok(())
    }

    fn substitute_variables(&self, content: &str, variables: &HashMap<String, String>) -> String {
        let mut result = content.to_string();

        // Substitute {{variable}} patterns
        for (key, value) in variables {
            let pattern = format!("{{{{{}}}}}", key);
            result = result.replace(&pattern, value);
        }

        // Also handle default values for variables not provided
        for var in &self.metadata.variables {
            if !variables.contains_key(&var.name) {
                if let Some(default) = &var.default {
                    let pattern = format!("{{{{{}}}}}", var.name);
                    result = result.replace(&pattern, default);
                }
            }
        }

        result
    }
}
