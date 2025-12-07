use crate::core::{LpmError, LpmResult};
use crate::luarocks::rockspec::Rockspec;
use crate::package::manifest::PackageManifest;
use crate::core::path::{packages_metadata_dir, ensure_dir};
use std::fs;
use std::path::Path;

/// Convert a rockspec to package.yaml format and save it
pub fn convert_rockspec_to_manifest(
    rockspec: &Rockspec,
    project_root: &Path,
    package_name: &str,
) -> LpmResult<PackageManifest> {
    // Convert rockspec to manifest
    let manifest = rockspec.to_package_manifest();

    // Save to lua_modules/.lpm/packages/<name>/package.yaml
    let packages_dir = packages_metadata_dir(project_root);
    let package_metadata_dir = packages_dir.join(package_name);
    ensure_dir(&package_metadata_dir)?;

    let manifest_path = package_metadata_dir.join("package.yaml");
    let yaml_content = serde_yaml::to_string(&manifest)
        .map_err(|e| LpmError::Package(format!("Failed to serialize manifest: {}", e)))?;

    fs::write(&manifest_path, yaml_content)?;

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::luarocks::rockspec::{Rockspec, RockspecBuild, RockspecSource, InstallTable};
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_convert_rockspec() {
        let temp = TempDir::new().unwrap();
        let rockspec = Rockspec {
            package: "luasocket".to_string(),
            version: "3.0-1".to_string(),
            source: RockspecSource {
                url: "https://github.com/lunarmodules/luasocket/archive/v3.0.tar.gz".to_string(),
                tag: None,
                branch: None,
            },
            dependencies: vec!["lua >= 5.1".to_string()],
            build: RockspecBuild {
                build_type: "builtin".to_string(),
                modules: {
                    let mut m = HashMap::new();
                    m.insert("socket".to_string(), "src/socket.lua".to_string());
                    m
                },
                install: InstallTable::default(),
            },
            description: Some("Network support for Lua".to_string()),
            homepage: Some("https://github.com/lunarmodules/luasocket".to_string()),
            license: Some("MIT".to_string()),
            lua_version: Some(">=5.1".to_string()),
            binary_urls: HashMap::new(),
        };

        let manifest = convert_rockspec_to_manifest(&rockspec, temp.path(), "luasocket").unwrap();
        
        assert_eq!(manifest.name, "luasocket");
        assert_eq!(manifest.version, "3.0-1");
        assert_eq!(manifest.dependencies.len(), 1);
        assert!(manifest.dependencies.contains_key("lua"));
        
        // Verify file was created
        let manifest_path = temp.path()
            .join("lua_modules")
            .join(".lpm")
            .join("packages")
            .join("luasocket")
            .join("package.yaml");
        assert!(manifest_path.exists());
    }
}

