use crate::core::{LpmError, LpmResult};
use crate::lua_version::compatibility::PackageCompatibility;
use crate::lua_version::detector::LuaVersion;
use crate::luarocks::client::LuaRocksClient;
use crate::luarocks::manifest::Manifest;
use crate::luarocks::rockspec::Rockspec;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinSet;

/// Information about a package to download
#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub name: String,
    pub version: String,
    pub rockspec_url: String,
    pub source_url: Option<String>,
}

/// Result of a download operation
#[derive(Debug)]
pub struct DownloadResult {
    pub name: String,
    pub version: String,
    pub rockspec: Rockspec,
    pub source_path: Option<PathBuf>,
    pub error: Option<LpmError>,
}

/// Manages parallel/concurrent package downloads
pub struct ParallelDownloader {
    client: Arc<LuaRocksClient>,
    max_concurrent: usize,
}

impl ParallelDownloader {
    /// Create a new parallel downloader
    pub fn new(client: LuaRocksClient, max_concurrent: Option<usize>) -> Self {
        Self {
            client: Arc::new(client),
            max_concurrent: max_concurrent.unwrap_or(10), // Default to 10 concurrent downloads
        }
    }

    /// Download multiple packages in parallel
    pub async fn download_packages(
        &self,
        tasks: Vec<DownloadTask>,
        installed_lua: Option<&LuaVersion>,
    ) -> Vec<DownloadResult> {
        let mut results = Vec::new();
        let mut join_set = JoinSet::new();

        // Spawn download tasks with concurrency limit
        for task in tasks {
            if join_set.len() >= self.max_concurrent {
                // Wait for one task to complete before adding another
                if let Some(Ok(download_result)) = join_set.join_next().await {
                    results.push(download_result);
                }
            }

            let client = Arc::clone(&self.client);
            let task_clone = task.clone();
            let lua_version = installed_lua.cloned();
            join_set.spawn(async move {
                Self::download_single_package(client.as_ref(), task_clone, lua_version.as_ref()).await
            });
        }

        // Wait for all remaining tasks
        while let Some(result) = join_set.join_next().await {
            if let Ok(download_result) = result {
                results.push(download_result);
            }
        }

        results
    }

    /// Download a single package (used by parallel downloader)
    async fn download_single_package(
        client: &LuaRocksClient,
        task: DownloadTask,
        installed_lua: Option<&LuaVersion>,
    ) -> DownloadResult {
        let name = task.name.clone();
        let version = task.version.clone();

        // Download rockspec
        let rockspec_result = client.download_rockspec(&task.rockspec_url).await;
        let rockspec = match rockspec_result {
            Ok(content) => match client.parse_rockspec(&content) {
                Ok(r) => {
                    // Check Lua version compatibility if installed version is known
                    let rockspec = r;
                    if let Some(lua_version) = installed_lua {
                        let lua_version_str = rockspec.lua_version.as_deref().unwrap_or("unknown").to_string();
                        match PackageCompatibility::check_rockspec(lua_version, &rockspec) {
                            Ok(true) => {
                                // Compatible, continue
                            }
                            Ok(false) => {
                                // Incompatible - return error
                                return DownloadResult {
                                    name: name.clone(),
                                    version: version.clone(),
                                    rockspec,
                                    source_path: None,
                                    error: Some(LpmError::Version(format!(
                                        "Package '{}' version '{}' requires Lua {}, but installed version is {}",
                                        name, version,
                                        lua_version_str,
                                        lua_version.version_string()
                                    ))),
                                };
                            }
                            Err(e) => {
                                // Parse error, but continue (might be invalid constraint format)
                                eprintln!("Warning: Failed to parse Lua version constraint for {}: {}", name, e);
                            }
                        }
                    }
                    rockspec
                }
                Err(e) => {
                    use crate::luarocks::rockspec::{Rockspec, RockspecSource, RockspecBuild};
                    return DownloadResult {
                        name: name.clone(),
                        version: version.clone(),
                        rockspec: Rockspec {
                            package: name.clone(),
                            version: version.clone(),
                            source: RockspecSource {
                                url: String::new(),
                                tag: None,
                                branch: None,
                            },
                            dependencies: Vec::new(),
                            build: RockspecBuild {
                                build_type: String::new(),
                                modules: HashMap::new(),
                                install: crate::luarocks::rockspec::InstallTable::default(),
                            },
                            description: None,
                            homepage: None,
                            license: None,
                            lua_version: None,
                            binary_urls: HashMap::new(),
                        },
                        source_path: None,
                        error: Some(e),
                    };
                }
            },
            Err(e) => {
                return DownloadResult {
                    name: name.clone(),
                    version: version.clone(),
                    rockspec: Rockspec {
                        package: name.clone(),
                        version: version.clone(),
                        source: crate::luarocks::rockspec::RockspecSource {
                            url: String::new(),
                            tag: None,
                            branch: None,
                        },
                        dependencies: Vec::new(),
                        build: crate::luarocks::rockspec::RockspecBuild {
                            build_type: String::new(),
                            modules: HashMap::new(),
                            install: crate::luarocks::rockspec::InstallTable::default(),
                        },
                        description: None,
                        homepage: None,
                        license: None,
                        lua_version: None,
                        binary_urls: HashMap::new(),
                    },
                    source_path: None,
                    error: Some(e),
                };
            }
        };

        // Download source if URL is provided
        let source_path = if let Some(source_url) = &task.source_url {
            match client.download_source(source_url).await {
                Ok(path) => Some(path),
                Err(e) => {
                    return DownloadResult {
                        name: name.clone(),
                        version: version.clone(),
                        rockspec,
                        source_path: None,
                        error: Some(e),
                    };
                }
            }
        } else {
            None
        };

        DownloadResult {
            name,
            version,
            rockspec,
            source_path,
            error: None,
        }
    }

