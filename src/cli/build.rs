use lpm::build::builder::RustBuilder;
use lpm::build::targets::Target;
use lpm::core::path::find_project_root;
use lpm::core::{LpmError, LpmResult};
use lpm::package::manifest::PackageManifest;
use std::env;

pub fn run(target: Option<String>, all_targets: bool) -> LpmResult<()> {
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

    let builder = RustBuilder::new(&project_root, &manifest)?;

    if all_targets {
        // Build for all supported targets
        eprintln!("Building for all supported targets...");
        // build_all_targets is not async, but calls build which is async
        // We need to handle this differently
        let mut results = Vec::new();
        let rt = tokio::runtime::Runtime::new().unwrap();

        for target_triple in lpm::build::targets::SUPPORTED_TARGETS {
            let target = Target::new(target_triple)?;
            eprintln!("Building for target: {}", target.triple);

            match rt.block_on(builder.build(Some(&target))) {
                Ok(path) => {
                    results.push((target, path));
                    eprintln!("✓ Built successfully for {}", target_triple);
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to build for {}: {}", target_triple, e);
                }
            }
        }

        if results.is_empty() {
            return Err(LpmError::Package(
                "Failed to build for all targets".to_string(),
            ));
        }

        eprintln!("\n✓ Build complete for {} target(s):", results.len());
        for (target, path) in &results {
            eprintln!("  {} -> {}", target.triple, path.display());
        }
    } else {
        // Build for specific target or default
        let build_target = if let Some(triple) = target {
            Some(Target::new(&triple)?)
        } else {
            None
        };

        let target_display = build_target
            .as_ref()
            .map(|t| t.triple.as_str())
            .unwrap_or("default");
        eprintln!("Building for target: {}", target_display);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let output_path = rt.block_on(builder.build(build_target.as_ref()))?;
        eprintln!("✓ Build complete: {}", output_path.display());
    }

    Ok(())
}
