use crate::cache::Cache;
use crate::config::Config;
use crate::core::LpmResult;
use crate::luarocks::client::LuaRocksClient;
use crate::luarocks::rockspec::Rockspec;
use crate::luarocks::search_api::SearchAPI;
use crate::package::lockfile::{LockedPackage, Lockfile};
use crate::package::manifest::PackageManifest;
use crate::resolver::DependencyResolver;
use std::collections::HashMap;
use std::path::Path;

/// Builder for creating lockfiles from manifests
pub struct LockfileBuilder {
    cache: Cache,
}

impl LockfileBuilder {
    pub fn new(cache: Cache) -> Self {
        Self { cache }
    }

    /// Generate a lockfile from a manifest
    ///
    /// This implementation:
    /// 1. Resolves all dependencies (using resolver)
    /// 2. Fetches rockspecs to get source URLs
    /// 3. Calculates checksums from cached source files
    /// 4. Records checksums in package.lock
    ///
    /// If `exclude_dev` is true, dev_dependencies are excluded (for production builds)
    pub async fn build_lockfile(
        &self,
        manifest: &PackageManifest,
        _project_root: &Path,
        exclude_dev: bool,
    ) -> LpmResult<Lockfile> {
        let mut lockfile = Lockfile::new();

        // Setup clients for fetching rockspecs
        let config = Config::load()?;
        let client = LuaRocksClient::new(&config, self.cache.clone());
        let search_api = SearchAPI::new();

        // Fetch manifest for resolver
        let luarocks_manifest = client.fetch_manifest().await?;
        let resolver = DependencyResolver::new(luarocks_manifest.clone());

        // Resolve all dependencies
        let resolved_versions = resolver.resolve(&manifest.dependencies).await?;
        let resolved_dev_versions = if !exclude_dev {
            resolver.resolve(&manifest.dev_dependencies).await?
        } else {
            HashMap::new()
        };

        // Use parallel downloads for better performance
        use crate::package::downloader::{DownloadTask, ParallelDownloader};
        let parallel_downloader = ParallelDownloader::new(client, Some(10));

        // Get source URLs from manifest for parallel downloads (already fetched above)

        // Create download tasks for all packages
        let mut download_tasks = Vec::new();
        for (name, version) in &resolved_versions {
            let version_str = version.to_string();
            let rockspec_url = search_api.get_rockspec_url(name, &version_str, None);

            // Try to get source URL from manifest
            let source_url = luarocks_manifest
                .get_package_versions(name)
                .and_then(|versions| {
                    versions
                        .iter()
                        .find(|pv| pv.version == version_str)
                        .and_then(|pv| pv.archive_url.as_ref())
                        .cloned()
                });

            download_tasks.push(DownloadTask {
                name: name.clone(),
                version: version_str,
                rockspec_url,
                source_url,
            });
        }

        if !exclude_dev {
            for (name, version) in &resolved_dev_versions {
                let version_str = version.to_string();
                let rockspec_url = search_api.get_rockspec_url(name, &version_str, None);

                // Try to get source URL from manifest
                let source_url =
                    luarocks_manifest
                        .get_package_versions(name)
                        .and_then(|versions| {
                            versions
                                .iter()
                                .find(|pv| pv.version == version_str)
                                .and_then(|pv| pv.archive_url.as_ref())
                                .cloned()
                        });

                download_tasks.push(DownloadTask {
                    name: name.clone(),
                    version: version_str,
                    rockspec_url,
                    source_url,
                });
            }
        }

        // Download all packages in parallel
        let download_results = parallel_downloader
            .download_packages(download_tasks, None)
            .await;

        // Build lockfile entries from download results
        for result in download_results {
            if let Some(error) = result.error {
                return Err(error);
            }

            // Calculate checksum from downloaded source
            let checksum = if let Some(ref source_path) = result.source_path {
                Cache::checksum(source_path)?
            } else {
                return Err(crate::core::LpmError::Package(format!(
                    "No source path for {}",
                    result.name
                )));
            };

            // Get file size
            let size = result
                .source_path
                .as_ref()
                .and_then(|p| std::fs::metadata(p).ok())
                .map(|m| m.len());

            // Parse dependencies from rockspec
            let mut dependencies = HashMap::new();
            for dep in &result.rockspec.dependencies {
                // Skip lua runtime dependencies
                if dep.trim().starts_with("lua")
                    && (dep.contains(">=")
                        || dep.contains(">")
                        || dep.contains("==")
                        || dep.contains("~>"))
                {
                    continue;
                }

                // Parse dependency string
                if let Some(pos) = dep.find(char::is_whitespace) {
                    let dep_name = dep[..pos].trim().to_string();
                    let dep_version = dep[pos..].trim().to_string();
                    dependencies.insert(dep_name, dep_version);
                } else {
                    dependencies.insert(dep.trim().to_string(), "*".to_string());
                }
            }

            let version = result.version.clone();
            let name = result.name.clone();
            let locked_package = crate::package::lockfile::LockedPackage {
                version: version.clone(),
                source: "luarocks".to_string(),
                rockspec_url: Some(search_api.get_rockspec_url(&name, &version, None)),
                source_url: result.rockspec.source.url.clone().into(),
                checksum,
                size,
                dependencies,
                build: None,
            };

            lockfile.add_package(name, locked_package);
        }

        Ok(lockfile)
    }

