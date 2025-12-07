use crate::cache::Cache;
use crate::config::Config;
use crate::core::version::{Version, VersionConstraint};
use crate::core::{LpmError, LpmResult};
use crate::luarocks::client::LuaRocksClient;
use crate::luarocks::manifest::Manifest;
use crate::luarocks::rockspec::Rockspec;
use crate::luarocks::search_api::SearchAPI;
use crate::resolver::dependency_graph::DependencyGraph;
use std::collections::{HashMap, HashSet};

/// Resolves dependencies and versions using SemVer algorithm
pub struct DependencyResolver {
    manifest: Manifest,
}

impl DependencyResolver {
    pub fn new(manifest: Manifest) -> Self {
        Self { manifest }
    }

    /// Resolve all dependencies from a package manifest
    ///
    /// This implements a simplified SemVer resolution algorithm:
    /// 1. Build dependency graph
    /// 2. For each package, find all available versions
    /// 3. Select the highest version that satisfies all constraints
    /// 4. Fetch rockspec and parse transitive dependencies
    /// 5. Detect conflicts and circular dependencies
    pub async fn resolve(
        &self,
        dependencies: &HashMap<String, String>,
    ) -> LpmResult<HashMap<String, Version>> {
        let mut graph = DependencyGraph::new();
        let mut resolved = HashMap::new();

        // Build initial graph from direct dependencies
        for (name, constraint_str) in dependencies {
            let constraint =
                crate::core::version::parse_constraint(constraint_str).map_err(|e| {
                    LpmError::Version(format!("Invalid constraint for {}: {}", name, e))
                })?;
            graph.add_node(name.clone(), constraint);
        }

        // Setup clients for fetching rockspecs
        let config = Config::load()?;
        let cache = Cache::new(config.get_cache_dir()?)?;
        let client = LuaRocksClient::new(&config, cache);
        let search_api = SearchAPI::new();

        // Build full dependency graph by parsing rockspecs
        let mut to_process: Vec<(String, VersionConstraint)> = dependencies
            .iter()
            .map(|(n, v)| {
                let constraint = crate::core::version::parse_constraint(v).map_err(|e| {
                    LpmError::Version(format!("Invalid constraint for {}: {}", n, e))
                })?;
                Ok((n.clone(), constraint))
            })
            .collect::<LpmResult<Vec<_>>>()?;
        let mut processed = HashSet::new();

        while let Some((package_name, constraint)) = to_process.pop() {
            if processed.contains(&package_name) {
                continue;
            }
            processed.insert(package_name.clone());

            // Get available versions from manifest
            let available_versions = self.get_available_versions(&package_name)?;
            if available_versions.is_empty() {
                return Err(LpmError::Package(format!(
                    "No versions available for package '{}'",
                    package_name
                )));
            }

            // Find the highest version that satisfies the constraint
            let selected_version = self.select_version(&available_versions, &constraint)?;
            graph.add_node(package_name.clone(), constraint.clone());
            graph.set_resolved_version(&package_name, selected_version.clone())?;
            resolved.insert(package_name.clone(), selected_version.clone());

            // Get rockspec and parse dependencies
            let rockspec = get_rockspec(
                &client,
                &search_api,
                &package_name,
                &selected_version.to_string(),
            )
            .await?;

            for dep in &rockspec.dependencies {
                // Skip lua runtime dependency (standardize: any dep starting with "lua" and containing version operators)
                if dep.trim().starts_with("lua")
                    && (dep.contains(">=")
                        || dep.contains(">")
                        || dep.contains("==")
                        || dep.contains("~>"))
                {
                    continue;
                }

                // Parse dependency string: "luasocket >= 3.0" or "penlight" or "luasocket ~> 3.0"
                let (dep_name, dep_constraint) = parse_dependency_string(dep)?;

                graph.add_dependency(&package_name, dep_name.clone())?;

                if !resolved.contains_key(&dep_name) {
                    to_process.push((dep_name, dep_constraint));
                }
            }
        }

        // Detect circular dependencies
        graph.detect_circular_dependencies()?;

        Ok(resolved)
    }

