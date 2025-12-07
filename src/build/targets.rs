use crate::core::{LpmError, LpmResult};

/// Supported cross-compilation targets
pub const SUPPORTED_TARGETS: &[&str] = &[
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-gnu",
    "x86_64-pc-windows-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
];

/// Represents a build target
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub triple: String,
}

impl Target {
    pub fn new(triple: &str) -> LpmResult<Self> {
        if !SUPPORTED_TARGETS.contains(&triple) {
            return Err(LpmError::Package(format!(
                "Unsupported target '{}'. Supported targets: {}",
                triple,
                SUPPORTED_TARGETS.join(", ")
            )));
        }
        Ok(Self {
            triple: triple.to_string(),
        })
    }

    /// Get the default target for the current platform
    pub fn default_target() -> Self {
        #[cfg(target_os = "linux")]
        {
            #[cfg(target_arch = "x86_64")]
            return Self {
                triple: "x86_64-unknown-linux-gnu".to_string(),
            };
            #[cfg(target_arch = "aarch64")]
            return Self {
                triple: "aarch64-unknown-linux-gnu".to_string(),
            };
        }
        #[cfg(target_os = "macos")]
        {
            #[cfg(target_arch = "x86_64")]
            return Self {
                triple: "x86_64-apple-darwin".to_string(),
            };
            #[cfg(target_arch = "aarch64")]
            return Self {
                triple: "aarch64-apple-darwin".to_string(),
            };
        }
        #[cfg(target_os = "windows")]
        {
            Self {
                triple: "x86_64-pc-windows-gnu".to_string(),
            }
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            // Fallback
            Self {
                triple: "x86_64-unknown-linux-gnu".to_string(),
            }
        }
    }

    /// Get the file extension for native modules on this target
    pub fn module_extension(&self) -> &'static str {
        if self.triple.contains("windows") {
            ".dll"
        } else if self.triple.contains("darwin") {
            ".dylib"
        } else {
            ".so"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_validation() {
        assert!(Target::new("x86_64-unknown-linux-gnu").is_ok());
        assert!(Target::new("invalid-target").is_err());
    }

    #[test]
    fn test_module_extension() {
        let linux = Target::new("x86_64-unknown-linux-gnu").unwrap();
        assert_eq!(linux.module_extension(), ".so");

        let windows = Target::new("x86_64-pc-windows-gnu").unwrap();
        assert_eq!(windows.module_extension(), ".dll");

        let macos = Target::new("x86_64-apple-darwin").unwrap();
        assert_eq!(macos.module_extension(), ".dylib");
    }
}

