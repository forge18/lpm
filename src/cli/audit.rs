use lpm::core::{LpmError, LpmResult};
use lpm::core::path::find_project_root;
use lpm::package::lockfile::Lockfile;
use lpm::security::audit::format_report;
use lpm::security::osv::OsvApi;
use lpm::security::vulnerability::VulnerabilityReport;
use std::env;

pub async fn run() -> LpmResult<()> {
    let current_dir = env::current_dir()
        .map_err(|e| LpmError::Path(format!("Failed to get current directory: {}", e)))?;

    let project_root = find_project_root(&current_dir)?;

    // Load lockfile
    let lockfile = Lockfile::load(&project_root)?
        .ok_or_else(|| LpmError::Package("No lockfile. Run 'lpm install' first".to_string()))?;
    
    println!("Running security audit...");
    println!("  Querying OSV (Open Source Vulnerabilities) database...");
    println!();

    // Query OSV for each package
    let osv = OsvApi::new();
    let mut report = VulnerabilityReport::new();
    report.package_count = lockfile.packages.len();
    
    for (name, locked_pkg) in &lockfile.packages {
        println!("Checking {}@{}", name, locked_pkg.version);
        let vulns = osv.query_package(name, &locked_pkg.version).await?;
        for vuln in vulns {
            report.add(vuln);
        }
        report.checked_packages += 1;
    }
    
    // Display results
    let output = format_report(&report);
    print!("{}", output);

    // Exit with error code if critical/high vulnerabilities found
    if report.has_critical() || report.has_high() {
        std::process::exit(1);
    }

    Ok(())
}

