use crate::core::{LpmError, LpmResult};
use crate::luarocks::rockspec::{Rockspec, RockspecBuild, RockspecSource};
use regex::Regex;
use std::collections::HashMap;

/// Parse a rockspec file using regex-based extraction
///
/// This is a basic parser that extracts common fields from rockspec files.
/// For full sandboxed parsing, we'll need a Lua interpreter later.
pub fn parse_rockspec(content: &str) -> LpmResult<Rockspec> {
    let package = extract_string_field(content, r#"package\s*=\s*"([^"]+)""#)?;
    let version = extract_string_field(content, r#"version\s*=\s*"([^"]+)""#)?;

    // Parse source table
    let source = parse_source(content)?;

    // Parse dependencies
    let dependencies = parse_dependencies(content)?;

    // Parse build table
    let build = parse_build(content)?;

    // Optional fields
    let description = extract_string_field(content, r#"description\s*=\s*"([^"]+)""#).ok();
    let homepage = extract_string_field(content, r#"homepage\s*=\s*"([^"]+)""#).ok();
    let license = extract_string_field(content, r#"license\s*=\s*"([^"]+)""#).ok();
    let lua_version = extract_string_field(content, r#"lua_version\s*=\s*"([^"]+)""#).ok();

    // Parse binary_urls from metadata (if present)
    let binary_urls = parse_binary_urls(content).unwrap_or_default();

    Ok(Rockspec {
        package,
        version,
        source,
        dependencies,
        build,
        description,
        homepage,
        license,
        lua_version,
        binary_urls,
    })
}

fn extract_string_field(content: &str, pattern: &str) -> LpmResult<String> {
    let re = Regex::new(pattern)
        .map_err(|e| LpmError::Package(format!("Invalid regex pattern: {}", e)))?;

    re.captures(content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| LpmError::Package(format!("Field not found: {}", pattern)))
}

fn parse_source(content: &str) -> LpmResult<RockspecSource> {
    // Extract source table
    let source_block = extract_table_block(content, "source")?;

    let url = extract_string_field(&source_block, r#"url\s*=\s*"([^"]+)""#)
        .or_else(|_| extract_string_field(&source_block, r#"url\s*=\s*'([^']+)'"#))?;

    let tag = extract_string_field(&source_block, r#"tag\s*=\s*"([^"]+)""#)
        .or_else(|_| extract_string_field(&source_block, r#"tag\s*=\s*'([^']+)'"#))
        .ok();

    let branch = extract_string_field(&source_block, r#"branch\s*=\s*"([^"]+)""#)
        .or_else(|_| extract_string_field(&source_block, r#"branch\s*=\s*'([^']+)'"#))
        .ok();

    Ok(RockspecSource { url, tag, branch })
}

fn parse_dependencies(content: &str) -> LpmResult<Vec<String>> {
    let deps_block = extract_table_block(content, "dependencies")?;

    // Match entries like: "lua >= 5.1" or 'luasocket'
    let re = Regex::new(r#"(?m)^\s*["']([^"']+)["']"#)
        .map_err(|e| LpmError::Package(format!("Invalid regex: {}", e)))?;

    let mut deps = Vec::new();
    for cap in re.captures_iter(&deps_block) {
        if let Some(m) = cap.get(1) {
            deps.push(m.as_str().to_string());
        }
    }

    Ok(deps)
}

fn parse_build(content: &str) -> LpmResult<RockspecBuild> {
    let build_block = extract_table_block(content, "build")?;

    let build_type = extract_string_field(&build_block, r#"type\s*=\s*"([^"]+)""#)
        .or_else(|_| extract_string_field(&build_block, r#"type\s*=\s*'([^']+)'"#))
        .unwrap_or_else(|_| "builtin".to_string());

    // Parse modules table
    let modules = parse_modules_table(&build_block)?;

    // Parse install table
    let install = parse_install_table(&build_block)?;

    Ok(RockspecBuild {
        build_type,
        modules,
        install,
    })
}

/// Parse install table from build block
/// Format: install = { bin = { ["name"] = "path" }, lua = { ... }, lib = { ... }, conf = { ... } }
fn parse_install_table(build_block: &str) -> LpmResult<crate::luarocks::rockspec::InstallTable> {
    use crate::luarocks::rockspec::InstallTable;

    let install_block = extract_table_block(build_block, "install").unwrap_or_default();

    let bin = parse_install_section(&install_block, "bin")?;
    let lua = parse_install_section(&install_block, "lua")?;
    let lib = parse_install_section(&install_block, "lib")?;
    let conf = parse_install_section(&install_block, "conf")?;

    Ok(InstallTable {
        bin,
        lua,
        lib,
        conf,
    })
}

/// Parse a section of the install table (bin, lua, lib, or conf)
fn parse_install_section(
    install_block: &str,
    section_name: &str,
) -> LpmResult<HashMap<String, String>> {
    let section_block = extract_table_block(install_block, section_name).unwrap_or_default();

    // Match entries like: ["name"] = "path" or name = "path"
    let re = Regex::new(r#"(?m)^\s*(?:\["([^"]+)"\]|(\w+))\s*=\s*["']([^"']+)["']"#)
        .map_err(|e| LpmError::Package(format!("Invalid regex: {}", e)))?;

    let mut entries = HashMap::new();
    for cap in re.captures_iter(&section_block) {
        let name = cap
            .get(1)
            .or_else(|| cap.get(2))
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| {
                LpmError::Package(format!("Invalid entry in install.{}", section_name))
            })?;
        let path = cap.get(3).map(|m| m.as_str().to_string()).ok_or_else(|| {
            LpmError::Package(format!("Invalid path in install.{}", section_name))
        })?;
        entries.insert(name, path);
    }

    Ok(entries)
}

fn parse_modules_table(build_block: &str) -> LpmResult<HashMap<String, String>> {
    // Extract modules table block
    let modules_block = extract_table_block(build_block, "modules").unwrap_or_default();

    // Match entries like: socket = "src/socket.lua"
    let re = Regex::new(r#"(?m)^\s*(\w+)\s*=\s*["']([^"']+)["']"#)
        .map_err(|e| LpmError::Package(format!("Invalid regex: {}", e)))?;

    let mut modules = HashMap::new();
    for cap in re.captures_iter(&modules_block) {
        if let (Some(name), Some(path)) = (cap.get(1), cap.get(2)) {
            modules.insert(name.as_str().to_string(), path.as_str().to_string());
        }
    }

    Ok(modules)
}

/// Parse binary_urls table from metadata section
/// Format: binary_urls = { ["5.4-x86_64-unknown-linux-gnu"] = "https://..." }
/// Or directly: binary_urls = { ["5.4-x86_64-unknown-linux-gnu"] = "https://..." }
fn parse_binary_urls(content: &str) -> LpmResult<HashMap<String, String>> {
    // Try to find binary_urls directly first
    let binary_urls_block = extract_table_block(content, "binary_urls")
        .or_else(|_| {
            // If not found, try within metadata table
            let metadata_block = extract_table_block(content, "metadata")?;
            extract_table_block(&metadata_block, "binary_urls")
        })
        .ok();

    let Some(binary_urls_block) = binary_urls_block else {
        return Ok(HashMap::new());
    };

    // Match entries like: ["5.4-x86_64-unknown-linux-gnu"] = "https://..."
    let re = Regex::new(r#"(?m)\["([^\]]+)"\]\s*=\s*["']([^"']+)["']"#)
        .map_err(|e| LpmError::Package(format!("Invalid regex: {}", e)))?;

    let mut urls = HashMap::new();
    for cap in re.captures_iter(&binary_urls_block) {
        if let (Some(target), Some(url)) = (cap.get(1), cap.get(2)) {
            urls.insert(target.as_str().to_string(), url.as_str().to_string());
        }
    }

    Ok(urls)
}

/// Extract a table block from Lua code
///
/// Finds a table like:
///   field = {
///     ...
///   }
fn extract_table_block(content: &str, field_name: &str) -> LpmResult<String> {
    let pattern = format!(r#"{}\s*=\s*\{{"#, field_name);
    let start_re =
        Regex::new(&pattern).map_err(|e| LpmError::Package(format!("Invalid regex: {}", e)))?;

    let start_match = start_re
        .find(content)
        .ok_or_else(|| LpmError::Package(format!("Field '{}' not found", field_name)))?;

    let start_pos = start_match.end();
    let mut brace_count = 1;
    let mut pos = start_pos;
    let chars: Vec<char> = content.chars().collect();

    while pos < chars.len() && brace_count > 0 {
        match chars[pos] {
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            _ => {}
        }
        pos += 1;
    }

    if brace_count != 0 {
        return Err(LpmError::Package(format!(
            "Unclosed table block for '{}'",
            field_name
        )));
    }

    Ok(content[start_match.start()..pos].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_rockspec() {
        let content = r#"
package = "luasocket"
version = "3.0-1"

source = {
   url = "https://github.com/lunarmodules/luasocket/archive/v3.0.tar.gz"
}

dependencies = {
   "lua >= 5.1"
}

build = {
   type = "builtin",
   modules = {
      socket = "src/socket.lua"
   }
}
"#;

        let rockspec = parse_rockspec(content).unwrap();
        assert_eq!(rockspec.package, "luasocket");
        assert_eq!(rockspec.version, "3.0-1");
        assert_eq!(
            rockspec.source.url,
            "https://github.com/lunarmodules/luasocket/archive/v3.0.tar.gz"
        );
        assert_eq!(rockspec.dependencies.len(), 1);
        assert_eq!(rockspec.build.build_type, "builtin");
        assert_eq!(rockspec.build.modules.len(), 1);
    }
}
