use super::metadata::TemplateMetadata;
use lpm_core::core::path::lpm_home;
use lpm_core::{LpmError, LpmResult};
use std::path::{Path, PathBuf};

/// Discovers available templates from built-in and user locations
pub struct TemplateDiscovery;

impl TemplateDiscovery {
    /// Get the user templates directory
    pub fn user_templates_dir() -> LpmResult<PathBuf> {
        Ok(lpm_home()?.join("templates"))
    }

    /// Get built-in templates directory (in the binary/resources)
    /// For now, we'll use a directory relative to the binary or a default location
    pub fn builtin_templates_dir() -> PathBuf {
        // Check for templates in the source directory (for development)
        // In production, this could be embedded in the binary or installed separately
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                // Check if we're in a development build (target/debug or target/release)
                if exe_dir.to_string_lossy().contains("target") {
                    // Look for templates relative to workspace root
                    if let Some(workspace_root) = exe_dir
                        .ancestors()
                        .find(|p| p.join("Cargo.toml").exists() && p.join("src").exists())
                    {
                        return workspace_root.join("src").join("templates");
                    }
                }
            }
        }
        // Fallback: check current directory or default location
        PathBuf::from("templates")
    }

    /// List all available templates
    pub fn list_templates() -> LpmResult<Vec<TemplateInfo>> {
        let mut templates = Vec::new();

        // Check built-in templates
        let builtin_dir = Self::builtin_templates_dir();
        if builtin_dir.exists() {
            templates.extend(Self::discover_in_dir(
                &builtin_dir,
                TemplateSource::Builtin,
            )?);
        }

        // Check user templates
        if let Ok(user_dir) = Self::user_templates_dir() {
            if user_dir.exists() {
                templates.extend(Self::discover_in_dir(&user_dir, TemplateSource::User)?);
            }
        }

        Ok(templates)
    }

    fn discover_in_dir(dir: &Path, source: TemplateSource) -> LpmResult<Vec<TemplateInfo>> {
        let mut templates = Vec::new();

        if !dir.exists() || !dir.is_dir() {
            return Ok(templates);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Check if this directory contains a template.yaml
                let metadata_path = path.join("template.yaml");
                if metadata_path.exists() {
                    match TemplateMetadata::load(&path) {
                        Ok(metadata) => {
                            templates.push(TemplateInfo {
                                name: metadata.name.clone(),
                                description: metadata.description,
                                path,
                                source,
                            });
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to load template metadata from {}: {}",
                                path.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(templates)
    }

    /// Find a template by name
    pub fn find_template(name: &str) -> LpmResult<TemplateInfo> {
        // First check user templates (higher priority)
        if let Ok(user_dir) = Self::user_templates_dir() {
            if user_dir.exists() {
                let template_path = user_dir.join(name);
                if template_path.exists() && template_path.join("template.yaml").exists() {
                    let metadata = TemplateMetadata::load(&template_path)?;
                    return Ok(TemplateInfo {
                        name: metadata.name,
                        description: metadata.description,
                        path: template_path,
                        source: TemplateSource::User,
                    });
                }
            }
        }

        // Then check built-in templates
        let builtin_dir = Self::builtin_templates_dir();
        let template_path = builtin_dir.join(name);
        if template_path.exists() && template_path.join("template.yaml").exists() {
            let metadata = TemplateMetadata::load(&template_path)?;
            return Ok(TemplateInfo {
                name: metadata.name,
                description: metadata.description,
                path: template_path,
                source: TemplateSource::Builtin,
            });
        }

        Err(LpmError::Config(format!("Template '{}' not found", name)))
    }
}

#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub source: TemplateSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateSource {
    Builtin,
    User,
}