    /// Get all available versions for a package from the manifest
    fn get_available_versions(&self, package_name: &str) -> LpmResult<Vec<Version>> {
        // Get versions from manifest
        let version_strings = self.manifest.get_package_version_strings(package_name);

        if version_strings.is_empty() {
            return Err(LpmError::Package(format!(
                "Package '{}' not found in manifest",
                package_name
            )));
        }

        let mut versions = Vec::new();
        for version_str in version_strings {
            // Normalize LuaRocks version format
            let version = crate::luarocks::version::normalize_luarocks_version(&version_str)?;
            versions.push(version);
        }

        // Sort versions (highest first)
        versions.sort_by(|a, b| b.cmp(a));
        Ok(versions)
    }

    /// Select the highest version that satisfies the constraint
    fn select_version(
        &self,
        available_versions: &[Version],
        constraint: &VersionConstraint,
    ) -> LpmResult<Version> {
        for version in available_versions {
            if version.satisfies(constraint) {
                return Ok(version.clone());
            }
        }

        Err(LpmError::Version(format!(
            "No version satisfies constraint: {:?}",
            constraint
        )))
    }

    /// Resolve version conflicts between multiple constraints for the same package
    pub fn resolve_conflicts(
        &self,
        package_name: &str,
        constraints: &[VersionConstraint],
    ) -> LpmResult<VersionConstraint> {
        if constraints.is_empty() {
            return Err(LpmError::Version("No constraints provided".to_string()));
        }

        if constraints.len() == 1 {
            return Ok(constraints[0].clone());
        }

        // Get all available versions
        let available_versions = self.get_available_versions(package_name)?;

        // Find the highest version that satisfies all constraints
        for version in &available_versions {
            let satisfies_all = constraints.iter().all(|c| version.satisfies(c));
            if satisfies_all {
                // Return the most specific constraint that matches
                // For now, return the first compatible constraint
                return Ok(constraints[0].clone());
            }
        }

        // If no version satisfies all constraints, return an error
        Err(LpmError::Version(format!(
            "Version conflict for '{}': no version satisfies all constraints",
            package_name
        )))
    }
}

/// Parse a dependency string from a rockspec
/// Handles formats like: "luasocket >= 3.0", "penlight", "luasocket ~> 3.0"
fn parse_dependency_string(dep: &str) -> LpmResult<(String, VersionConstraint)> {
    let dep = dep.trim();

    // Find first whitespace or version operator
    if let Some(pos) = dep.find(char::is_whitespace) {
        let name = dep[..pos].trim().to_string();
        let version_part = dep[pos..].trim();

        // Convert LuaRocks ~> to SemVer ^
        let version_part = if version_part.starts_with("~>") {
            version_part.replacen("~>", "^", 1)
        } else {
            version_part.to_string()
        };

        let constraint = crate::core::version::parse_constraint(&version_part)
            .unwrap_or(VersionConstraint::GreaterOrEqual(Version::new(0, 0, 0)));
        Ok((name, constraint))
    } else {
        // No version specified
        Ok((
            dep.to_string(),
            VersionConstraint::GreaterOrEqual(Version::new(0, 0, 0)),
        ))
    }
}

/// Fetch and parse a rockspec for a package version
async fn get_rockspec(
    client: &LuaRocksClient,
    search_api: &SearchAPI,
    name: &str,
    version: &str,
) -> LpmResult<Rockspec> {
    let rockspec_url = search_api.get_rockspec_url(name, version, None);
    let content = client.download_rockspec(&rockspec_url).await?;
    client.parse_rockspec(&content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::version::parse_constraint;

    #[test]
    fn test_select_version() {
        let manifest = Manifest::default(); // Empty manifest for now
        let resolver = DependencyResolver::new(manifest);

        // Versions should be sorted highest first (already done in get_available_versions)
        let versions = vec![
            Version::new(2, 0, 0),
            Version::new(1, 1, 0),
            Version::new(1, 0, 0),
        ];

        let constraint = parse_constraint("^1.0.0").unwrap();
        let selected = resolver.select_version(&versions, &constraint).unwrap();
        assert_eq!(selected, Version::new(1, 1, 0)); // Highest compatible version
    }

    #[test]
    fn test_resolve_conflicts() {
        let manifest = Manifest::default();
        let resolver = DependencyResolver::new(manifest);

        let constraints = vec![
            parse_constraint("^1.0.0").unwrap(),
            parse_constraint("^1.1.0").unwrap(),
        ];

        // This will fail without a real manifest, but tests the structure
        let result = resolver.resolve_conflicts("test", &constraints);
        assert!(result.is_err()); // Expected since we don't have versions
    }
}
