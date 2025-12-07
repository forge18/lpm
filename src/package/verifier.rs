use crate::cache::Cache;
use crate::core::{LpmError, LpmResult};
use crate::package::lockfile::{LockedPackage, Lockfile};
use std::path::Path;

/// Verifies package checksums against the lockfile
pub struct PackageVerifier {
    cache: Cache,
}

impl PackageVerifier {
    pub fn new(cache: Cache) -> Self {
        Self { cache }
    }

    /// Verify all packages in the lockfile match their checksums
    pub fn verify_all(
        &self,
        lockfile: &Lockfile,
        project_root: &Path,
    ) -> LpmResult<VerificationResult> {
        let mut result = VerificationResult::new();

        for (name, package) in &lockfile.packages {
            match self.verify_package(name, package, project_root) {
                Ok(()) => result.add_success(name.clone()),
                Err(e) => result.add_failure(name.clone(), e.to_string()),
            }
        }

        Ok(result)
    }

    /// Verify a single package's checksum
    pub fn verify_package(
        &self,
        package_name: &str,
        package: &LockedPackage,
        _project_root: &Path,
    ) -> LpmResult<()> {
        // Extract checksum from lockfile (format: "sha256:...")
        let expected_checksum = &package.checksum;

        if !expected_checksum.starts_with("sha256:") {
            return Err(LpmError::Package(format!(
                "Invalid checksum format for '{}': expected 'sha256:...'",
                package_name
            )));
        }

        // Get the source file path from cache
        let source_path = if let Some(source_url) = &package.source_url {
            self.cache.source_path(source_url)
        } else {
            return Err(LpmError::Package(format!(
                "No source_url for package '{}' in lockfile",
                package_name
            )));
        };

        // Check if source file exists
        if !source_path.exists() {
            return Err(LpmError::Package(format!(
                "Source file not found for '{}': {}",
                package_name,
                source_path.display()
            )));
        }

        // Calculate actual checksum
        let actual_checksum = Cache::checksum(&source_path)?;

        // Compare checksums
        if actual_checksum != *expected_checksum {
            return Err(LpmError::Package(format!(
                "Checksum mismatch for '{}':\n  Expected: {}\n  Actual:   {}",
                package_name, expected_checksum, actual_checksum
            )));
        }

        Ok(())
    }

    /// Verify a package's checksum from a file path
    pub fn verify_file(&self, file_path: &Path, expected_checksum: &str) -> LpmResult<()> {
        if !expected_checksum.starts_with("sha256:") {
            return Err(LpmError::Package(
                "Invalid checksum format: expected 'sha256:...'".to_string(),
            ));
        }

        if !file_path.exists() {
            return Err(LpmError::Package(format!(
                "File not found: {}",
                file_path.display()
            )));
        }

        let actual_checksum = Cache::checksum(file_path)?;

        if actual_checksum != expected_checksum {
            return Err(LpmError::Package(format!(
                "Checksum mismatch:\n  Expected: {}\n  Actual:   {}",
                expected_checksum, actual_checksum
            )));
        }

        Ok(())
    }
}

/// Result of verification operation
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub successful: Vec<String>,
    pub failed: Vec<(String, String)>,
}

impl VerificationResult {
    pub fn new() -> Self {
        Self {
            successful: Vec::new(),
            failed: Vec::new(),
        }
    }

    pub fn add_success(&mut self, package: String) {
        self.successful.push(package);
    }

    pub fn add_failure(&mut self, package: String, error: String) {
        self.failed.push((package, error));
    }

    pub fn is_success(&self) -> bool {
        self.failed.is_empty()
    }

    pub fn total_verified(&self) -> usize {
        self.successful.len() + self.failed.len()
    }
}

impl Default for VerificationResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::Cache;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_verify_file_success() {
        let temp = TempDir::new().unwrap();
        let cache = Cache::new(temp.path().to_path_buf()).unwrap();
        let verifier = PackageVerifier::new(cache);

        let test_file = temp.path().join("test.txt");
        fs::write(&test_file, b"test data").unwrap();

        let checksum = Cache::checksum(&test_file).unwrap();
        verifier.verify_file(&test_file, &checksum).unwrap();
    }

    #[test]
    fn test_verify_file_mismatch() {
        let temp = TempDir::new().unwrap();
        let cache = Cache::new(temp.path().to_path_buf()).unwrap();
        let verifier = PackageVerifier::new(cache);

        let test_file = temp.path().join("test.txt");
        fs::write(&test_file, b"test data").unwrap();

        let wrong_checksum =
            "sha256:0000000000000000000000000000000000000000000000000000000000000000";
        let result = verifier.verify_file(&test_file, wrong_checksum);
        assert!(result.is_err());
    }
}
