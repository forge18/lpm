use crate::cache::Cache;
use crate::config::Config;
use crate::core::{LpmError, LpmResult};
use crate::core::path::{lua_modules_dir, ensure_dir, lpm_metadata_dir, packages_metadata_dir};
use crate::luarocks::client::LuaRocksClient;
use crate::luarocks::search_api::SearchAPI;
use crate::luarocks::rockspec::Rockspec;
use crate::package::extractor::PackageExtractor;
use crate::package::lockfile::Lockfile;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Install a package to lua_modules/
pub struct PackageInstaller {
    project_root: PathBuf,
    lua_modules: PathBuf,
    metadata_dir: PathBuf,
    packages_dir: PathBuf,
    search_api: SearchAPI,
    client: LuaRocksClient,
    extractor: PackageExtractor,
}

impl PackageInstaller {
    /// Create a new installer for a project
    pub fn new(project_root: &Path) -> LpmResult<Self> {
        let lua_modules = lua_modules_dir(project_root);
        let metadata_dir = lpm_metadata_dir(project_root);
        let packages_dir = packages_metadata_dir(project_root);
        let config = Config::load()?;
        let cache = Cache::new(config.get_cache_dir()?)?;
        let client = LuaRocksClient::new(&config, cache);
        let search_api = SearchAPI::new();
        let extractor = PackageExtractor::new(lua_modules.clone());

        Ok(Self {
            project_root: project_root.to_path_buf(),
            lua_modules,
            metadata_dir,
            packages_dir,
            search_api,
            client,
            extractor,
        })
    }

    /// Initialize the directory structure
    pub fn init(&self) -> LpmResult<()> {
        ensure_dir(&self.lua_modules)?;
        ensure_dir(&self.metadata_dir)?;
        ensure_dir(&self.packages_dir)?;
        Ok(())
    }

    /// Install a package
    pub async fn install_package(&self, name: &str, version: &str) -> LpmResult<PathBuf> {
        println!("Installing {}@{}", name, version);
        
        // Step 1: Construct and verify rockspec URL
        println!("  Fetching package info...");
        let rockspec_url = self.search_api.get_rockspec_url(name, version, None);
        self.search_api.verify_rockspec_url(&rockspec_url).await?;
        
        // Step 2: Download and parse rockspec to get build configuration
        println!("  Downloading rockspec...");
        let rockspec_content = self.client.download_rockspec(&rockspec_url).await?;
        let rockspec = self.client.parse_rockspec(&rockspec_content)?;
        
        // Step 3: Download source archive
        println!("  Downloading source...");
        let source_path = self.client.download_source(&rockspec.source.url).await?;
        
        // Step 4: Verify checksum if lockfile exists (ensures reproducible installs)
        if let Some(lockfile) = Lockfile::load(&self.project_root)? {
            if let Some(locked_pkg) = lockfile.get_package(name) {
                println!("  Verifying checksum...");
                let actual = Cache::checksum(&source_path)?;
                if actual != locked_pkg.checksum {
                    return Err(LpmError::Package(format!(
                        "Checksum mismatch for {}@{}. Expected {}, got {}",
                        name, version, locked_pkg.checksum, actual
                    )));
                }
                println!("  ✓ Checksum verified");
            }
        }
        
        // Step 5: Extract source archive to temporary directory
        println!("  Extracting...");
        let extracted_path = self.extractor.extract(&source_path)?;
        
        // Step 6: Build and install based on rockspec build type
        println!("  Installing...");
        self.install_from_source(&extracted_path, name, &rockspec)?;
        
        // Step 7: Calculate checksum for lockfile generation
        let checksum = Cache::checksum(&source_path)?;
        
        println!("  ✓ Installed {} (checksum: {})", name, checksum);
        
        Ok(self.lua_modules.join(name))
    }
    
