use crate::core::{LpmError, LpmResult};
use crate::core::version::Version;
use crate::luarocks::manifest::Manifest;
use reqwest::Client;

/// Client for interacting with LuaRocks search and manifest APIs
pub struct SearchAPI {
    client: Client,
    base_url: String,
}

impl Default for SearchAPI {
    fn default() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://luarocks.org".to_string(),
        }
    }
}

impl SearchAPI {
    /// Create a new SearchAPI instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the latest version of a package by fetching the manifest
    pub async fn get_latest_version(&self, package_name: &str) -> LpmResult<String> {
        // Fetch manifest
        let manifest_url = format!("{}/manifests/luarocks/manifest?format=json", self.base_url);
        let response = self
            .client
            .get(&manifest_url)
            .send()
            .await
            .map_err(LpmError::Http)?;

        if !response.status().is_success() {
            return Err(LpmError::Package(format!(
                "Failed to fetch manifest: HTTP {}",
                response.status()
            )));
        }

        let content = response
            .text()
            .await
            .map_err(LpmError::Http)?;

        // Parse manifest
        let manifest = Manifest::parse_json(&content)?;

        // Get all versions for this package
        let versions = manifest
            .get_package_versions(package_name)
            .ok_or_else(|| LpmError::Package(format!("Package '{}' not found in manifest", package_name)))?;

        // Find latest version by parsing and comparing
        let latest = versions
            .iter()
            .max_by_key(|pv| {
                // Parse version string (handles LuaRocks format like "3.0-1")
                Version::parse(&pv.version).unwrap_or_else(|_| Version::new(0, 0, 0))
            })
            .ok_or_else(|| LpmError::Package(format!("No versions found for package '{}'", package_name)))?;

        Ok(latest.version.clone())
    }

    /// Construct rockspec URL using LuaRocks standard format
    /// Format: https://luarocks.org/manifests/{manifest}/{package}-{version}.rockspec
    pub fn get_rockspec_url(&self, package_name: &str, version: &str, manifest: Option<&str>) -> String {
        let manifest_name = manifest.unwrap_or("luarocks");
        let rockspec_name = format!("{}-{}.rockspec", package_name, version);
        format!("{}/manifests/{}/{}", self.base_url, manifest_name, rockspec_name)
    }

    /// Verify a rockspec URL exists
    pub async fn verify_rockspec_url(&self, url: &str) -> LpmResult<()> {
        let response = self.client.head(url).send().await.map_err(LpmError::Http)?;
        if !response.status().is_success() {
            return Err(LpmError::Package(format!("Rockspec not found: {}", url)));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_api_new() {
        let api = SearchAPI::new();
        assert_eq!(api.base_url, "https://luarocks.org");
    }

    #[test]
    fn test_search_api_default() {
        let api = SearchAPI::default();
        assert_eq!(api.base_url, "https://luarocks.org");
    }

    #[test]
    fn test_get_rockspec_url() {
        let api = SearchAPI::new();
        let url = api.get_rockspec_url("test-package", "1.0.0", None);
        assert_eq!(url, "https://luarocks.org/manifests/luarocks/test-package-1.0.0.rockspec");
    }

    #[test]
    fn test_get_rockspec_url_with_manifest() {
        let api = SearchAPI::new();
        let url = api.get_rockspec_url("test-package", "1.0.0", Some("custom"));
        assert_eq!(url, "https://luarocks.org/manifests/custom/test-package-1.0.0.rockspec");
    }
}

