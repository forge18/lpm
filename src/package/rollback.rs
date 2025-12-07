use crate::core::LpmResult;
use crate::package::lockfile::Lockfile;
use crate::package::manifest::PackageManifest;
use std::path::Path;

/// Manages rollback for failed installations
pub struct RollbackManager {
    backup_lockfile: Option<Lockfile>,
    backup_manifest: Option<PackageManifest>,
}

impl RollbackManager {
    /// Create a new rollback manager and backup current state
    pub fn new(project_root: &Path) -> LpmResult<Self> {
        // Backup lockfile if it exists
        let backup_lockfile = Lockfile::load(project_root)?;

        // Backup manifest
        let backup_manifest = PackageManifest::load(project_root).ok();

        Ok(Self {
            backup_lockfile,
            backup_manifest,
        })
    }

    /// Rollback to the previous state
    pub fn rollback(&self, project_root: &Path) -> LpmResult<()> {
        // Restore lockfile if we had a backup
        if let Some(ref lockfile) = self.backup_lockfile {
            lockfile.save(project_root)?;
            eprintln!("✓ Rolled back package.lock");
        }

        // Restore manifest if we had a backup
        if let Some(ref manifest) = self.backup_manifest {
            manifest.save(project_root)?;
            eprintln!("✓ Rolled back package.yaml");
        }

        Ok(())
    }

    /// Check if rollback is available
    pub fn has_backup(&self) -> bool {
        self.backup_lockfile.is_some() || self.backup_manifest.is_some()
    }
}

/// Execute a function with automatic rollback on error
pub fn with_rollback<F, T>(
    project_root: &Path,
    f: F,
) -> LpmResult<T>
where
    F: FnOnce() -> LpmResult<T>,
{
    // Create rollback manager
    let rollback = RollbackManager::new(project_root)?;

    // Execute the function
    match f() {
        Ok(result) => Ok(result),
        Err(e) => {
            // Attempt rollback
            if rollback.has_backup() {
                eprintln!("\n⚠️  Installation failed. Attempting rollback...");
                if let Err(rollback_err) = rollback.rollback(project_root) {
                    eprintln!("❌ Rollback failed: {}", rollback_err);
                } else {
                    eprintln!("✓ Rollback completed");
                }
            }
            Err(e)
        }
    }
}

/// Execute an async function with automatic rollback on error
pub async fn with_rollback_async<F, Fut, T>(
    project_root: &Path,
    f: F,
) -> LpmResult<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = LpmResult<T>>,
{
    // Create rollback manager
    let rollback = RollbackManager::new(project_root)?;

    // Execute the function
    match f().await {
        Ok(result) => Ok(result),
        Err(e) => {
            // Attempt rollback
            if rollback.has_backup() {
                eprintln!("\n⚠️  Installation failed. Attempting rollback...");
                if let Err(rollback_err) = rollback.rollback(project_root) {
                    eprintln!("❌ Rollback failed: {}", rollback_err);
                } else {
                    eprintln!("✓ Rollback completed");
                }
            }
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::package::manifest::PackageManifest;

    #[test]
    fn test_rollback_manager() {
        let temp = TempDir::new().unwrap();
        let manifest = PackageManifest::default("test".to_string());
        manifest.save(temp.path()).unwrap();

        let rollback = RollbackManager::new(temp.path()).unwrap();
        assert!(rollback.has_backup());
    }
}

