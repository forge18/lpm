use lpm::core::LpmResult;
use lpm::core::path::find_project_root;
use lpm::package::manifest::PackageManifest;
use lpm::path_setup::{LuaRunner, PathSetup, RunOptions};
use std::env;

pub fn run(script_name: String) -> LpmResult<()> {
    let current_dir = env::current_dir()?;
    let project_root = find_project_root(&current_dir)?;

    // Load manifest to get scripts
    let manifest = PackageManifest::load(&project_root)?;

    // Find the script
    let script_command = manifest
        .scripts
        .get(&script_name)
        .ok_or_else(|| {
            lpm::core::LpmError::Package(format!(
                "Script '{}' not found in package.yaml",
                script_name
            ))
        })?;

    // Ensure loader is installed (sets up package.path automatically)
    PathSetup::install_loader(&project_root)?;

    // Parse the script command (e.g., "lua src/main.lua" or "luajit -e 'print(1)'")
    let parts: Vec<&str> = script_command.split_whitespace().collect();
    if parts.is_empty() {
        return Err(lpm::core::LpmError::Package(format!(
            "Script '{}' has no command",
            script_name
        )));
    }

    // Execute the command with proper path setup
    let exit_code = LuaRunner::exec_command(script_command, RunOptions::default())?;
    std::process::exit(exit_code);
}