    /// Download packages with progress reporting
    pub async fn download_with_progress(
        &self,
        tasks: Vec<DownloadTask>,
        installed_lua: Option<&LuaVersion>,
    ) -> LpmResult<Vec<DownloadResult>> {
        let total = tasks.len();
        
        // Create progress bar
        let pb = ProgressBar::new(total as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} packages")
                .unwrap()
                .progress_chars("#>-")
        );

        // Download packages
        let results = self.download_packages(tasks, installed_lua).await;

        // Update progress bar and report results
        let mut error_count = 0;

        for result in &results {
            pb.inc(1);
            if result.error.is_none() {
                pb.println(format!("  ✓ {}", result.name));
            } else {
                error_count += 1;
                if let Some(ref error) = result.error {
                    pb.println(format!("  ✗ {} (error: {})", result.name, error));
                }
            }
        }

        pb.finish_with_message("Download complete");

        if error_count > 0 {
            return Err(LpmError::Package(format!(
                "Failed to download {} package(s)",
                error_count
            )));
        }

        Ok(results)
    }
}

/// Helper to create download tasks from manifest and resolved versions
pub fn create_download_tasks(
    manifest: &Manifest,
    resolved_versions: &HashMap<String, String>, // package name -> version string
) -> Vec<DownloadTask> {
    let mut tasks = Vec::new();

    for (name, version) in resolved_versions {
        if let Some(package_versions) = manifest.get_package_versions(name) {
            // Find the matching version
            if let Some(package_version) = package_versions
                .iter()
                .find(|pv| pv.version == *version)
            {
                tasks.push(DownloadTask {
                    name: name.clone(),
                    version: version.clone(),
                    rockspec_url: package_version.rockspec_url.clone(),
                    source_url: package_version.archive_url.clone(),
                });
            }
        }
    }

    tasks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::luarocks::manifest::{Manifest, PackageVersion};

    #[test]
    fn test_create_download_tasks() {
        let mut manifest = Manifest::default();
        let mut versions = Vec::new();
        versions.push(PackageVersion {
            version: "1.0.0".to_string(),
            rockspec_url: "https://example.com/test-1.0.0.rockspec".to_string(),
            archive_url: Some("https://example.com/test-1.0.0.tar.gz".to_string()),
        });
        manifest.packages.insert("test-package".to_string(), versions);
        
        let mut resolved = HashMap::new();
        resolved.insert("test-package".to_string(), "1.0.0".to_string());
        
        let tasks = create_download_tasks(&manifest, &resolved);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "test-package");
        assert_eq!(tasks[0].version, "1.0.0");
        assert!(tasks[0].source_url.is_some());
    }

    #[test]
    fn test_create_download_tasks_no_match() {
        let manifest = Manifest::default();
        let mut resolved = HashMap::new();
        resolved.insert("test-package".to_string(), "1.0.0".to_string());
        
        let tasks = create_download_tasks(&manifest, &resolved);
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn test_create_download_tasks_multiple_packages() {
        let mut manifest = Manifest::default();
        let mut versions1 = Vec::new();
        versions1.push(PackageVersion {
            version: "1.0.0".to_string(),
            rockspec_url: "https://example.com/test1-1.0.0.rockspec".to_string(),
            archive_url: Some("https://example.com/test1-1.0.0.tar.gz".to_string()),
        });
        manifest.packages.insert("test-package-1".to_string(), versions1);
        
        let mut versions2 = Vec::new();
        versions2.push(PackageVersion {
            version: "2.0.0".to_string(),
            rockspec_url: "https://example.com/test2-2.0.0.rockspec".to_string(),
            archive_url: Some("https://example.com/test2-2.0.0.tar.gz".to_string()),
        });
        manifest.packages.insert("test-package-2".to_string(), versions2);
        
        let mut resolved = HashMap::new();
        resolved.insert("test-package-1".to_string(), "1.0.0".to_string());
        resolved.insert("test-package-2".to_string(), "2.0.0".to_string());
        
        let tasks = create_download_tasks(&manifest, &resolved);
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn test_download_task_clone() {
        let task = DownloadTask {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            rockspec_url: "https://example.com/test.rockspec".to_string(),
            source_url: Some("https://example.com/test.tar.gz".to_string()),
        };
        let cloned = task.clone();
        assert_eq!(task.name, cloned.name);
        assert_eq!(task.version, cloned.version);
    }
}

