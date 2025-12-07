use crate::core::{LpmError, LpmResult};
use crate::package::lockfile::Lockfile;
use crate::security::advisory::AdvisoryDatabase;
use crate::security::vulnerability::{Severity, Vulnerability, VulnerabilityReport};
use std::path::Path;

/// Security auditor for checking package vulnerabilities
pub struct SecurityAuditor {
    advisory_db: AdvisoryDatabase,
}

impl SecurityAuditor {
    /// Create a new security auditor
    pub fn new() -> LpmResult<Self> {
        let advisory_db = AdvisoryDatabase::load()?;
        Ok(Self { advisory_db })
    }

    /// Create a new security auditor with OSV integration
    /// 
    /// This will query OSV for vulnerabilities in the provided packages.
    pub async fn new_with_osv(packages: &[String]) -> LpmResult<Self> {
        let mut advisory_db = AdvisoryDatabase::load()?;
        
        // Load from OSV
        advisory_db.load_from_osv_batch(packages).await?;
        
        Ok(Self { advisory_db })
    }

    /// Run a security audit on the current project
    pub fn audit_project(project_root: &Path) -> LpmResult<VulnerabilityReport> {
        let auditor = Self::new()?;
        auditor.audit(project_root)
    }

    /// Run a security audit with OSV integration
    pub async fn audit_project_with_osv(project_root: &Path) -> LpmResult<VulnerabilityReport> {
        // Load lockfile to get package names
        let lockfile = crate::package::lockfile::Lockfile::load(project_root)?
            .ok_or_else(|| LpmError::Package("No package.lock found. Run 'lpm install' first.".to_string()))?;
        
        let package_names: Vec<String> = lockfile.packages.keys().cloned().collect();
        
        let auditor = Self::new_with_osv(&package_names).await?;
        auditor.audit(project_root)
    }

    /// Perform security audit
    fn audit(&self, project_root: &Path) -> LpmResult<VulnerabilityReport> {
        // Load lockfile to get installed packages
        let lockfile = Lockfile::load(project_root)?
            .ok_or_else(|| LpmError::Package("No package.lock found. Run 'lpm install' first.".to_string()))?;
        
        let mut report = VulnerabilityReport::new();
        report.package_count = lockfile.packages.len();
        
        // Check each package for vulnerabilities
        for (package_name, package_info) in &lockfile.packages {
            report.checked_packages += 1;
            
            // Check against advisory database
            let vulnerabilities = self.advisory_db.check_package(
                package_name,
                &package_info.version,
            );
            
            for vuln in vulnerabilities {
                report.add(vuln.clone());
            }
        }
        
        Ok(report)
    }

    /// Check a specific package for vulnerabilities
    pub fn check_package(&self, package: &str, version: &str) -> Vec<&Vulnerability> {
        self.advisory_db.check_package(package, version)
    }

    /// Get all known advisories for a package
    pub fn get_advisories(&self, package: &str) -> Vec<&Vulnerability> {
        self.advisory_db.get_advisories(package)
    }
}

/// Format vulnerability report for display
pub fn format_report(report: &VulnerabilityReport) -> String {
    use std::fmt::Write;
    
    let mut output = String::new();
    
    if report.is_empty() {
        writeln!(output, "✓ No known vulnerabilities found").unwrap();
        writeln!(output, "  Checked {} package(s)", report.checked_packages).unwrap();
        return output;
    }
    
    // Sort vulnerabilities by severity (critical first)
    let mut vulns = report.vulnerabilities.clone();
    vulns.sort_by(|a, b| b.severity.cmp(&a.severity));
    
    // Count by severity
    let counts = report.count_by_severity();
    
    writeln!(output, "\n🚨 Security Audit Results").unwrap();
    writeln!(output, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").unwrap();
    writeln!(output, "Checked: {} package(s)", report.checked_packages).unwrap();
    writeln!(output, "Found: {} vulnerability(ies)", report.vulnerabilities.len()).unwrap();
    writeln!(output).unwrap();
    
    // Summary by severity
    if let Some(count) = counts.get(&Severity::Critical) {
        writeln!(output, "  {} Critical: {}", Severity::Critical.emoji(), count).unwrap();
    }
    if let Some(count) = counts.get(&Severity::High) {
        writeln!(output, "  {} High: {}", Severity::High.emoji(), count).unwrap();
    }
    if let Some(count) = counts.get(&Severity::Medium) {
        writeln!(output, "  {} Medium: {}", Severity::Medium.emoji(), count).unwrap();
    }
    if let Some(count) = counts.get(&Severity::Low) {
        writeln!(output, "  {} Low: {}", Severity::Low.emoji(), count).unwrap();
    }
    
    writeln!(output).unwrap();
    writeln!(output, "Vulnerabilities:").unwrap();
    writeln!(output).unwrap();
    
    // List each vulnerability
    for (i, vuln) in vulns.iter().enumerate() {
        writeln!(
            output,
            "{}. {} {} {}",
            i + 1,
            vuln.severity.emoji(),
            vuln.severity.as_str(),
            vuln.package
        ).unwrap();
        writeln!(output, "   Package: {}@{}", vuln.package, vuln.affected_versions).unwrap();
        writeln!(output, "   Title: {}", vuln.title).unwrap();
        
        if let Some(ref cve) = vuln.cve {
            writeln!(output, "   CVE: {}", cve).unwrap();
        }
        
        if let Some(ref fixed_in) = vuln.fixed_in {
            writeln!(output, "   Fixed in: {}", fixed_in).unwrap();
        }
        
        writeln!(output, "   Description: {}", vuln.description).unwrap();
        
        if !vuln.references.is_empty() {
            writeln!(output, "   References:").unwrap();
            for ref_link in &vuln.references {
                writeln!(output, "     - {}", ref_link).unwrap();
            }
        }
        
        writeln!(output).unwrap();
    }
    
    // Recommendations
    writeln!(output, "Recommendations:").unwrap();
    if report.has_critical() || report.has_high() {
        writeln!(output, "  • Update vulnerable packages immediately").unwrap();
        writeln!(output, "  • Review and test updates before deploying").unwrap();
    } else {
        writeln!(output, "  • Consider updating packages to latest versions").unwrap();
    }
    writeln!(output, "  • Run 'lpm outdated' to see available updates").unwrap();
    writeln!(output, "  • Run 'lpm update <package>' to update specific packages").unwrap();
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_empty_report() {
        let report = VulnerabilityReport::new();
        let output = format_report(&report);
        assert!(output.contains("No known vulnerabilities"));
    }
}

