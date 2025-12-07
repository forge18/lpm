use crate::build::targets::Target;
use crate::cache::Cache;
use crate::core::path::cache_dir;
use crate::core::{LpmError, LpmResult};
use crate::lua_version::detector::LuaVersion;
use std::fs;
use std::path::PathBuf;

/// Manages pre-built binary downloads for Rust-compiled Lua native modules
///
/// These are pre-compiled dynamic libraries (.so/.dylib/.dll) that were built
/// from Rust code and are part of Lua module packages. NOT standalone Rust libraries.
pub struct PrebuiltBinaryManager {
    cache: Cache,
}

impl PrebuiltBinaryManager {
    /// Create a new pre-built binary manager
    pub fn new() -> LpmResult<Self> {
        let cache = Cache::new(cache_dir()?)?;
        Ok(Self { cache })
    }

    /// Check if a pre-built native module binary is available for a package
    ///
    /// These are compiled Rust code as dynamic libraries (.so/.dylib/.dll)
    /// that are part of Lua module packages.
    ///
    /// This checks:
    /// 1. Local cache (already downloaded)
    /// 2. LuaRocks manifest for binary URLs (future: CDN/registry)
    pub fn has_prebuilt(
        &self,
        package: &str,
        version: &str,
        lua_version: &LuaVersion,
        target: &Target,
    ) -> bool {
        let lua_version_str = lua_version.major_minor();
        self.cache
            .has_rust_build(package, version, &lua_version_str, &target.triple)
    }

    /// Get the path to a pre-built binary if available
    pub fn get_prebuilt(
        &self,
        package: &str,
        version: &str,
        lua_version: &LuaVersion,
        target: &Target,
    ) -> Option<PathBuf> {
        let lua_version_str = lua_version.major_minor();
        self.cache
            .get_rust_build(package, version, &lua_version_str, &target.triple)
    }

    /// Download a pre-built native module binary from a URL
    ///
    /// Downloads a compiled Rust dynamic library (.so/.dylib/.dll) that is
    /// part of a Lua module package and stores it in the cache.
    pub async fn download_prebuilt(
        &self,
        package: &str,
        version: &str,
        lua_version: &LuaVersion,
        target: &Target,
        url: &str,
    ) -> LpmResult<PathBuf> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;

        let lua_version_str = lua_version.major_minor();
        let cache_path =
            self.cache
                .rust_build_path(package, version, &lua_version_str, &target.triple);

        // Ensure parent directory exists
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        eprintln!("Downloading pre-built binary from {}...", url);

        // Download the binary
        let response = reqwest::get(url).await.map_err(|e| {
            LpmError::Package(format!("Failed to download pre-built binary: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(LpmError::Package(format!(
                "Failed to download pre-built binary: HTTP {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| LpmError::Package(format!("Failed to read binary data: {}", e)))?;

        // Write to cache
        let mut file = File::create(&cache_path)
            .await
            .map_err(|e| LpmError::Cache(format!("Failed to create cache file: {}", e)))?;

        file.write_all(&bytes)
            .await
            .map_err(|e| LpmError::Cache(format!("Failed to write binary to cache: {}", e)))?;

        file.sync_all()
            .await
            .map_err(|e| LpmError::Cache(format!("Failed to sync cache file: {}", e)))?;

        eprintln!("✓ Downloaded pre-built binary: {}", cache_path.display());

        Ok(cache_path)
    }

    /// Find binary URL from rockspec's binary_urls table
    ///
    /// Looks for a binary URL matching the current Lua version and target.
    /// Format: `binary_urls = { ["5.4-x86_64-unknown-linux-gnu"] = "https://..." }`
    pub fn find_binary_url(
        rockspec_binary_urls: &std::collections::HashMap<String, String>,
        target: &Target,
        lua_version: &LuaVersion,
    ) -> Option<String> {
        // The key format is: "{lua_version}-{target_triple}"
        let key = format!("{}-{}", lua_version.major_minor(), target.triple);
        rockspec_binary_urls.get(&key).cloned()
    }

    /// Try to get or download a pre-built binary
    ///
    /// Returns the path to the binary if available, or None if not available
    pub async fn get_or_download(
        &self,
        package: &str,
        version: &str,
        lua_version: &LuaVersion,
        target: &Target,
        binary_url: Option<&str>,
    ) -> LpmResult<Option<PathBuf>> {
        // First, check if we already have it cached
        if let Some(cached) = self.get_prebuilt(package, version, lua_version, target) {
            return Ok(Some(cached));
        }

        // If a binary URL is provided, try to download it
        if let Some(url) = binary_url {
            match self
                .download_prebuilt(package, version, lua_version, target, url)
                .await
            {
                Ok(path) => Ok(Some(path)),
                Err(e) => {
                    eprintln!("Warning: Failed to download pre-built binary: {}", e);
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_prebuilt_manager() {
        let temp = TempDir::new().unwrap();
        let cache = Cache::new(temp.path().to_path_buf()).unwrap();
        let manager = PrebuiltBinaryManager { cache };

        let lua_version = LuaVersion::new(5, 4, 0);
        let target = Target::default_target();

        // Should not have pre-built binary for non-existent package
        assert!(!manager.has_prebuilt("test-package", "1.0.0", &lua_version, &target));
    }
}
