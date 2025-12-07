use crate::core::error::{LpmError, LpmResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Version constraint types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionConstraint {
    /// Exact version: "1.2.3"
    Exact(Version),
    /// Compatible version: "^1.2.3" (>=1.2.3 <2.0.0)
    Compatible(Version),
    /// Patch version: "~1.2.3" (>=1.2.3 <1.3.0)
    Patch(Version),
    /// Greater than or equal: ">=1.2.3"
    GreaterOrEqual(Version),
    /// Less than: "<2.0.0"
    LessThan(Version),
    /// Any patch version: "1.2.x"
    AnyPatch(Version),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl Version {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse a version string (e.g., "1.2.3" or "3.0-1" from LuaRocks)
    pub fn parse(s: &str) -> LpmResult<Self> {
        // Handle LuaRocks format: "3.0-1" -> "3.0.1"
        let normalized = s.replace('-', ".");
        let parts: Vec<&str> = normalized.split('.').collect();

        if parts.len() < 2 {
            return Err(LpmError::Version(format!("Invalid version format: {}", s)));
        }

        let major = parts[0]
            .parse()
            .map_err(|_| LpmError::Version(format!("Invalid major version: {}", s)))?;
        let minor = parts
            .get(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let patch = parts
            .get(2)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    /// Check if this version satisfies a constraint
    pub fn satisfies(&self, constraint: &VersionConstraint) -> bool {
        match constraint {
            VersionConstraint::Exact(v) => self == v,
            VersionConstraint::Compatible(v) => {
                self >= v && self.major == v.major && (self.major, self.minor, self.patch) < (v.major + 1, 0, 0)
            }
            VersionConstraint::Patch(v) => {
                self >= v && (self.major, self.minor) == (v.major, v.minor) && (self.major, self.minor, self.patch) < (v.major, v.minor + 1, 0)
            }
            VersionConstraint::GreaterOrEqual(v) => self >= v,
            VersionConstraint::LessThan(v) => self < v,
            VersionConstraint::AnyPatch(v) => {
                self.major == v.major && self.minor == v.minor
            }
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Parse a version constraint string
pub fn parse_constraint(s: &str) -> LpmResult<VersionConstraint> {
    let s = s.trim();

    if let Some(rest) = s.strip_prefix('^') {
        let version = Version::parse(rest)?;
        Ok(VersionConstraint::Compatible(version))
    } else if let Some(rest) = s.strip_prefix('~') {
        let version = Version::parse(rest)?;
        Ok(VersionConstraint::Patch(version))
    } else if let Some(rest) = s.strip_prefix(">=") {
        let version = Version::parse(rest)?;
        Ok(VersionConstraint::GreaterOrEqual(version))
    } else if let Some(rest) = s.strip_prefix('<') {
        let version = Version::parse(rest)?;
        Ok(VersionConstraint::LessThan(version))
    } else if let Some(base) = s.strip_suffix(".x") {
        let version = Version::parse(base)?;
        Ok(VersionConstraint::AnyPatch(version))
    } else {
        // Exact version
        let version = Version::parse(s)?;
        Ok(VersionConstraint::Exact(version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_parse_luarocks() {
        let v = Version::parse("3.0-1").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 1);
    }

    #[test]
    fn test_version_satisfies_exact() {
        let v = Version::parse("1.2.3").unwrap();
        let constraint = VersionConstraint::Exact(Version::parse("1.2.3").unwrap());
        assert!(v.satisfies(&constraint));
    }

    #[test]
    fn test_version_satisfies_compatible() {
        let v1 = Version::parse("1.2.3").unwrap();
        let v2 = Version::parse("1.3.0").unwrap();
        let v3 = Version::parse("2.0.0").unwrap();

        let constraint = VersionConstraint::Compatible(Version::parse("1.2.0").unwrap());
        assert!(v1.satisfies(&constraint));
        assert!(v2.satisfies(&constraint));
        assert!(!v3.satisfies(&constraint));
    }

    #[test]
    fn test_parse_constraint() {
        assert!(matches!(
            parse_constraint("^1.2.3").unwrap(),
            VersionConstraint::Compatible(_)
        ));
        assert!(matches!(
            parse_constraint("~1.2.3").unwrap(),
            VersionConstraint::Patch(_)
        ));
        assert!(matches!(
            parse_constraint(">=1.2.3").unwrap(),
            VersionConstraint::GreaterOrEqual(_)
        ));
        assert!(matches!(
            parse_constraint("1.2.3").unwrap(),
            VersionConstraint::Exact(_)
        ));
    }
}

