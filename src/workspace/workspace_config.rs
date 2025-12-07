use crate::core::{LpmError, LpmResult};
use crate::package::manifest::PackageManifest;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a workspace (monorepo) with multiple packages
pub struct Workspace {
    /// Root directory of the workspace
    pub root: PathBuf,
    /// Workspace configuration (from root package.yaml or workspace.yaml)
    pub config: WorkspaceConfig,
    /// All packages in the workspace, keyed by package name
    pub packages: HashMap<String, WorkspacePackage>,
}

/// Workspace configuration
#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    /// Workspace name
    pub name: String,
    /// Package directories (relative to workspace root)
    pub packages: Vec<String>,
}

/// A package within a workspace
#[derive(Debug, Clone)]
pub struct WorkspacePackage {
    /// Package name
    pub name: String,
    /// Path to package directory (relative to workspace root)
    pub path: PathBuf,
    /// Package manifest
    pub manifest: PackageManifest,
}

impl Workspace {
    /// Load a workspace from a directory
    pub fn load(workspace_root: &Path) -> LpmResult<Self> {
        // Check for workspace.yaml or package.yaml with workspace config
        let config = Self::load_config(workspace_root)?;
        
        // Find all packages in the workspace
        let packages = Self::find_packages(workspace_root, &config)?;
        
        Ok(Self {
            root: workspace_root.to_path_buf(),
            config,
            packages,
        })
    }

    /// Load workspace configuration
    fn load_config(workspace_root: &Path) -> LpmResult<WorkspaceConfig> {
        // Try workspace.yaml first
        let workspace_yaml = workspace_root.join("workspace.yaml");
        if workspace_yaml.exists() {
            return Self::load_workspace_yaml(&workspace_yaml);
        }

        // Try package.yaml with workspace field
        let package_yaml = workspace_root.join("package.yaml");
        if package_yaml.exists() {
            if let Ok(config) = Self::load_from_package_yaml(&package_yaml) {
                return Ok(config);
            }
        }

        // Default: auto-detect packages in common locations
        Ok(WorkspaceConfig {
            name: workspace_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("workspace")
                .to_string(),
            packages: vec!["packages/*".to_string(), "apps/*".to_string()],
        })
    }

    /// Load workspace.yaml
    fn load_workspace_yaml(path: &Path) -> LpmResult<WorkspaceConfig> {
        use std::fs;
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct WorkspaceYaml {
            name: String,
            packages: Vec<String>,
        }

        let content = fs::read_to_string(path)?;
        let workspace: WorkspaceYaml = serde_yaml::from_str(&content)
            .map_err(|e| LpmError::Package(format!("Failed to parse workspace.yaml: {}", e)))?;

        Ok(WorkspaceConfig {
            name: workspace.name,
            packages: workspace.packages,
        })
    }

    /// Load workspace config from package.yaml
    fn load_from_package_yaml(path: &Path) -> LpmResult<WorkspaceConfig> {
        use std::fs;
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct PackageYamlWithWorkspace {
            name: String,
            workspace: Option<WorkspaceYamlSection>,
        }

        #[derive(Deserialize)]
        struct WorkspaceYamlSection {
            packages: Vec<String>,
        }

        let content = fs::read_to_string(path)?;
        let package: PackageYamlWithWorkspace = serde_yaml::from_str(&content)
            .map_err(|e| LpmError::Package(format!("Failed to parse package.yaml: {}", e)))?;

        if let Some(workspace) = package.workspace {
            Ok(WorkspaceConfig {
                name: package.name,
                packages: workspace.packages,
            })
        } else {
            Err(LpmError::Package("No workspace section in package.yaml".to_string()))
        }
    }

