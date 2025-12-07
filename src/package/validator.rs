use crate::core::version::parse_constraint;
use crate::core::{LpmError, LpmResult};
use crate::package::manifest::PackageManifest;
use std::collections::HashSet;

/// Validates package.yaml schema and content
pub struct ManifestValidator;

impl ManifestValidator {
    /// Validate a manifest with comprehensive checks
    pub fn validate(manifest: &PackageManifest) -> LpmResult<()> {
        // Basic validation (already done in manifest.validate())
        manifest.validate()?;

        // Additional schema validation
        Self::validate_name(&manifest.name)?;
        Self::validate_version_format(&manifest.version)?;
        Self::validate_lua_version(&manifest.lua_version)?;
        Self::validate_dependencies(&manifest.dependencies)?;
        Self::validate_dev_dependencies(&manifest.dev_dependencies)?;
        Self::validate_build_config(&manifest.build)?;
        Self::validate_scripts(&manifest.scripts)?;

        Ok(())
    }

    fn validate_name(name: &str) -> LpmResult<()> {
        // Name should be valid identifier
        if name.is_empty() {
            return Err(LpmError::Package(
                "Package name cannot be empty".to_string(),
            ));
        }

        // Check for valid characters (alphanumeric, hyphen, underscore)
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(LpmError::Package(format!(
                "Package name '{}' contains invalid characters. Use only alphanumeric, hyphen, or underscore",
                name
            )));
        }

        // Name should not start with hyphen or underscore
        if name.starts_with('-') || name.starts_with('_') {
            return Err(LpmError::Package(format!(
                "Package name '{}' cannot start with '-' or '_'",
                name
            )));
        }

        Ok(())
    }

    fn validate_version_format(version: &str) -> LpmResult<()> {
        // Try to parse as version to validate format
        if version.is_empty() {
            return Err(LpmError::Package("Version cannot be empty".to_string()));
        }

        // Basic SemVer check (major.minor.patch)
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() > 3 {
            return Err(LpmError::Package(format!(
                "Invalid version format '{}'. Expected SemVer format (e.g., '1.2.3')",
                version
            )));
        }

        // Check each part is numeric
        for part in &parts {
            if !part.chars().all(|c| c.is_ascii_digit()) {
                return Err(LpmError::Package(format!(
                    "Invalid version format '{}'. Version parts must be numeric",
                    version
                )));
            }
        }

        Ok(())
    }

    fn validate_lua_version(lua_version: &str) -> LpmResult<()> {
        if lua_version.is_empty() {
            return Err(LpmError::Package("lua_version cannot be empty".to_string()));
        }

        // Check for valid constraint format
        // Allow: "5.4", ">=5.1", "5.1 || 5.3 || 5.4", etc.
        // For now, just check it's not empty and has reasonable length
        if lua_version.len() > 100 {
            return Err(LpmError::Package(
                "lua_version constraint is too long".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_dependencies(deps: &std::collections::HashMap<String, String>) -> LpmResult<()> {
        let mut seen = HashSet::new();

        for (name, version) in deps {
            // Check for duplicate dependencies
            if seen.contains(name) {
                return Err(LpmError::Package(format!(
                    "Duplicate dependency '{}' found",
                    name
                )));
            }
            seen.insert(name.clone());

            // Validate dependency name
            Self::validate_name(name)?;

            // Validate version constraint
            parse_constraint(version).map_err(|e| {
                LpmError::Package(format!(
                    "Invalid version constraint '{}' for dependency '{}': {}",
                    version, name, e
                ))
            })?;
        }

        Ok(())
    }

    fn validate_dev_dependencies(
        deps: &std::collections::HashMap<String, String>,
    ) -> LpmResult<()> {
        // Same validation as regular dependencies
        Self::validate_dependencies(deps)
    }

    fn validate_build_config(
        build: &Option<crate::package::manifest::BuildConfig>,
    ) -> LpmResult<()> {
        if let Some(build) = build {
            // Validate build type
            match build.build_type.as_str() {
                "rust" | "builtin" | "none" => {}
                _ => {
                    return Err(LpmError::Package(format!(
                        "Invalid build type '{}'. Supported types: rust, builtin, none",
                        build.build_type
                    )));
                }
            }

            // If rust build, validate manifest path and ensure modules are specified
            // Rust builds must produce native Lua modules, not standalone libraries
            if build.build_type == "rust" {
                if let Some(manifest) = &build.manifest {
                    if !manifest.ends_with("Cargo.toml") {
                        return Err(LpmError::Package(format!(
                            "Rust build manifest should be 'Cargo.toml', got '{}'",
                            manifest
                        )));
                    }
                }

                // Rust builds must specify modules (native Lua modules, not standalone libraries)
                if build.modules.is_empty() {
                    return Err(LpmError::Package(
                        "Rust build must specify 'modules' mapping to native Lua module paths. \
                        Rust code must be compiled as dynamic libraries (.so/.dylib/.dll) \
                        that are part of a Lua module package, not standalone Rust libraries."
                            .to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_scripts(scripts: &std::collections::HashMap<String, String>) -> LpmResult<()> {
        for (name, command) in scripts {
            if name.is_empty() {
                return Err(LpmError::Package("Script name cannot be empty".to_string()));
            }

            if command.is_empty() {
                return Err(LpmError::Package(format!(
                    "Script '{}' has no command",
                    name
                )));
            }

            // Check for reserved script names
            let reserved = [
                "install",
                "preinstall",
                "postinstall",
                "prepublish",
                "publish",
            ];
            if reserved.contains(&name.as_str()) {
                return Err(LpmError::Package(format!(
                    "Script name '{}' is reserved and cannot be used",
                    name
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name() {
        assert!(ManifestValidator::validate_name("valid-name").is_ok());
        assert!(ManifestValidator::validate_name("valid_name").is_ok());
        assert!(ManifestValidator::validate_name("valid123").is_ok());
        assert!(ManifestValidator::validate_name("").is_err());
        assert!(ManifestValidator::validate_name("-invalid").is_err());
        assert!(ManifestValidator::validate_name("invalid@name").is_err());
    }

    #[test]
    fn test_validate_version_format() {
        assert!(ManifestValidator::validate_version_format("1.2.3").is_ok());
        assert!(ManifestValidator::validate_version_format("1.2").is_ok());
        assert!(ManifestValidator::validate_version_format("1").is_ok());
        assert!(ManifestValidator::validate_version_format("").is_err());
        assert!(ManifestValidator::validate_version_format("1.2.3.4").is_err());
        assert!(ManifestValidator::validate_version_format("1.2.x").is_err());
    }
}