    /// Build a LockedPackage entry by fetching rockspec and calculating checksum
    async fn build_locked_package(
        &self,
        client: &LuaRocksClient,
        search_api: &SearchAPI,
        name: &str,
        version: &str,
    ) -> LpmResult<LockedPackage> {
        // Get rockspec URL and fetch it
        let rockspec_url = search_api.get_rockspec_url(name, version, None);
        let rockspec_content = client.download_rockspec(&rockspec_url).await?;
        let rockspec: Rockspec = client.parse_rockspec(&rockspec_content)?;

        // Download source to get it in cache (if not already there)
        let source_path = client.download_source(&rockspec.source.url).await?;

        // Calculate checksum from cached source
        let checksum = Cache::checksum(&source_path)?;

        // Get file size
        let size = std::fs::metadata(&source_path).ok().map(|m| m.len());

        // Parse dependencies from rockspec
        let mut dependencies = HashMap::new();
        for dep in &rockspec.dependencies {
            // Skip lua runtime dependencies
            if dep.trim().starts_with("lua")
                && (dep.contains(">=")
                    || dep.contains(">")
                    || dep.contains("==")
                    || dep.contains("~>"))
            {
                continue;
            }

            // Parse dependency string
            if let Some(pos) = dep.find(char::is_whitespace) {
                let dep_name = dep[..pos].trim().to_string();
                let dep_version = dep[pos..].trim().to_string();
                dependencies.insert(dep_name, dep_version);
            } else {
                dependencies.insert(dep.trim().to_string(), "*".to_string());
            }
        }

        Ok(LockedPackage {
            version: version.to_string(),
            source: "luarocks".to_string(),
            rockspec_url: Some(rockspec_url),
            source_url: Some(rockspec.source.url.clone()),
            checksum,
            size,
            dependencies,
            build: None,
        })
    }

    /// Update lockfile incrementally - only rebuild changed packages
    pub async fn update_lockfile(
        &self,
        existing: &Lockfile,
        manifest: &PackageManifest,
        _project_root: &Path,
        exclude_dev: bool,
    ) -> LpmResult<Lockfile> {
        let mut new_lockfile = Lockfile::new();

        // Setup clients
        let config = Config::load()?;
        let client = LuaRocksClient::new(&config, self.cache.clone());
        let search_api = SearchAPI::new();

        // Fetch manifest for resolver
        let luarocks_manifest = client.fetch_manifest().await?;
        let resolver = DependencyResolver::new(luarocks_manifest);

        // Resolve all dependencies
        let resolved_versions = resolver.resolve(&manifest.dependencies).await?;
        let resolved_dev_versions = if !exclude_dev {
            resolver.resolve(&manifest.dev_dependencies).await?
        } else {
            HashMap::new()
        };

        // Combine all dependencies
        let mut all_dependencies = resolved_versions.clone();
        if !exclude_dev {
            all_dependencies.extend(resolved_dev_versions.clone());
        }

        // Track which packages have been processed
        let mut processed = std::collections::HashSet::new();

        // Check each dependency - reuse from existing lockfile if version unchanged
        for (name, resolved_version) in &all_dependencies {
            let version_str = resolved_version.to_string();

            // Check if package exists in existing lockfile with same version
            if let Some(existing_pkg) = existing.get_package(name) {
                if existing_pkg.version == version_str {
                    // Version unchanged - reuse existing entry
                    new_lockfile.add_package(name.clone(), existing_pkg.clone());
                    processed.insert(name.clone());
                }
            }
        }

        // Rebuild packages that changed or are new (all_dependencies already includes transitive deps from resolver)
        for (package_name, resolved_version) in &all_dependencies {
            if processed.contains(package_name) {
                continue;
            }

            let version_str = resolved_version.to_string();

            // Build new lockfile entry
            let locked_package = self
                .build_locked_package(&client, &search_api, package_name, &version_str)
                .await?;

            new_lockfile.add_package(package_name.clone(), locked_package);
            processed.insert(package_name.clone());
        }

        Ok(new_lockfile)
    }
}