    fn install_from_source(&self, source_path: &Path, package_name: &str, rockspec: &Rockspec) -> LpmResult<()> {
        match rockspec.build.build_type.as_str() {
            "none" | "builtin" => {
                // Pure Lua modules: copy files directly without building.
                self.install_builtin(source_path, package_name, rockspec)
            },
            "make" => {
                // Build using Makefile.
                self.build_with_make(source_path, package_name, rockspec)
            },
            "cmake" => {
                // Build using CMake.
                self.build_with_cmake(source_path, package_name, rockspec)
            },
            "command" => {
                // Build using custom command specified in rockspec.
                self.build_with_command(source_path, package_name, rockspec)
            },
            "rust" | "rust-mlua" => {
                // Rust extensions using mlua: build with cargo.
                self.build_with_rust(source_path, package_name, rockspec)
            },
            _ => Err(LpmError::NotImplemented(format!(
                "Build type '{}' not supported. Supported types: builtin, none, make, cmake, command, rust.",
                rockspec.build.build_type
            ))),
        }
    }
    
    fn build_with_make(&self, source_path: &Path, package_name: &str, rockspec: &Rockspec) -> LpmResult<()> {
        use std::process::Command;
        
        println!("  Building with make...");
        
        let mut make_cmd = Command::new("make");
        make_cmd.current_dir(source_path);
        
        let status = make_cmd.status()
            .map_err(|e| LpmError::Package(format!("Failed to run make: {}", e)))?;
        
        if !status.success() {
            return Err(LpmError::Package("make build failed".to_string()));
        }
        
        // Install using make install (if install target exists) or copy built files
        let dest = self.lua_modules.join(package_name);
        fs::create_dir_all(&dest)?;
        
        // Attempt make install first, fall back to manual file copying if needed
        let mut install_cmd = Command::new("make");
        install_cmd.arg("install");
        install_cmd.current_dir(source_path);
        install_cmd.env("PREFIX", &dest);
        
        if install_cmd.status().is_ok() {
            println!("  ✓ Installed via make install");
            return Ok(());
        }
        
        // Fall back to copying files based on rockspec.build.install or build.modules.
        // Handle install table sections: bin, lua, lib, conf.
        let has_install = !rockspec.build.install.bin.is_empty() 
            || !rockspec.build.install.lua.is_empty()
            || !rockspec.build.install.lib.is_empty()
            || !rockspec.build.install.conf.is_empty();
            
        if has_install {
            // Copy files from install.bin (executables).
            for source_path_str in rockspec.build.install.bin.values() {
                let src = source_path.join(source_path_str);
                if src.exists() {
                    let relative = src.strip_prefix(source_path)
                        .map_err(|e| LpmError::Path(e.to_string()))?;
                    let dst = dest.join(relative);
                    
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    if src.is_dir() {
                        copy_dir_recursive(&src, &dst)?;
                    } else {
                        fs::copy(&src, &dst)?;
                    }
                }
            }
            
            // Copy files from install.lua (Lua modules).
            for source_path_str in rockspec.build.install.lua.values() {
                let src = source_path.join(source_path_str);
                if src.exists() {
                    let relative = src.strip_prefix(source_path)
                        .map_err(|e| LpmError::Path(e.to_string()))?;
                    let dst = dest.join(relative);
                    
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    if src.is_dir() {
                        copy_dir_recursive(&src, &dst)?;
                    } else {
                        fs::copy(&src, &dst)?;
                    }
                }
            }
            
            // Copy files from install.lib (native libraries).
            for source_path_str in rockspec.build.install.lib.values() {
                let src = source_path.join(source_path_str);
                if src.exists() {
                    let relative = src.strip_prefix(source_path)
                        .map_err(|e| LpmError::Path(e.to_string()))?;
                    let dst = dest.join(relative);
                    
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    fs::copy(&src, &dst)?;
                }
            }
            
            // Copy files from install.conf (configuration files).
            for source_path_str in rockspec.build.install.conf.values() {
                let src = source_path.join(source_path_str);
                if src.exists() {
                    let relative = src.strip_prefix(source_path)
                        .map_err(|e| LpmError::Path(e.to_string()))?;
                    let dst = dest.join(relative);
                    
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    if src.is_dir() {
                        copy_dir_recursive(&src, &dst)?;
                    } else {
                        fs::copy(&src, &dst)?;
                    }
                }
            }
        } else if !rockspec.build.modules.is_empty() {
            // Copy modules specified in build.modules.
            for source_file in rockspec.build.modules.values() {
                let src = source_path.join(source_file);
                if src.exists() {
                    let relative = src.strip_prefix(source_path)
                        .map_err(|e| LpmError::Path(e.to_string()))?;
                    let dst = dest.join(relative);
                    
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(&src, &dst)?;
                }
            }
        } else {
            // Copy everything as fallback.
            copy_dir_recursive(source_path, &dest)?;
        }
        
        println!("  ✓ Installed built package");
        Ok(())
    }
    
