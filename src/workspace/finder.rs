use crate::core::LpmResult;
use crate::workspace::Workspace;
use std::path::{Path, PathBuf};

/// Finds workspace root from a given directory
pub struct WorkspaceFinder;

impl WorkspaceFinder {
    /// Find workspace root by walking up the directory tree
    /// 
    /// Looks for workspace.yaml or package.yaml with workspace configuration
    pub fn find_workspace_root(start_dir: &Path) -> LpmResult<Option<PathBuf>> {
        let mut current = start_dir.to_path_buf();

        loop {
            // Check for workspace.yaml
            if current.join("workspace.yaml").exists() {
                return Ok(Some(current));
            }

            // Check for package.yaml with workspace config
            let package_yaml = current.join("package.yaml");
            if package_yaml.exists() && Workspace::is_workspace(&current) {
                return Ok(Some(current));
            }

            // Move to parent directory
            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => break, // Reached filesystem root
            }
        }

        Ok(None)
    }

    /// Find all package.yaml files in a workspace
    pub fn find_package_manifests(workspace_root: &Path) -> LpmResult<Vec<PathBuf>> {
        use walkdir::WalkDir;

        let mut manifests = Vec::new();

        for entry in WalkDir::new(workspace_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == "package.yaml" && entry.file_type().is_file() {
                manifests.push(entry.path().to_path_buf());
            }
        }

        Ok(manifests)
    }

    /// Check if a directory is within a workspace
    pub fn is_in_workspace(dir: &Path) -> bool {
        Self::find_workspace_root(dir)
            .ok()
            .flatten()
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_find_workspace_root() {
        let temp = TempDir::new().unwrap();
        let workspace_yaml = temp.path().join("workspace.yaml");
        fs::write(&workspace_yaml, "name: test\npackages: []\n").unwrap();

        let subdir = temp.path().join("subdir");
        fs::create_dir_all(&subdir).unwrap();

        let root = WorkspaceFinder::find_workspace_root(&subdir).unwrap();
        assert_eq!(root, Some(temp.path().to_path_buf()));
    }
}

