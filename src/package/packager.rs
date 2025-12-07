use crate::build::builder::RustBuilder;
use crate::build::targets::Target;
use crate::core::{LpmError, LpmResult};
use crate::package::manifest::PackageManifest;
use std::fs;
use std::path::{Path, PathBuf};

/// Packages built Rust-compiled Lua native module binaries
///
/// These are dynamic libraries (.so/.dylib/.dll) compiled from Rust code
/// that are part of Lua module packages, not standalone Rust libraries.
pub struct BinaryPackager {
    project_root: PathBuf,
    manifest: PackageManifest,
}

impl BinaryPackager {
    /// Create a new packager
    pub fn new(project_root: &Path, manifest: PackageManifest) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            manifest,
        }
    }

    /// Package built binaries for a specific target
    pub fn package_target(&self, target: &Target) -> LpmResult<PathBuf> {
        // Build the extension for the target
        let builder = RustBuilder::new(&self.project_root, &self.manifest)?;
        let rt = tokio::runtime::Runtime::new().unwrap();
        let binary_path = rt.block_on(builder.build(Some(target)))?;

        // Create package directory
        let package_name = format!(
            "{}-{}-{}",
            self.manifest.name, self.manifest.version, target.triple
        );
        let package_dir = self.project_root.join("dist").join(&package_name);
        fs::create_dir_all(&package_dir)?;

        // Copy binary to package directory
        let binary_name = binary_path
            .file_name()
            .ok_or_else(|| LpmError::Package("Invalid binary path".to_string()))?;
        let dest_binary = package_dir.join(binary_name);
        fs::copy(&binary_path, &dest_binary)?;

        // Create package manifest
        self.create_package_manifest(&package_dir, target, &dest_binary)?;

        // Create archive (tar.gz or zip)
        let archive_path = self.create_archive(&package_dir, &package_name)?;

        println!("✓ Packaged: {}", archive_path.display());
        println!("  Binary: {}", dest_binary.display());
        println!("  Target: {}", target.triple);

        Ok(archive_path)
    }

    /// Create a package manifest file
    fn create_package_manifest(
        &self,
        package_dir: &Path,
        target: &Target,
        binary_path: &Path,
    ) -> LpmResult<()> {
        let manifest_content = format!(
            r#"# LPM Binary Package Manifest
name: {}
version: {}
target: {}
binary: {}
generated_at: "{}"
"#,
            self.manifest.name,
            self.manifest.version,
            target.triple,
            binary_path.file_name().unwrap().to_string_lossy(),
            chrono::Utc::now().to_rfc3339(),
        );

        let manifest_path = package_dir.join("package.yaml");
        fs::write(&manifest_path, manifest_content)?;

        Ok(())
    }

    /// Create an archive (tar.gz on Unix, zip on Windows)
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
            // Use zip on Windows
            self.create_zip(package_dir, &archive_path)?;
        } else {
            // Use tar.gz on Unix
            self.create_tar_gz(package_dir, &archive_path)?;
        }

        Ok(archive_path)
    }

    /// Create a zip archive (Windows)
    fn create_zip(&self, source_dir: &Path, archive_path: &Path) -> LpmResult<()> {
        use std::process::Command;

        // Try to use system zip command
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

    /// Create a tar.gz archive (Unix)
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

    /// Package binaries for all targets
    pub fn package_all_targets(&self) -> LpmResult<Vec<(Target, PathBuf)>> {
        let mut results = Vec::new();

        for target_triple in crate::build::targets::SUPPORTED_TARGETS {
            let target = Target::new(target_triple)?;
            eprintln!("Packaging for target: {}", target.triple);

            match self.package_target(&target) {
                Ok(path) => {
                    results.push((target, path));
                    eprintln!("✓ Packaged successfully for {}", target_triple);
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to package for {}: {}", target_triple, e);
                    // Continue with other targets
                }
            }
        }

        if results.is_empty() {
            return Err(LpmError::Package(
                "Failed to package for all targets".to_string(),
            ));
        }

        Ok(results)
    }
}
