use crate::core::{LpmError, LpmResult};
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tar::Archive;

/// Extracts package archives (tar.gz, zip) to temporary directories
pub struct PackageExtractor {
    dest_dir: PathBuf,
}

impl PackageExtractor {
    /// Create a new PackageExtractor
    pub fn new(dest_dir: PathBuf) -> Self {
        Self { dest_dir }
    }

    /// Extract an archive file
    /// Returns the path to the root directory of the extracted archive
    pub fn extract(&self, archive_path: &Path) -> LpmResult<PathBuf> {
        // Determine archive type from extension
        let extension = archive_path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| LpmError::Package("Unknown archive format".to_string()))?;

        let result = match extension {
            "gz" | "tgz" => self.extract_targz(archive_path),
            "zip" => self.extract_zip(archive_path),
            _ => Err(LpmError::Package(format!(
                "Unsupported format: {}",
                extension
            ))),
        };

        // Cleanup temp directory on error
        if result.is_err() {
            let temp_dir = self.dest_dir.join(format!(
                ".tmp-{}",
                archive_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
            ));
            let _ = fs::remove_dir_all(&temp_dir); // Ignore cleanup errors
        }

        result
    }

    fn extract_targz(&self, archive_path: &Path) -> LpmResult<PathBuf> {
        let file = File::open(archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        let temp_dir = self.dest_dir.join(format!(
            ".tmp-{}",
            archive_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
        ));

        // Clean up any existing temp dir
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        fs::create_dir_all(&temp_dir)?;

        archive.unpack(&temp_dir)?;

        // Find root directory (standardize: use first directory found)
        let mut entries = fs::read_dir(&temp_dir)?;
        let root = entries
            .find_map(|e| {
                let entry = e.ok()?;
                if entry.file_type().ok()?.is_dir() {
                    Some(entry.path())
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                // If no root dir, archive might have files at root - use temp_dir itself
                LpmError::Package(
                    "Archive has no root directory. Files at root level not supported.".to_string(),
                )
            })?;

        Ok(root)
    }

    fn extract_zip(&self, archive_path: &Path) -> LpmResult<PathBuf> {
        use zip::ZipArchive;

        let file = File::open(archive_path)?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| LpmError::Package(format!("Invalid zip: {}", e)))?;

        let temp_dir = self.dest_dir.join(format!(
            ".tmp-{}",
            archive_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
        ));

        // Clean up any existing temp dir
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        fs::create_dir_all(&temp_dir)?;

        archive
            .extract(&temp_dir)
            .map_err(|e| LpmError::Package(format!("Extract failed: {}", e)))?;

        // Find root directory (standardize: use first directory found)
        let mut entries = fs::read_dir(&temp_dir)?;
        let root = entries
            .find_map(|e| {
                let entry = e.ok()?;
                if entry.file_type().ok()?.is_dir() {
                    Some(entry.path())
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                LpmError::Package(
                    "Archive has no root directory. Files at root level not supported.".to_string(),
                )
            })?;

        Ok(root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_package_extractor_new() {
        let temp = TempDir::new().unwrap();
        let extractor = PackageExtractor::new(temp.path().to_path_buf());
        assert_eq!(extractor.dest_dir, temp.path());
    }

    #[test]
    fn test_package_extractor_unsupported_format() {
        let temp = TempDir::new().unwrap();
        let extractor = PackageExtractor::new(temp.path().to_path_buf());
        let test_file = temp.path().join("test.unknown");
        fs::write(&test_file, "test").unwrap();

        let result = extractor.extract(&test_file);
        assert!(result.is_err());
        match result {
            Err(LpmError::Package(msg)) => {
                assert!(msg.contains("Unsupported format") || msg.contains("unknown"));
            }
            _ => panic!("Expected Package error"),
        }
    }

    #[test]
    fn test_package_extractor_missing_file() {
        let temp = TempDir::new().unwrap();
        let extractor = PackageExtractor::new(temp.path().to_path_buf());
        let test_file = temp.path().join("nonexistent.tar.gz");

        let result = extractor.extract(&test_file);
        assert!(result.is_err());
    }
}
