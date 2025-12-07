use crate::core::{LpmError, LpmResult};
use crate::lua_version::constraint::parse_lua_version_constraint;
use crate::lua_version::detector::LuaVersion;
use crate::luarocks::rockspec::Rockspec;

/// Checks package compatibility with Lua versions
pub struct PackageCompatibility;

impl PackageCompatibility {
    /// Check if a package is compatible with the installed Lua version
    pub fn check_package(
        installed_version: &LuaVersion,
        package_lua_version: Option<&str>,
    ) -> LpmResult<bool> {
        let package_constraint = if let Some(lua_version) = package_lua_version {
            parse_lua_version_constraint(lua_version)?
        } else {
            // No constraint specified, assume compatible
            return Ok(true);
        };

        Ok(package_constraint.matches(installed_version))
    }

    /// Check if a rockspec is compatible with the installed Lua version
    pub fn check_rockspec(installed_version: &LuaVersion, rockspec: &Rockspec) -> LpmResult<bool> {
        Self::check_package(installed_version, rockspec.lua_version.as_deref())
    }

    /// Validate that the project's lua_version constraint matches the installed version
    pub fn validate_project_constraint(
        installed_version: &LuaVersion,
        project_constraint: &str,
    ) -> LpmResult<()> {
        let constraint = parse_lua_version_constraint(project_constraint)?;

        if !constraint.matches(installed_version) {
            return Err(LpmError::Version(format!(
                "Installed Lua version {} does not satisfy project requirement '{}'",
                installed_version, project_constraint
            )));
        }

        Ok(())
    }

    /// Filter packages by Lua version compatibility
    pub fn filter_compatible_packages(
        installed_version: &LuaVersion,
        packages: &[(String, Option<String>)], // (name, lua_version)
    ) -> Vec<String> {
        packages
            .iter()
            .filter_map(|(name, lua_version)| {
                match Self::check_package(installed_version, lua_version.as_deref()) {
                    Ok(true) => Some(name.clone()),
                    Ok(false) => None,
                    Err(_) => None, // Skip on parse error
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_package_exact() {
        let installed = LuaVersion::new(5, 4, 0);
        assert!(PackageCompatibility::check_package(&installed, Some("5.4")).unwrap());
        assert!(!PackageCompatibility::check_package(&installed, Some("5.3")).unwrap());
    }

    #[test]
    fn test_check_package_range() {
        let installed = LuaVersion::new(5, 4, 0);
        assert!(PackageCompatibility::check_package(&installed, Some(">=5.1")).unwrap());
        assert!(!PackageCompatibility::check_package(&installed, Some("<5.3")).unwrap());
    }

    #[test]
    fn test_check_package_multiple() {
        let installed = LuaVersion::new(5, 3, 0);
        assert!(
            PackageCompatibility::check_package(&installed, Some("5.1 || 5.3 || 5.4")).unwrap()
        );
    }
}