    /// Find all packages in the workspace
    fn find_packages(
        workspace_root: &Path,
        config: &WorkspaceConfig,
    ) -> LpmResult<HashMap<String, WorkspacePackage>> {
        use walkdir::WalkDir;

        let mut packages = HashMap::new();

        // For each pattern, find matching directories
        for pattern in &config.packages {
            // Handle glob patterns like "packages/*" or "apps/*"
            if pattern.contains('*') {
                // Extract base path before wildcard
                let base_path = if let Some(star_pos) = pattern.find('*') {
                    pattern[..star_pos].trim_end_matches('/')
                } else {
                    pattern
                };

                let search_dir = workspace_root.join(base_path);
                if search_dir.exists() {
                    // Walk directory to find package.yaml files
                    for entry in WalkDir::new(&search_dir)
                        .max_depth(3) // Limit depth to avoid deep recursion
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        if entry.file_name() == "package.yaml" && entry.file_type().is_file() {
                            if let Some(package_dir) = entry.path().parent() {
                                match PackageManifest::load(package_dir) {
                                    Ok(manifest) => {
                                        let package = WorkspacePackage {
                                            name: manifest.name.clone(),
                                            path: package_dir
                                                .strip_prefix(workspace_root)
                                                .unwrap_or(package_dir)
                                                .to_path_buf(),
                                            manifest,
                                        };
                                        packages.insert(package.name.clone(), package);
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "Warning: Failed to load package at {}: {}",
                                            package_dir.display(),
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                // Direct path (no wildcard)
                let package_dir = workspace_root.join(pattern);
                if package_dir.exists() {
                    match PackageManifest::load(&package_dir) {
                        Ok(manifest) => {
                            let package = WorkspacePackage {
                                name: manifest.name.clone(),
                                path: package_dir
                                    .strip_prefix(workspace_root)
                                    .unwrap_or(&package_dir)
                                    .to_path_buf(),
                                manifest,
                            };
                            packages.insert(package.name.clone(), package);
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to load package at {}: {}",
                                package_dir.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(packages)
    }

    /// Get a package by name
    pub fn get_package(&self, name: &str) -> Option<&WorkspacePackage> {
        self.packages.get(name)
    }

    /// Get all package names
    pub fn package_names(&self) -> Vec<String> {
        self.packages.keys().cloned().collect()
    }

    /// Get shared dependencies across all workspace packages
    /// 
    /// Returns a map of package name to version constraint, where the same
    /// package appears in multiple workspace packages with potentially different constraints.
    pub fn shared_dependencies(&self) -> HashMap<String, Vec<(String, String)>> {
        let mut shared: HashMap<String, Vec<(String, String)>> = HashMap::new();

        for (package_name, workspace_pkg) in &self.packages {
            // Check regular dependencies
            for (dep_name, dep_version) in &workspace_pkg.manifest.dependencies {
                shared
                    .entry(dep_name.clone())
                    .or_default()
                    .push((package_name.clone(), dep_version.clone()));
            }

            // Check dev dependencies
            for (dep_name, dep_version) in &workspace_pkg.manifest.dev_dependencies {
                shared
                    .entry(dep_name.clone())
                    .or_default()
                    .push((package_name.clone(), format!("{} (dev)", dep_version)));
            }
        }

        // Filter to only dependencies used by multiple packages
        shared.retain(|_, usages| usages.len() > 1);
        shared
    }

    /// Check if a workspace is detected at the given path
    pub fn is_workspace(path: &Path) -> bool {
        path.join("workspace.yaml").exists()
            || (path.join("package.yaml").exists()
                && Self::load_from_package_yaml(&path.join("package.yaml")).is_ok())
    }
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            name: "workspace".to_string(),
            packages: vec!["packages/*".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_workspace_config_default() {
        let config = WorkspaceConfig::default();
        assert_eq!(config.name, "workspace");
        assert!(!config.packages.is_empty());
    }

    #[test]
    fn test_workspace_yaml_loading() {
        let temp = TempDir::new().unwrap();
        let workspace_yaml = temp.path().join("workspace.yaml");
        fs::write(
            &workspace_yaml,
            r#"
name: test-workspace
packages:
  - packages/*
  - apps/*
"#,
        )
        .unwrap();

        let config = Workspace::load_config(temp.path()).unwrap();
        assert_eq!(config.name, "test-workspace");
        assert_eq!(config.packages.len(), 2);
    }
}

