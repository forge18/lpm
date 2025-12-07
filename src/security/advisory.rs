use crate::core::{LpmError, LpmResult};
use crate::security::vulnerability::{Severity, Vulnerability};
use std::collections::HashMap;

/// Database of security advisories
///
/// This can be extended to load from external sources (API, file, etc.)
pub struct AdvisoryDatabase {
    advisories: HashMap<String, Vec<Vulnerability>>,
}

impl AdvisoryDatabase {
    /// Create a new advisory database
    pub fn new() -> Self {
        Self {
            advisories: HashMap::new(),
        }
    }

    /// Load advisories from a source (currently uses built-in data)
    pub fn load() -> LpmResult<Self> {
        let mut db = Self::new();

        // Load built-in advisories
        db.load_builtin_advisories();

        // Future enhancement: Load from external sources (API, file, etc.)
        // This would enable automatic updates and integration with vulnerability databases
        // db.load_from_file("~/.lpm/advisories.json")?;
        // db.load_from_api("https://advisories.luarocks.org/api/v1/advisories")?;

        Ok(db)
    }

    /// Load built-in security advisories.
    ///
    /// This is a placeholder for known vulnerabilities.
    /// In production, this would be loaded from an external database.
    fn load_builtin_advisories(&mut self) {
        // Placeholder for adding known vulnerabilities.
        // In production, advisories would be loaded from external sources
        // such as OSV (Open Source Vulnerabilities) or a LuaRocks-specific database.

        // Example vulnerability structure (not a real vulnerability):
        // self.add_advisory(Vulnerability {
        //     package: "example-package".to_string(),
        //     affected_versions: "<2.0.0".to_string(),
        //     severity: Severity::High,
        //     cve: Some("CVE-2024-XXXX".to_string()),
        //     title: "Security vulnerability in example-package".to_string(),
        //     description: "Detailed description...".to_string(),
        //     fixed_in: Some("2.0.0".to_string()),
        //     references: vec!["https://example.com/advisory".to_string()],
        // });
    }

    /// Add an advisory to the database
    pub fn add_advisory(&mut self, vuln: Vulnerability) {
        self.advisories
            .entry(vuln.package.clone())
            .or_default()
            .push(vuln);
    }

    /// Check a package version for vulnerabilities
    pub fn check_package(&self, package: &str, version: &str) -> Vec<&Vulnerability> {
        let mut found = Vec::new();

        if let Some(advisories) = self.advisories.get(package) {
            for vuln in advisories {
                if vuln.affects_version(version) {
                    found.push(vuln);
                }
            }
        }

        found
    }

    /// Get all advisories for a package (regardless of version)
    pub fn get_advisories(&self, package: &str) -> Vec<&Vulnerability> {
        self.advisories
            .get(package)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Check if a package has any known vulnerabilities
    pub fn has_vulnerabilities(&self, package: &str) -> bool {
        self.advisories.contains_key(package)
    }

    /// Load advisories from OSV (Open Source Vulnerabilities) database
    ///
    /// OSV is Google's vulnerability database with a public API.
    /// API: https://osv.dev/api/v1/query
    ///
    /// This queries OSV for Lua package vulnerabilities.
    /// Note: OSV uses "LuaRocks" as the ecosystem identifier.
    pub async fn load_from_osv(&mut self, package_name: &str) -> LpmResult<()> {
        use reqwest;

        let client = reqwest::Client::new();
        let query = serde_json::json!({
            "version": "0",
            "package": {
                "name": package_name,
                "ecosystem": "LuaRocks"  // OSV ecosystem identifier
            }
        });

        let response = client
            .post("https://osv.dev/api/v1/query")
            .json(&query)
            .send()
            .await
            .map_err(|e| LpmError::Package(format!("Failed to query OSV API: {}", e)))?;

        if response.status().is_success() {
            let osv_response: serde_json::Value = response
                .json()
                .await
                .map_err(|e| LpmError::Package(format!("Failed to parse OSV response: {}", e)))?;

            // Parse OSV vulnerabilities
            if let Some(vulns) = osv_response.get("vulns").and_then(|v| v.as_array()) {
                for vuln in vulns {
                    if let Some(parsed) = self.parse_osv_vulnerability(vuln, package_name) {
                        self.add_advisory(parsed);
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse an OSV vulnerability entry
    fn parse_osv_vulnerability(
        &self,
        vuln: &serde_json::Value,
        package_name: &str,
    ) -> Option<Vulnerability> {
        let id = vuln.get("id")?.as_str()?;
        let summary = vuln
            .get("summary")
            .and_then(|s| s.as_str())
            .unwrap_or("Unknown vulnerability");
        let details = vuln
            .get("details")
            .and_then(|d| d.as_str())
            .unwrap_or(summary);

        // Parse severity from database_specific or use default
        let severity = vuln
            .get("database_specific")
            .and_then(|db| db.get("severity"))
            .and_then(|s| s.as_str())
            .and_then(|s| match s.to_uppercase().as_str() {
                "CRITICAL" => Some(Severity::Critical),
                "HIGH" => Some(Severity::High),
                "MEDIUM" => Some(Severity::Medium),
                "LOW" => Some(Severity::Low),
                _ => None,
            })
            .unwrap_or(Severity::Medium);

        // Parse affected versions - simplified for now
        // OSV uses complex range structures, we'll extract a simple constraint
        let affected_versions = "<999.0.0".to_string(); // Default: all versions until we parse ranges properly

        // Get fixed version
        let fixed_in = vuln
            .get("affected")
            .and_then(|a| a.as_array())
            .and_then(|a| a.first())
            .and_then(|a| a.get("ranges"))
            .and_then(|r| r.as_array())
            .and_then(|r| r.first())
            .and_then(|r| r.get("events"))
            .and_then(|e| e.as_array())
            .and_then(|e| {
                for event in e {
                    if let Some(fixed) = event.get("fixed") {
                        if let Some(version) = fixed.as_str() {
                            return Some(version.to_string());
                        }
                    }
                }
                None
            });

        // Get references
        let references = vuln
            .get("references")
            .and_then(|r| r.as_array())
            .map(|r| {
                r.iter()
                    .filter_map(|ref_obj| ref_obj.get("url").and_then(|u| u.as_str()))
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        // Extract CVE from ID if it's a CVE
        let cve = if id.starts_with("CVE-") {
            Some(id.to_string())
        } else {
            None
        };

        Some(Vulnerability {
            package: package_name.to_string(),
            affected_versions,
            severity,
            cve,
            title: summary.to_string(),
            description: details.to_string(),
            fixed_in,
            references,
        })
    }

    /// Batch load advisories for multiple packages from OSV
    pub async fn load_from_osv_batch(&mut self, packages: &[String]) -> LpmResult<()> {
        for package in packages {
            if let Err(e) = self.load_from_osv(package).await {
                eprintln!(
                    "Warning: Failed to load OSV advisories for {}: {}",
                    package, e
                );
            }
        }
        Ok(())
    }
}

impl Default for AdvisoryDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advisory_database() {
        let mut db = AdvisoryDatabase::new();

        let vuln = Vulnerability {
            package: "test-package".to_string(),
            affected_versions: "<2.0.0".to_string(),
            severity: Severity::High,
            cve: None,
            title: "Test".to_string(),
            description: "Test".to_string(),
            fixed_in: Some("2.0.0".to_string()),
            references: Vec::new(),
        };

        db.add_advisory(vuln);

        let found = db.check_package("test-package", "1.0.0");
        assert_eq!(found.len(), 1);

        let found = db.check_package("test-package", "2.0.0");
        assert_eq!(found.len(), 0);
    }
}
