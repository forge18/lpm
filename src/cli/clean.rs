use lpm::core::{LpmError, LpmResult};
use lpm::core::path::{find_project_root, lua_modules_dir};
use std::env;
use std::fs;

pub fn run() -> LpmResult<()> {
    let current_dir = env::current_dir()
        .map_err(|e| LpmError::Path(format!("Failed to get current directory: {}", e)))?;

    let project_root = find_project_root(&current_dir)?;
    let lua_modules = lua_modules_dir(&project_root);

    if !lua_modules.exists() {
        println!("lua_modules directory does not exist. Nothing to clean.");
        return Ok(());
    }

    println!("Cleaning lua_modules directory...");

    // Count packages before cleaning
    let package_count = count_packages(&lua_modules)?;

    // Remove lua_modules directory
    fs::remove_dir_all(&lua_modules)?;

    println!("✓ Cleaned {} package(s)", package_count);
    println!("  Removed: {}", lua_modules.display());

    Ok(())
}

fn count_packages(lua_modules: &std::path::Path) -> LpmResult<usize> {
    let mut count = 0;
    
    if lua_modules.exists() {
        for entry in fs::read_dir(lua_modules)? {
            let entry = entry?;
            let path = entry.path();
            
            // Skip .lpm metadata directory
            if path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n == ".lpm")
                .unwrap_or(false)
            {
                continue;
            }
            
            if path.is_dir() {
                count += 1;
            }
        }
    }
    
    Ok(count)
}
