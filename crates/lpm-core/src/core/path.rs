use crate::core::error::{LpmError, LpmResult};
use std::path::{Path, PathBuf};

/// Get the LPM home directory
///
/// Platform-specific locations:
/// - Windows: %APPDATA%\lpm
/// - Linux: ~/.config/lpm
/// - macOS: ~/Library/Application Support/lpm
pub fn lpm_home() -> LpmResult<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| LpmError::Path("Could not determine config directory".to_string()))?;
    Ok(config_dir.join("lpm"))
}

/// Get the cache directory
///
/// Platform-specific locations:
/// - Windows: %LOCALAPPDATA%\lpm\cache
/// - Linux: ~/.cache/lpm
/// - macOS: ~/Library/Caches/lpm
pub fn cache_dir() -> LpmResult<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| LpmError::Path("Could not determine cache directory".to_string()))?;
    Ok(cache_dir.join("lpm"))
}

/// Get the config file path
///
/// Platform-specific locations:
/// - Windows: %APPDATA%\lpm\config.yaml
/// - Linux: ~/.config/lpm/config.yaml
/// - macOS: ~/Library/Application Support/lpm/config.yaml
pub fn config_file() -> LpmResult<PathBuf> {
    Ok(lpm_home()?.join("config.yaml"))
}

/// Get the credentials file path (deprecated - use CredentialStore instead)
///
/// Platform-specific locations:
/// - Windows: %APPDATA%\lpm\credentials
/// - Linux: ~/.config/lpm/credentials
/// - macOS: ~/Library/Application Support/lpm/credentials
///
/// Note: LPM uses OS keychain for credential storage. This path is kept
/// for compatibility but should not be used. If any credential files exist,
/// they should have 0600 permissions.
pub fn credentials_file() -> LpmResult<PathBuf> {
    Ok(lpm_home()?.join("credentials"))
}

/// Get the Lua modules directory for the current project (./lua_modules)
pub fn lua_modules_dir(project_root: &Path) -> PathBuf {
    project_root.join("lua_modules")
}

/// Get the LPM metadata directory (./lua_modules/.lpm)
pub fn lpm_metadata_dir(project_root: &Path) -> PathBuf {
    lua_modules_dir(project_root).join(".lpm")
}

/// Get the packages metadata directory (./lua_modules/.lpm/packages)
pub fn packages_metadata_dir(project_root: &Path) -> PathBuf {
    lpm_metadata_dir(project_root).join("packages")
}

/// Get the global installation directory
///
/// Platform-specific locations:
/// - Windows: %APPDATA%\lpm\global
/// - Linux: ~/.config/lpm/global
/// - macOS: ~/Library/Application Support/lpm/global
pub fn global_dir() -> LpmResult<PathBuf> {
    Ok(lpm_home()?.join("global"))
}

/// Get the global Lua modules directory
pub fn global_lua_modules_dir() -> LpmResult<PathBuf> {
    Ok(global_dir()?.join("lua_modules"))
}

/// Get the global bin directory (for executables)
///
/// This is the same as ~/.lpm/bin/ (where Lua wrappers are)
pub fn global_bin_dir() -> LpmResult<PathBuf> {
    Ok(lpm_home()?.join("bin"))
}

/// Get the global packages metadata directory
pub fn global_packages_metadata_dir() -> LpmResult<PathBuf> {
    Ok(global_dir()?.join(".lpm").join("packages"))
}

/// Find the project root by looking for package.yaml or workspace.yaml
///
/// Checks for workspace.yaml first, then falls back to package.yaml.
/// Workspace detection is done by checking for workspace.yaml file or
/// package.yaml with workspace configuration.
pub fn find_project_root(start: &Path) -> LpmResult<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        // Check for workspace.yaml first
        let workspace_yaml = current.join("workspace.yaml");
        if workspace_yaml.exists() {
            return Ok(current);
        }

        // Check for package.yaml (which may contain workspace config)
        let package_yaml = current.join("package.yaml");
        if package_yaml.exists() {
            // Check if this package.yaml has workspace configuration
            // by looking for a "workspace" key (simple heuristic)
            if let Ok(content) = std::fs::read_to_string(&package_yaml) {
                if content.contains("workspace:") || content.contains("workspaces:") {
                    return Ok(current);
                }
            }
            // Regular package.yaml found
            return Ok(current);
        }

        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            return Err(LpmError::Path(
                "Could not find package.yaml or workspace.yaml in current directory or parents"
                    .to_string(),
            ));
        }
    }
}

/// Check if we're in an LPM project (package.yaml exists)
pub fn is_project_root(dir: &Path) -> bool {
    dir.join("package.yaml").exists()
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_dir(path: &Path) -> LpmResult<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Normalize a path for cross-platform compatibility
pub fn normalize_path(path: &Path) -> PathBuf {
    // Convert to string and back to handle path separators
    path.components().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_project_root() {
        let temp = TempDir::new().unwrap();
        let project_dir = temp.path().join("project");
        fs::create_dir_all(&project_dir).unwrap();
        fs::write(project_dir.join("package.yaml"), "name: test\n").unwrap();

        let found = find_project_root(&project_dir.join("subdir")).unwrap();
        assert_eq!(found, project_dir);
    }

    #[test]
    fn test_ensure_dir() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("test_dir");

        ensure_dir(&dir).unwrap();
        assert!(dir.exists());
        assert!(dir.is_dir());
    }
}
