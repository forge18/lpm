use lpm_core::{LpmError, LpmResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Plugin configuration stored per-plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin name
    pub plugin_name: String,
    /// Configuration key-value pairs
    pub settings: std::collections::HashMap<String, serde_yaml::Value>,
}

impl PluginConfig {
    /// Load plugin configuration
    pub fn load(plugin_name: &str) -> LpmResult<Self> {
        let config_path = Self::config_path(plugin_name)?;

        if !config_path.exists() {
            // Return default config
            return Ok(PluginConfig {
                plugin_name: plugin_name.to_string(),
                settings: std::collections::HashMap::new(),
            });
        }

        let content = fs::read_to_string(&config_path)?;
        let config: PluginConfig = serde_yaml::from_str(&content)
            .map_err(|e| LpmError::Config(format!("Invalid plugin config: {}", e)))?;

        Ok(config)
    }

    /// Save plugin configuration
    pub fn save(&self) -> LpmResult<()> {
        let config_path = Self::config_path(&self.plugin_name)?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(self)
            .map_err(|e| LpmError::Config(format!("Failed to serialize config: {}", e)))?;
        fs::write(&config_path, content)?;

        Ok(())
    }

    /// Get configuration path for a plugin
    pub fn config_path(plugin_name: &str) -> LpmResult<PathBuf> {
        let lpm_home = lpm_core::core::path::lpm_home()?;
        Ok(lpm_home
            .join("plugins")
            .join(format!("{}.config.yaml", plugin_name)))
    }

    /// Get a setting value
    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.settings
            .get(key)
            .and_then(|v| serde_yaml::from_value(v.clone()).ok())
    }

    /// Set a setting value
    pub fn set<T>(&mut self, key: String, value: T) -> LpmResult<()>
    where
        T: Serialize,
    {
        let yaml_value = serde_yaml::to_value(value)
            .map_err(|e| LpmError::Config(format!("Failed to serialize value: {}", e)))?;
        self.settings.insert(key, yaml_value);
        Ok(())
    }
}
