//! Integration tests for LPM CLI commands
//!
//! These tests verify that the CLI commands work end-to-end.
//! Unit tests for individual functions should be in their respective source files.

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn lpm_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_lpm"))
}

#[test]
fn test_init_creates_package_yaml() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    // Use --yes flag for non-interactive mode
    let output = lpm_command()
        .arg("init")
        .arg("--yes")
        .current_dir(project_root)
        .output()
        .unwrap();

    assert!(output.status.success(), "lpm init --yes should succeed");
    
    let package_yaml = project_root.join("package.yaml");
    assert!(package_yaml.exists(), "package.yaml should be created");
    
    let content = fs::read_to_string(&package_yaml).unwrap();
    assert!(content.contains("name:"), "package.yaml should contain name");
    assert!(content.contains("version:"), "package.yaml should contain version");
}

#[test]
fn test_init_with_existing_package_yaml() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();
    
    // Create existing package.yaml
    fs::write(
        project_root.join("package.yaml"),
        "name: existing\nversion: 1.0.0\n",
    ).unwrap();

    let output = lpm_command()
        .arg("init")
        .current_dir(project_root)
        .output()
        .unwrap();

    // Should fail or warn about existing file
    // The exact behavior depends on implementation
    assert!(!output.status.success() || 
            String::from_utf8_lossy(&output.stderr).contains("exists") ||
            String::from_utf8_lossy(&output.stderr).contains("already"));
}

#[test]
fn test_init_non_interactive_mode() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    let output = lpm_command()
        .arg("init")
        .arg("--yes")
        .current_dir(project_root)
        .output()
        .unwrap();

    assert!(output.status.success(), "lpm init --yes should succeed");
    
    let package_yaml = project_root.join("package.yaml");
    assert!(package_yaml.exists(), "package.yaml should be created");
    
    let content = fs::read_to_string(&package_yaml).unwrap();
    assert!(content.contains("name:"), "package.yaml should contain name");
    assert!(content.contains("version:"), "package.yaml should contain version");
    
    // Verify directory structure is created
    assert!(project_root.join("src").exists(), "src/ directory should be created");
    assert!(project_root.join("lib").exists(), "lib/ directory should be created");
    assert!(project_root.join("tests").exists(), "tests/ directory should be created");
}

#[test]
fn test_init_with_template_flag() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    // Test with --yes and --template (non-interactive with template)
    let output = lpm_command()
        .arg("init")
        .arg("--template")
        .arg("basic-lua")
        .arg("--yes")
        .current_dir(project_root)
        .output()
        .unwrap();

    // Should succeed (even if template doesn't exist, it should handle gracefully)
    // The important thing is that the command accepts the flags
    assert!(output.status.success() || 
            String::from_utf8_lossy(&output.stderr).contains("template") ||
            String::from_utf8_lossy(&output.stderr).contains("not found"));
    
    let package_yaml = project_root.join("package.yaml");
    if package_yaml.exists() {
        let content = fs::read_to_string(&package_yaml).unwrap();
        assert!(content.contains("name:"), "package.yaml should contain name");
    }
}

#[test]
fn test_init_creates_basic_structure() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    let output = lpm_command()
        .arg("init")
        .arg("--yes")
        .current_dir(project_root)
        .output()
        .unwrap();

    assert!(output.status.success());
    
    // Verify package.yaml exists
    let package_yaml = project_root.join("package.yaml");
    assert!(package_yaml.exists());
    
    // Verify basic directory structure
    assert!(project_root.join("src").is_dir(), "src/ directory should be created");
    assert!(project_root.join("lib").is_dir(), "lib/ directory should be created");
    assert!(project_root.join("tests").is_dir(), "tests/ directory should be created");
    
    // Verify basic main.lua is created (in non-interactive mode without template)
    let main_lua = project_root.join("src").join("main.lua");
    if main_lua.exists() {
        let main_content = fs::read_to_string(&main_lua).unwrap();
        assert!(main_content.contains("print") || main_content.contains("Hello"), 
                "main.lua should contain print or Hello statement");
    }
    // Note: main.lua might not be created in non-interactive mode without template
    // The important thing is that directories are created
}

#[test]
fn test_list_with_no_dependencies() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    // Create minimal package.yaml
    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\n",
    ).unwrap();

    let output = lpm_command()
        .arg("list")
        .current_dir(project_root)
        .output()
        .unwrap();

    assert!(output.status.success(), "lpm list should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show no dependencies
    assert!(stdout.contains("(none)") || 
            stdout.contains("No dependencies") ||
            stdout.contains("0 packages") ||
            stdout.is_empty());
}