    fn build_with_cmake(&self, source_path: &Path, package_name: &str, rockspec: &Rockspec) -> LpmResult<()> {
        use std::process::Command;
        
        println!("  Building with cmake...");
        
        // Create build directory for CMake.
        let build_dir = source_path.join("build");
        fs::create_dir_all(&build_dir)?;
        
        // Run cmake configure step.
        let mut cmake_cmd = Command::new("cmake");
        cmake_cmd.arg("..");
        cmake_cmd.current_dir(&build_dir);
        
        let status = cmake_cmd.status()
            .map_err(|e| LpmError::Package(format!("Failed to run cmake: {}", e)))?;
        
        if !status.success() {
            return Err(LpmError::Package("cmake configure failed".to_string()));
        }
        
        // Run cmake build step.
        let mut build_cmd = Command::new("cmake");
        build_cmd.args(["--build", "."]);
        build_cmd.current_dir(&build_dir);
        
        let status = build_cmd.status()
            .map_err(|e| LpmError::Package(format!("Failed to run cmake build: {}", e)))?;
        
        if !status.success() {
            return Err(LpmError::Package("cmake build failed".to_string()));
        }
        
        // Install built files to destination.
        let dest = self.lua_modules.join(package_name);
        fs::create_dir_all(&dest)?;
        
        // Attempt cmake install first.
        let mut install_cmd = Command::new("cmake");
        install_cmd.args(["--install", ".", "--prefix", dest.to_str().unwrap()]);
        install_cmd.current_dir(&build_dir);
        
        if install_cmd.status().is_ok() {
            println!("  ✓ Installed via cmake install");
            return Ok(());
        }
        
        // Fall back to copying from build directory.
        let has_install = !rockspec.build.install.bin.is_empty() 
            || !rockspec.build.install.lua.is_empty()
            || !rockspec.build.install.lib.is_empty()
            || !rockspec.build.install.conf.is_empty();
            
        if has_install {
            // Copy from install table sections.
            for source_path_str in rockspec.build.install.bin.values() {
                let src = build_dir.join(source_path_str);
                if src.exists() {
                    let relative = src.strip_prefix(&build_dir)
                        .map_err(|e| LpmError::Path(e.to_string()))?;
                    let dst = dest.join(relative);
                    
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    if src.is_dir() {
                        copy_dir_recursive(&src, &dst)?;
                    } else {
                        fs::copy(&src, &dst)?;
                    }
                }
            }
            
            for source_path_str in rockspec.build.install.lua.values() {
                let src = build_dir.join(source_path_str);
                if src.exists() {
                    let relative = src.strip_prefix(&build_dir)
                        .map_err(|e| LpmError::Path(e.to_string()))?;
                    let dst = dest.join(relative);
                    
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    fs::copy(&src, &dst)?;
                }
            }
            
            for source_path_str in rockspec.build.install.lib.values() {
                let src = build_dir.join(source_path_str);
                if src.exists() {
                    let relative = src.strip_prefix(&build_dir)
                        .map_err(|e| LpmError::Path(e.to_string()))?;
                    let dst = dest.join(relative);
                    
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    fs::copy(&src, &dst)?;
                }
            }
        } else {
            // Copy built files from build directory.
            copy_dir_recursive(&build_dir, &dest)?;
        }
        
        println!("  ✓ Installed built package");
        Ok(())
    }
    
