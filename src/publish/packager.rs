use crate::build::prebuilt::PrebuiltBinaryManager;
use crate::build::targets::Target;
use crate::core::{LpmError, LpmResult};
use crate::lua_version::detector::LuaVersionDetector;
use crate::package::manifest::PackageManifest;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Packages Lua modules for publishing to LuaRocks
pub struct PublishPackager {
    project_root: PathBuf,
    manifest: PackageManifest,
}

impl PublishPackager {
    /// Create a new publish packager
    pub fn new(project_root: &Path, manifest: PackageManifest) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            manifest,
        }
    }

    /// Package the module for publishing
    ///
    /// This creates a distribution archive containing:
    /// - All Lua source files
    /// - Pre-built Rust binaries (if available)
    /// - Generated rockspec
    pub fn package(&self, include_binaries: bool) -> LpmResult<PathBuf> {
        let dist_dir = self.project_root.join("dist");
        let package_name = format!("{}-{}", self.manifest.name, self.manifest.version);
        let package_dir = dist_dir.join(&package_name);

        // Clean and create package directory
        if package_dir.exists() {
            fs::remove_dir_all(&package_dir)?;
        }
        fs::create_dir_all(&package_dir)?;

        // Copy Lua source files
        self.copy_lua_files(&package_dir)?;

        // Copy Rust binaries if available and requested
        if include_binaries && self.manifest.build.is_some() {
            self.copy_rust_binaries(&package_dir)?;
        }

        // Create archive
        let archive_path = self.create_archive(&package_dir, &package_name)?;

        println!("✓ Packaged for publishing: {}", archive_path.display());

        Ok(archive_path)
    }

    /// Copy all Lua source files to the package directory
    fn copy_lua_files(&self, package_dir: &Path) -> LpmResult<()> {
        // Look for Lua files in common directories
        let lua_dirs = vec!["lua", "src", "lib", "."];

        for dir_name in lua_dirs {
            let source_dir = self.project_root.join(dir_name);
            if source_dir.exists() && source_dir.is_dir() {
                let dest_dir = package_dir.join(dir_name);
                self.copy_directory(&source_dir, &dest_dir, |path| {
                    path.extension().map(|e| e == "lua").unwrap_or(false)
                })?;
            }
        }

        // Also copy any .lua files in the root
        for entry in fs::read_dir(&self.project_root)? {
            let entry = entry
                .map_err(|e| LpmError::Path(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "lua").unwrap_or(false) {
                let dest = package_dir.join(path.file_name().unwrap());
                fs::copy(&path, &dest)?;
            }
        }

        Ok(())
    }

    /// Copy Rust-compiled native module binaries
    fn copy_rust_binaries(&self, package_dir: &Path) -> LpmResult<()> {
        let lua_version = LuaVersionDetector::detect()?;
        let prebuilt_manager = PrebuiltBinaryManager::new()?;
        let default_target = Target::default_target();

        // Try to get pre-built binary for current platform
        if let Some(binary_path) = prebuilt_manager.get_prebuilt(
            &self.manifest.name,
            &self.manifest.version,
            &lua_version,
            &default_target,
        ) {
            // Create lib directory for binaries
            let lib_dir = package_dir.join("lib");
            fs::create_dir_all(&lib_dir)?;

            let binary_name = binary_path
                .file_name()
                .ok_or_else(|| LpmError::Package("Invalid binary path".to_string()))?;
            let dest = lib_dir.join(binary_name);
            fs::copy(&binary_path, &dest)?;
            println!("  Included binary: {}", dest.display());
        } else {
            println!("  No pre-built binary found for current platform");
        }

        Ok(())
    }

    /// Copy directory with file filter
    fn copy_directory<F>(&self, source: &Path, dest: &Path, filter: F) -> LpmResult<()>
    where
        F: Fn(&Path) -> bool,
    {
        fs::create_dir_all(dest)?;

        for entry in WalkDir::new(source) {
            let entry = entry
                .map_err(|e| LpmError::Path(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            if path.is_file() && filter(path) {
                let rel_path = path
                    .strip_prefix(source)
                    .map_err(|e| LpmError::Path(format!("Failed to get relative path: {}", e)))?;
                let dest_path = dest.join(rel_path);

                // Create parent directories
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::copy(path, &dest_path)?;
            }
        }

        Ok(())
    }

    /// Create distribution archive (tar.gz or zip)
    fn create_archive(&self, package_dir: &Path, package_name: &str) -> LpmResult<PathBuf> {
        let dist_dir = package_dir.parent().unwrap();
        let archive_name = format!(
            "{}.{}",
            package_name,
            if cfg!(target_os = "windows") {
                "zip"
            } else {
                "tar.gz"
            }
        );
        let archive_path = dist_dir.join(&archive_name);

        if cfg!(target_os = "windows") {
            self.create_zip(package_dir, &archive_path)?;
        } else {
            self.create_tar_gz(package_dir, &archive_path)?;
        }

        Ok(archive_path)
    }

    /// Create zip archive (Windows)
    fn create_zip(&self, source_dir: &Path, archive_path: &Path) -> LpmResult<()> {
        use std::process::Command;

        let status = Command::new("zip")
            .arg("-r")
            .arg(archive_path)
            .arg(".")
            .current_dir(source_dir)
            .status()?;

        if !status.success() {
            return Err(LpmError::Package(
                "Failed to create zip archive. Install 'zip' command.".to_string(),
            ));
        }

        Ok(())
    }

    /// Create tar.gz archive (Unix)
    fn create_tar_gz(&self, source_dir: &Path, archive_path: &Path) -> LpmResult<()> {
        use std::process::Command;

        let status = Command::new("tar")
            .arg("-czf")
            .arg(archive_path)
            .arg("-C")
            .arg(source_dir)
            .arg(".")
            .status()?;

        if !status.success() {
            return Err(LpmError::Package(
                "Failed to create tar.gz archive. Install 'tar' command.".to_string(),
            ));
        }

        Ok(())
    }
}