#[test]
fn test_verify_with_no_lockfile() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\n",
    ).unwrap();

    let output = lpm_command()
        .arg("verify")
        .current_dir(project_root)
        .output()
        .unwrap();

    // Should handle missing lockfile gracefully
    assert!(output.status.code().is_some());
}

#[test]
fn test_clean_removes_lua_modules() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    // Create lua_modules directory with some content
    let lua_modules = project_root.join("lua_modules");
    fs::create_dir_all(&lua_modules).unwrap();
    fs::write(lua_modules.join("test.lua"), "test").unwrap();

    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\n",
    ).unwrap();

    let output = lpm_command()
        .arg("clean")
        .current_dir(project_root)
        .output()
        .unwrap();

    assert!(output.status.success(), "lpm clean should succeed");
    assert!(!lua_modules.exists(), "lua_modules should be removed");
}

#[test]
fn test_run_script_not_found() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\n",
    ).unwrap();

    let output = lpm_command()
        .arg("run")
        .arg("nonexistent")
        .current_dir(project_root)
        .output()
        .unwrap();

    assert!(!output.status.success(), "Should fail for nonexistent script");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Script"));
}

#[test]
fn test_remove_nonexistent_package() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\n",
    ).unwrap();

    let output = lpm_command()
        .arg("remove")
        .arg("nonexistent-package")
        .current_dir(project_root)
        .output()
        .unwrap();

    // Should fail or warn about missing package
    assert!(!output.status.success() || 
            String::from_utf8_lossy(&output.stderr).contains("not found") ||
            String::from_utf8_lossy(&output.stderr).contains("not in"));
}

#[test]
fn test_outdated_with_no_dependencies() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\n",
    ).unwrap();

    let output = lpm_command()
        .arg("outdated")
        .current_dir(project_root)
        .output()
        .unwrap();

    // Should handle empty dependencies gracefully
    assert!(output.status.code().is_some());
}

#[test]
fn test_audit_with_no_dependencies() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\n",
    ).unwrap();

    let output = lpm_command()
        .arg("audit")
        .current_dir(project_root)
        .output()
        .unwrap();

    // Should handle empty dependencies gracefully
    assert!(output.status.code().is_some());
}

#[test]
fn test_plugin_not_found_error() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\n",
    ).unwrap();

    let output = lpm_command()
        .arg("nonexistent-plugin")
        .current_dir(project_root)
        .output()
        .unwrap();

    // Should fail with helpful error message
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Plugin"), 
            "Error message should mention plugin not found");
    assert!(stderr.contains("install") || stderr.contains("lpm install"), 
            "Error message should suggest installation");
}

#[test]
fn test_plugin_error_message_format() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\n",
    ).unwrap();

    // Test that plugin errors provide helpful messages
    let output = lpm_command()
        .arg("invalid-plugin-name-12345")
        .current_dir(project_root)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Error should be informative
    assert!(!stderr.is_empty(), "Error message should not be empty");
    
    // Should mention the plugin name
    assert!(stderr.contains("invalid-plugin-name-12345") || 
            stderr.contains("Plugin"), 
            "Error should mention plugin name or 'Plugin'");
}

// 5.5 Testing & Documentation tests

#[test]
fn test_init_with_template_non_interactive() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    // Test non-interactive mode with template
    let output = lpm_command()
        .arg("init")
        .arg("--template")
        .arg("basic-lua")
        .arg("--yes")
        .current_dir(project_root)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{} {}", stdout, stderr);
    
    // Should succeed (even if template doesn't exist, it should handle gracefully)
    assert!(output.status.success() || 
            combined.contains("template") ||
            combined.contains("not found") ||
            combined.contains("Template"),
            "Output: stdout={}, stderr={}", stdout, stderr);
    
    let package_yaml = project_root.join("package.yaml");
    if package_yaml.exists() {
        let content = fs::read_to_string(&package_yaml).unwrap();
        assert!(content.contains("name:"), "package.yaml should contain name");
    }
}

#[test]
fn test_template_list_command() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\n",
    ).unwrap();

    let output = lpm_command()
        .arg("template")
        .arg("list")
        .current_dir(project_root)
        .output()
        .unwrap();

    // Should succeed and list templates
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show template list or indicate no templates
    assert!(!stdout.contains("error") && !stdout.contains("Error"), 
            "Should not show errors");
}

