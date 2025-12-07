use lpm::build::targets::Target;
use lpm::core::{LpmError, LpmResult};
use lpm::core::path::find_project_root;
use lpm::package::manifest::PackageManifest;
use lpm::package::packager::BinaryPackager;
use std::env;

pub fn run(target: Option<String>) -> LpmResult<()> {
    let current_dir = env::current_dir()
        .map_err(|e| LpmError::Path(format!("Failed to get current directory: {}", e)))?;

    let project_root = find_project_root(&current_dir)?;
    let manifest = PackageManifest::load(&project_root)?;

    // Check if project has Rust build configuration
    if manifest.build.is_none() {
        return Err(LpmError::Package(
            "No build configuration found in package.yaml. Add a 'build' section with type: rust"
                .to_string(),
        ));
    }

    let packager = BinaryPackager::new(&project_root, manifest);

    if let Some(triple) = target {
        // Package for specific target
        let build_target = Target::new(&triple)?;
        eprintln!("Packaging for target: {}", build_target.triple);
        packager.package_target(&build_target)?;
    } else {
        // Package for all targets
        eprintln!("Packaging for all supported targets...");
        let results = packager.package_all_targets()?;
        
        eprintln!("\n✓ Packaging complete for {} target(s):", results.len());
        for (target, path) in &results {
            eprintln!("  {} -> {}", target.triple, path.display());
        }
    }

    Ok(())
}

