use lpm_core::LpmResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Template metadata stored in template.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub name: String,
    pub description: String,
    pub author: Option<String>,
    pub version: Option<String>,
    pub variables: Vec<TemplateVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub description: Option<String>,
    pub default: Option<String>,
    pub required: bool,
}

impl TemplateMetadata {
    pub fn load(template_dir: &Path) -> LpmResult<Self> {
        let metadata_path = template_dir.join("template.yaml");
        if !metadata_path.exists() {
            return Err(lpm_core::LpmError::Config(format!(
                "Template metadata not found: {}",
                metadata_path.display()
            )));
        }

        let content = std::fs::read_to_string(&metadata_path)?;
        let metadata: TemplateMetadata = serde_yaml::from_str(&content)?;
        Ok(metadata)
    }
}