    fn build_with_command(&self, source_path: &Path, package_name: &str, rockspec: &Rockspec) -> LpmResult<()> {
        use std::process::Command;
        
        // For "command" build type, parse the command from rockspec.
        // LuaRocks stores it in build.variables or build.command.
        // This implementation checks for a common build.sh pattern.
        
        println!("  Building with custom command...");
        
        // Check for build script or command specification.
        // Full implementation would parse rockspec.build.variables.
        let build_script = source_path.join("build.sh");
        if build_script.exists() {
            let mut cmd = Command::new("sh");
            cmd.arg(&build_script);
            cmd.current_dir(source_path);
            
            let status = cmd.status()
                .map_err(|e| LpmError::Package(format!("Failed to run build script: {}", e)))?;
            
            if !status.success() {
                return Err(LpmError::Package("Custom build command failed".to_string()));
            }
        } else {
            return Err(LpmError::Package(
                "command build type requires a build script or command specification in rockspec".to_string()
            ));
        }
        
        // Install built files to destination.
        let dest = self.lua_modules.join(package_name);
        fs::create_dir_all(&dest)?;
        
        let has_install = !rockspec.build.install.bin.is_empty() 
            || !rockspec.build.install.lua.is_empty()
            || !rockspec.build.install.lib.is_empty()
            || !rockspec.build.install.conf.is_empty();
            
        if has_install {
            // Copy from install table sections.
            for source_path_str in rockspec.build.install.bin.values() {
                let src = source_path.join(source_path_str);
                if src.exists() {
                    let relative = src.strip_prefix(source_path)
                        .map_err(|e| LpmError::Path(e.to_string()))?;
                    let dst = dest.join(relative);
                    
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    if src.is_dir() {
                        copy_dir_recursive(&src, &dst)?;
                    } else {
                        fs::copy(&src, &dst)?;
                    }
                }
            }
            
            for source_path_str in rockspec.build.install.lua.values() {
                let src = source_path.join(source_path_str);
                if src.exists() {
                    let relative = src.strip_prefix(source_path)
                        .map_err(|e| LpmError::Path(e.to_string()))?;
                    let dst = dest.join(relative);
                    
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    fs::copy(&src, &dst)?;
                }
            }
        } else {
            // Copy everything as fallback.
            copy_dir_recursive(source_path, &dest)?;
        }
        
        println!("  ✓ Installed built package");
        Ok(())
    }
    
    fn build_with_rust(&self, source_path: &Path, package_name: &str, rockspec: &Rockspec) -> LpmResult<()> {
        use std::process::Command;
        
        println!("  Building Rust extension...");
        
        // Verify Cargo.toml exists (required for Rust builds).
        let cargo_toml = source_path.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err(LpmError::Package(
                "Rust build type requires Cargo.toml in package source".to_string()
            ));
        }
        
        // Build with cargo in release mode.
        let mut build_cmd = Command::new("cargo");
        build_cmd.args(["build", "--release"]);
        build_cmd.current_dir(source_path);
        
        let status = build_cmd.status()
            .map_err(|e| LpmError::Package(format!("Failed to run cargo build: {}", e)))?;
        
        if !status.success() {
            return Err(LpmError::Package("cargo build failed".to_string()));
        }
        
