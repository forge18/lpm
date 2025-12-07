use lpm::cache::Cache;
use lpm::config::Config;
use lpm::core::path::find_project_root;
use lpm::core::{LpmError, LpmResult};
use lpm::package::lockfile::Lockfile;
use lpm::package::verifier::PackageVerifier;
use std::env;

pub fn run() -> LpmResult<()> {
    let current_dir = env::current_dir()
        .map_err(|e| LpmError::Path(format!("Failed to get current directory: {}", e)))?;

    let project_root = find_project_root(&current_dir)?;

    // Load lockfile
    let lockfile = Lockfile::load(&project_root)?.ok_or_else(|| {
        LpmError::Package(
            "No package.lock found. Run 'lpm install' first to generate a lockfile.".to_string(),
        )
    })?;

    if lockfile.packages.is_empty() {
        println!("No packages to verify");
        return Ok(());
    }

    // Load config and create cache
    let config = Config::load()?;
    let cache = Cache::new(config.get_cache_dir()?)?;

    // Create verifier
    let verifier = PackageVerifier::new(cache);

    println!("Verifying {} package(s)...", lockfile.packages.len());

    // Verify all packages
    let result = verifier.verify_all(&lockfile, &project_root)?;

    // Display results
    if result.is_success() {
        println!("✓ All packages verified successfully");
        println!("  {} package(s) verified", result.successful.len());
    } else {
        println!("❌ Verification failed");
        println!("  {} package(s) verified", result.successful.len());
        println!("  {} package(s) failed", result.failed.len());

        for (package, error) in &result.failed {
            println!("  ❌ {}: {}", package, error);
        }

        return Err(LpmError::Package(format!(
            "Verification failed for {} package(s)",
            result.failed.len()
        )));
    }

    Ok(())
}