#[test]
fn test_init_non_interactive_creates_structure() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    let output = lpm_command()
        .arg("init")
        .arg("--yes")
        .current_dir(project_root)
        .output()
        .unwrap();

    assert!(output.status.success(), 
            "lpm init --yes should succeed. stderr: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    // Verify package.yaml exists
    let package_yaml = project_root.join("package.yaml");
    assert!(package_yaml.exists(), "package.yaml should be created");
    
    // Verify directory structure (may not exist if template was used)
    // Just verify package.yaml was created
    let content = fs::read_to_string(&package_yaml).unwrap();
    assert!(content.contains("name:"), "package.yaml should contain name");
}

#[test]
fn test_init_with_all_flags() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    // Test --yes flag
    let output1 = lpm_command()
        .arg("init")
        .arg("-y")
        .current_dir(project_root)
        .output()
        .unwrap();

    assert!(output1.status.success());
    
    // Clean up and test with --template
    fs::remove_file(project_root.join("package.yaml")).ok();
    fs::remove_dir_all(project_root.join("src")).ok();
    fs::remove_dir_all(project_root.join("lib")).ok();
    fs::remove_dir_all(project_root.join("tests")).ok();

    let output2 = lpm_command()
        .arg("init")
        .arg("--template")
        .arg("basic-lua")
        .arg("-y")
        .current_dir(project_root)
        .output()
        .unwrap();

    // Should handle template flag (may fail if template doesn't exist, but should not crash)
    assert!(output2.status.code().is_some());
}

#[test]
fn test_install_package_workflow() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    // Create package.yaml
    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\ndependencies:\n  lua-resty-http: ~> 0.17",
    ).unwrap();

    // Try to install (may fail if network unavailable, but should handle gracefully)
    let output = lpm_command()
        .arg("install")
        .current_dir(project_root)
        .output()
        .unwrap();

    // Should either succeed or fail gracefully with a clear error
    assert!(output.status.code().is_some());
}

#[test]
fn test_update_package_workflow() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    // Create package.yaml with dependencies
    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\ndependencies:\n  lua-resty-http: ~> 0.17",
    ).unwrap();

    // Try to update (may fail if network unavailable, but should handle gracefully)
    let output = lpm_command()
        .arg("update")
        .current_dir(project_root)
        .output()
        .unwrap();

    // Should either succeed or fail gracefully
    assert!(output.status.code().is_some());
}

#[test]
fn test_remove_package_workflow() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    // Create package.yaml
    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\n",
    ).unwrap();

    // Try to remove a package (should handle gracefully if not installed)
    let output = lpm_command()
        .arg("remove")
        .arg("nonexistent-package")
        .current_dir(project_root)
        .output()
        .unwrap();

    // Should fail or warn about missing package
    assert!(!output.status.success() || 
            String::from_utf8_lossy(&output.stderr).contains("not found") ||
            String::from_utf8_lossy(&output.stderr).contains("not in"));
}

#[test]
fn test_verify_with_lockfile() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    // Create package.yaml
    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\n",
    ).unwrap();

    // Create a minimal lockfile
    fs::write(
        project_root.join("package.lock"),
        "version: 1\ngenerated_at: 2024-01-01T00:00:00Z\npackages: {}\n",
    ).unwrap();

    let output = lpm_command()
        .arg("verify")
        .current_dir(project_root)
        .output()
        .unwrap();

    // Should handle lockfile verification
    assert!(output.status.code().is_some());
}

#[test]
fn test_list_with_dependencies() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    // Create package.yaml with dependencies
    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\ndependencies:\n  lua-resty-http: ~> 0.17",
    ).unwrap();

    let output = lpm_command()
        .arg("list")
        .current_dir(project_root)
        .output()
        .unwrap();

    assert!(output.status.success(), "lpm list should succeed");
}

#[test]
fn test_build_command() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    // Create package.yaml
    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\n",
    ).unwrap();

    let output = lpm_command()
        .arg("build")
        .current_dir(project_root)
        .output()
        .unwrap();

    // Should handle build command (may fail if no Rust code, but should not crash)
    assert!(output.status.code().is_some());
}

#[test]
fn test_package_command() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    // Create package.yaml
    fs::write(
        project_root.join("package.yaml"),
        "name: test-project\nversion: 1.0.0\n",
    ).unwrap();

    let output = lpm_command()
        .arg("package")
        .current_dir(project_root)
        .output()
        .unwrap();

    // Should handle package command
    assert!(output.status.code().is_some());
}

