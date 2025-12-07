use crate::core::{LpmError, LpmResult};
use crate::core::path::{cache_dir, ensure_dir};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Package cache manager
#[derive(Clone)]
pub struct Cache {
    root: PathBuf,
}

impl Cache {
    /// Create a new cache instance
    pub fn new(cache_root: PathBuf) -> LpmResult<Self> {
        ensure_dir(&cache_root)?;
        Ok(Self { root: cache_root })
    }

    /// Get the default cache directory
    pub fn default_cache() -> LpmResult<Self> {
        Self::new(cache_dir()?)
    }

    /// Get the LuaRocks cache directory
    pub fn luarocks_dir(&self) -> PathBuf {
        self.root.join("luarocks")
    }

    /// Get the rockspecs cache directory
    pub fn rockspecs_dir(&self) -> PathBuf {
        self.luarocks_dir().join("rockspecs")
    }

    /// Get the sources cache directory
    pub fn sources_dir(&self) -> PathBuf {
        self.luarocks_dir().join("sources")
    }

    /// Get the Rust builds cache directory
    pub fn rust_builds_dir(&self) -> PathBuf {
        self.root.join("rust-builds")
    }

    /// Initialize cache directory structure
    pub fn init(&self) -> LpmResult<()> {
        ensure_dir(&self.luarocks_dir())?;
        ensure_dir(&self.rockspecs_dir())?;
        ensure_dir(&self.sources_dir())?;
        ensure_dir(&self.rust_builds_dir())?;
        Ok(())
    }

    /// Get the cached path for a rockspec file
    pub fn rockspec_path(&self, package: &str, version: &str) -> PathBuf {
        let filename = format!("{}-{}.rockspec", package, version);
        self.rockspecs_dir().join(filename)
    }

    /// Get the cached path for a source archive
    pub fn source_path(&self, url: &str) -> PathBuf {
        // Use URL hash as filename to avoid path issues
        let hash = Self::url_hash(url);
        let extension = Path::new(url)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("tar.gz");
        self.sources_dir().join(format!("{}.{}", hash, extension))
    }

    /// Check if a file exists in cache
    pub fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    /// Read a file from cache
    pub fn read(&self, path: &Path) -> LpmResult<Vec<u8>> {
        fs::read(path).map_err(|e| {
            LpmError::Cache(format!("Failed to read from cache: {}: {}", path.display(), e))
        })
    }

    /// Write a file to cache
    pub fn write(&self, path: &Path, data: &[u8]) -> LpmResult<()> {
        if let Some(parent) = path.parent() {
            ensure_dir(parent)?;
        }
        let mut file = fs::File::create(path)
            .map_err(|e| LpmError::Cache(format!("Failed to create cache file: {}: {}", path.display(), e)))?;
        file.write_all(data)
            .map_err(|e| LpmError::Cache(format!("Failed to write to cache: {}: {}", path.display(), e)))?;
        Ok(())
    }

    /// Calculate SHA-256 checksum of a file
    pub fn checksum(path: &Path) -> LpmResult<String> {
        let data = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash = hasher.finalize();
        Ok(format!("sha256:{}", hex::encode(hash)))
    }

    /// Hash a URL for use as a filename
    fn url_hash(url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let hash = hasher.finalize();
        hex::encode(&hash[..16]) // Use first 16 bytes for shorter filename
    }

    /// Get the cached path for a Rust build artifact
    /// Get the cache path for a Rust build, including Lua version
    /// 
    /// Structure: rust_builds/{package}/{version}/{lua_version}/{target}/
    pub fn rust_build_path(&self, package: &str, version: &str, lua_version: &str, target: &str) -> PathBuf {
        let dir = self.rust_builds_dir()
            .join(package)
            .join(version)
            .join(lua_version)
            .join(target);
        // Determine library extension based on target
        let extension = if target.contains("windows") {
            "dll"
        } else if target.contains("darwin") || target.contains("apple") {
            "dylib"
        } else {
            "so"
        };
        dir.join(format!("lib{}.{}", package.replace('-', "_"), extension))
    }

    /// Check if a Rust build artifact is cached
    pub fn has_rust_build(&self, package: &str, version: &str, lua_version: &str, target: &str) -> bool {
        self.exists(&self.rust_build_path(package, version, lua_version, target))
    }

