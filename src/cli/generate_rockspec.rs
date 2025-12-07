use lpm::core::path::find_project_root;
use lpm::core::{LpmError, LpmResult};
use lpm::package::manifest::PackageManifest;
use lpm::publish::rockspec_generator::RockspecGenerator;
use std::env;
use std::fs;

pub fn run() -> LpmResult<()> {
    let current_dir = env::current_dir()
        .map_err(|e| LpmError::Path(format!("Failed to get current directory: {}", e)))?;

    let project_root = find_project_root(&current_dir)?;
    let manifest = PackageManifest::load(&project_root)?;

    println!(
        "Generating rockspec for {}@{}...",
        manifest.name, manifest.version
    );

    let rockspec_content = RockspecGenerator::generate(&manifest)?;

    let luarocks_version = lpm::luarocks::version::to_luarocks_version(
        &lpm::core::version::Version::parse(&manifest.version)?,
    );
    let rockspec_filename = format!("{}-{}.rockspec", manifest.name, luarocks_version);
    let rockspec_path = project_root.join(&rockspec_filename);

    fs::write(&rockspec_path, rockspec_content)?;

    println!("✓ Generated rockspec: {}", rockspec_path.display());

    Ok(())
}
