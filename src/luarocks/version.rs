use crate::core::version::Version;
use crate::core::{LpmError, LpmResult};

/// Normalize LuaRocks version format to SemVer
///
/// LuaRocks uses format like "3.0-1" where:
/// - "3.0" is the version
/// - "-1" is the rockspec revision
///
/// LPM converts this to "3.0.1" (SemVer format)
pub fn normalize_luarocks_version(luarocks_version: &str) -> LpmResult<Version> {
    // Split on '-' to separate version from revision
    let parts: Vec<&str> = luarocks_version.split('-').collect();

    if parts.is_empty() {
        return Err(LpmError::Version(format!(
            "Invalid LuaRocks version format: {}",
            luarocks_version
        )));
    }

    let version_str = parts[0];
    let revision = if parts.len() > 1 {
        parts[1].parse::<u64>().unwrap_or(0)
    } else {
        0
    };

    // Parse the version part
    let mut version = Version::parse(version_str)?;

    // If revision > 0, add it as patch version
    // If version already has patch, we increment it
    if revision > 0 {
        version.patch += revision;
    }

    Ok(version)
}

/// Convert SemVer back to LuaRocks format (for display)
pub fn to_luarocks_version(version: &Version) -> String {
    // Simple conversion: "3.0.1" -> "3.0-1"
    // This is a simplified approach - real conversion might be more complex
    if version.patch > 0 && version.patch < 10 {
        format!("{}.{}-{}", version.major, version.minor, version.patch)
    } else {
        format!("{}.{}", version.major, version.minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_luarocks_version() {
        let v = normalize_luarocks_version("3.0-1").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 1);

        let v = normalize_luarocks_version("1.13.1-1").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 13);
        assert_eq!(v.patch, 2); // 1 + 1

        let v = normalize_luarocks_version("3.0").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }
}