        // Find the built library in target/release/.
        // Look for platform-specific extensions: .so, .dylib, or .dll.
        let target_dir = source_path.join("target").join("release");
        let lib_ext = if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };
        
        // Search for the built library file in target/release/.
        let lib_file = std::fs::read_dir(&target_dir)?
            .filter_map(|e| e.ok())
            .find(|e| {
                e.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == lib_ext)
                    .unwrap_or(false)
            });
        
        // Install built files to destination.
        let dest = self.lua_modules.join(package_name);
        fs::create_dir_all(&dest)?;
        
        if let Some(lib_entry) = lib_file {
            // Copy the built library to destination.
            let lib_path = lib_entry.path();
            let lib_name = lib_path.file_name()
                .ok_or_else(|| LpmError::Package("Invalid library path".to_string()))?;
            let dest_lib = dest.join(lib_name);
            fs::copy(&lib_path, &dest_lib)?;
            println!("  ✓ Copied library: {}", lib_name.to_string_lossy());
        }
        
        // Copy Lua files if specified in modules.
        if !rockspec.build.modules.is_empty() {
            for source_file in rockspec.build.modules.values() {
                let src = source_path.join(source_file);
                if src.exists() {
                    let relative = src.strip_prefix(source_path)
                        .map_err(|e| LpmError::Path(e.to_string()))?;
                    let dst = dest.join(relative);
                    
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(&src, &dst)?;
                }
            }
        }
        
        // Copy any other files specified in install table.
        let has_install = !rockspec.build.install.bin.is_empty() 
            || !rockspec.build.install.lua.is_empty()
            || !rockspec.build.install.lib.is_empty()
            || !rockspec.build.install.conf.is_empty();
            
        if has_install {
            for source_path_str in rockspec.build.install.bin.values() {
                let src = source_path.join(source_path_str);
                if src.exists() {
                    let relative = src.strip_prefix(source_path)
                        .map_err(|e| LpmError::Path(e.to_string()))?;
                    let dst = dest.join(relative);
                    
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    fs::copy(&src, &dst)?;
                }
            }
            
            for source_path_str in rockspec.build.install.lua.values() {
                let src = source_path.join(source_path_str);
                if src.exists() {
                    let relative = src.strip_prefix(source_path)
                        .map_err(|e| LpmError::Path(e.to_string()))?;
                    let dst = dest.join(relative);
                    
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    fs::copy(&src, &dst)?;
                }
            }
            
            for source_path_str in rockspec.build.install.lib.values() {
                let src = source_path.join(source_path_str);
                if src.exists() {
                    let relative = src.strip_prefix(source_path)
                        .map_err(|e| LpmError::Path(e.to_string()))?;
                    let dst = dest.join(relative);
                    
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    fs::copy(&src, &dst)?;
                }
            }
        }
        
        println!("  ✓ Installed Rust extension");
        Ok(())
    }
    
    fn install_builtin(&self, source_path: &Path, package_name: &str, rockspec: &Rockspec) -> LpmResult<()> {
        let dest = self.lua_modules.join(package_name);
        fs::create_dir_all(&dest)?;
        
        if rockspec.build.modules.is_empty() {
            // Copy everything (standard case for most packages).
            copy_dir_recursive(source_path, &dest)?;
        } else {
            // Copy only the specified modules.
            for source_file in rockspec.build.modules.values() {
                let src = source_path.join(source_file);
                if !src.exists() {
                    return Err(LpmError::Package(format!(
                        "Module file not found in source: {}",
                        source_file
                    )));
                }
                
                let relative = src.strip_prefix(source_path)
                    .map_err(|e| LpmError::Path(e.to_string()))?;
                let dst = dest.join(relative);
                
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&src, &dst)?;
            }
        }
        
        Ok(())
    }

    /// Get the installation path for a package
    pub fn get_package_path(&self, name: &str) -> PathBuf {
        self.lua_modules.join(name)
    }

    /// Check if a package is installed
    pub fn is_installed(&self, name: &str) -> bool {
        self.lua_modules.join(name).exists()
    }

    /// Remove a package
    pub fn remove_package(&self, name: &str) -> LpmResult<()> {
        let package_dir = self.lua_modules.join(name);
        let metadata_dir = self.packages_dir.join(name);

        if package_dir.exists() {
            fs::remove_dir_all(&package_dir)?;
        }

        if metadata_dir.exists() {
            fs::remove_dir_all(&metadata_dir)?;
        }

        Ok(())
    }
}

/// Copy directory recursively from source to destination
fn copy_dir_recursive(src: &Path, dst: &Path) -> LpmResult<()> {
    for entry in WalkDir::new(src) {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(src)
            .map_err(|e| LpmError::Path(e.to_string()))?;
        let dest_path = dst.join(relative);
        
        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &dest_path)?;
        }
    }
    Ok(())
}