    /// Store a Rust build artifact in cache
    /// 
    /// Caches are stored per Lua version to support multiple Lua installations
    pub fn store_rust_build(
        &self,
        package: &str,
        version: &str,
        lua_version: &str,
        target: &str,
        artifact_path: &Path,
    ) -> LpmResult<PathBuf> {
        let cache_path = self.rust_build_path(package, version, lua_version, target);
        
        // Ensure parent directory exists
        if let Some(parent) = cache_path.parent() {
            ensure_dir(parent)?;
        }

        // Copy artifact to cache
        fs::copy(artifact_path, &cache_path)
            .map_err(|e| LpmError::Cache(format!("Failed to copy build artifact to cache: {}", e)))?;

        Ok(cache_path)
    }

    /// Get cached Rust build artifact path
    pub fn get_rust_build(&self, package: &str, version: &str, lua_version: &str, target: &str) -> Option<PathBuf> {
        let path = self.rust_build_path(package, version, lua_version, target);
        if self.exists(&path) {
            Some(path)
        } else {
            None
        }
    }

    /// Clean old cache entries based on age and size
    pub fn clean(&self, max_age_days: u64, max_size_mb: u64) -> LpmResult<CacheCleanResult> {
        use std::time::{SystemTime, Duration};

        let max_age = Duration::from_secs(max_age_days * 24 * 60 * 60);
        let max_size_bytes = max_size_mb * 1024 * 1024;
        let now = SystemTime::now();

        let mut result = CacheCleanResult {
            files_removed: 0,
            bytes_freed: 0,
        };

        // Clean rockspecs
        result += self.clean_directory(&self.rockspecs_dir(), &now, max_age, max_size_bytes)?;

        // Clean sources
        result += self.clean_directory(&self.sources_dir(), &now, max_age, max_size_bytes)?;

        // Clean Rust builds
        result += self.clean_directory(&self.rust_builds_dir(), &now, max_age, max_size_bytes)?;

        Ok(result)
    }

    /// Clean a directory based on age and total size
    fn clean_directory(
        &self,
        dir: &Path,
        now: &SystemTime,
        max_age: Duration,
        max_size_bytes: u64,
    ) -> LpmResult<CacheCleanResult> {
        use walkdir::WalkDir;

        if !dir.exists() {
            return Ok(CacheCleanResult::default());
        }

        let mut files: Vec<(PathBuf, SystemTime, u64)> = Vec::new();
        let mut total_size = 0u64;

        // Collect all files with metadata
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let size = metadata.len();
                        files.push((entry.path().to_path_buf(), modified, size));
                        total_size += size;
                    }
                }
            }
        }

        // Sort by modification time (oldest first)
        files.sort_by_key(|(_, modified, _)| *modified);

        let mut result = CacheCleanResult::default();

        // Remove files older than max_age
        for (path, modified, size) in &files {
            if let Ok(age) = now.duration_since(*modified) {
                if age > max_age {
                    if let Err(e) = fs::remove_file(path) {
                        eprintln!("Warning: Failed to remove old cache file {}: {}", path.display(), e);
                    } else {
                        result.files_removed += 1;
                        result.bytes_freed += size;
                        total_size -= size;
                    }
                }
            }
        }

        // If still over size limit, remove oldest files
        if total_size > max_size_bytes {
            let target_size = max_size_bytes;
            for (path, _, size) in &files {
                if total_size <= target_size {
                    break;
                }
                if path.exists() {
                    if let Err(e) = fs::remove_file(path) {
                        eprintln!("Warning: Failed to remove cache file {}: {}", path.display(), e);
                    } else {
                        result.files_removed += 1;
                        result.bytes_freed += size;
                        total_size -= size;
                    }
                }
            }
        }

        Ok(result)
    }
}

/// Result of cache cleaning operation
#[derive(Debug, Default)]
pub struct CacheCleanResult {
    pub files_removed: usize,
    pub bytes_freed: u64,
}

impl std::ops::AddAssign for CacheCleanResult {
    fn add_assign(&mut self, other: Self) {
        self.files_removed += other.files_removed;
        self.bytes_freed += other.bytes_freed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_init() {
        let temp = TempDir::new().unwrap();
        let cache = Cache::new(temp.path().to_path_buf()).unwrap();
        cache.init().unwrap();

        assert!(cache.luarocks_dir().exists());
        assert!(cache.rockspecs_dir().exists());
        assert!(cache.sources_dir().exists());
        assert!(cache.rust_builds_dir().exists());
    }

    #[test]
    fn test_cache_read_write() {
        let temp = TempDir::new().unwrap();
        let cache = Cache::new(temp.path().to_path_buf()).unwrap();
        cache.init().unwrap();

        let test_path = cache.rockspecs_dir().join("test.rockspec");
        let data = b"test data";

        cache.write(&test_path, data).unwrap();
        assert!(cache.exists(&test_path));

        let read_data = cache.read(&test_path).unwrap();
        assert_eq!(read_data, data);
    }
}
