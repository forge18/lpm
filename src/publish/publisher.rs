use crate::core::credentials::CredentialStore;
use crate::core::{LpmError, LpmResult};
use crate::package::manifest::PackageManifest;
use crate::publish::packager::PublishPackager;
use crate::publish::rockspec_generator::RockspecGenerator;
use crate::publish::validator::PublishValidator;
use std::fs;
use std::path::{Path, PathBuf};

/// Publishes Lua modules to LuaRocks
pub struct Publisher {
    project_root: PathBuf,
    manifest: PackageManifest,
}

impl Publisher {
    /// Create a new publisher
    pub fn new(project_root: &Path, manifest: PackageManifest) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            manifest,
        }
    }

    /// Publish the package to LuaRocks
    pub async fn publish(&self, include_binaries: bool) -> LpmResult<()> {
        // 1. Validate package
        println!("Validating package...");
        PublishValidator::validate(&self.manifest, &self.project_root)?;

        // 2. Check for LuaRocks credentials
        let username = CredentialStore::retrieve("luarocks_username").map_err(|_| {
            LpmError::Package("LuaRocks username not found. Run 'lpm login' first.".to_string())
        })?;

        let api_key = CredentialStore::retrieve("luarocks_api_key").map_err(|_| {
            LpmError::Package("LuaRocks API key not found. Run 'lpm login' first.".to_string())
        })?;

        println!("Publishing as: {}", username);

        // 3. Generate rockspec
        println!("Generating rockspec...");
        let rockspec_content = RockspecGenerator::generate(&self.manifest)?;
        let rockspec_path = self.project_root.join(format!(
            "{}-{}.rockspec",
            self.manifest.name,
            crate::luarocks::version::to_luarocks_version(&crate::core::version::Version::parse(
                &self.manifest.version
            )?)
        ));
        fs::write(&rockspec_path, rockspec_content)?;
        println!("✓ Generated rockspec: {}", rockspec_path.display());

        // 4. Package the module
        println!("Packaging module...");
        let packager = PublishPackager::new(&self.project_root, self.manifest.clone());
        let archive_path = packager.package(include_binaries)?;

        // 5. Upload to LuaRocks
        println!("Uploading to LuaRocks...");
        self.upload_to_luarocks(&rockspec_path, &archive_path, &username, &api_key)
            .await?;

        println!("✓ Published successfully!");

        Ok(())
    }

    /// Upload package to LuaRocks API
    async fn upload_to_luarocks(
        &self,
        rockspec_path: &Path,
        archive_path: &Path,
        username: &str,
        api_key: &str,
    ) -> LpmResult<()> {
        // LuaRocks API endpoint for uploading
        let api_url = "https://luarocks.org/api/upload";

        // Create multipart form data
        use reqwest::multipart;
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;

        let mut rockspec_file = File::open(rockspec_path)
            .await
            .map_err(|e| LpmError::Package(format!("Failed to open rockspec: {}", e)))?;
        let mut archive_file = File::open(archive_path)
            .await
            .map_err(|e| LpmError::Package(format!("Failed to open archive: {}", e)))?;

        let mut rockspec_bytes = Vec::new();
        rockspec_file
            .read_to_end(&mut rockspec_bytes)
            .await
            .map_err(|e| LpmError::Package(format!("Failed to read rockspec: {}", e)))?;

        let mut archive_bytes = Vec::new();
        archive_file
            .read_to_end(&mut archive_bytes)
            .await
            .map_err(|e| LpmError::Package(format!("Failed to read archive: {}", e)))?;

        let rockspec_part = multipart::Part::bytes(rockspec_bytes)
            .file_name(
                rockspec_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            )
            .mime_str("text/x-lua")
            .map_err(|e| LpmError::Package(format!("Failed to create multipart part: {}", e)))?;

        let archive_part = multipart::Part::bytes(archive_bytes)
            .file_name(
                archive_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            )
            .mime_str("application/gzip")
            .map_err(|e| LpmError::Package(format!("Failed to create multipart part: {}", e)))?;

        let form = multipart::Form::new()
            .text("username", username.to_string())
            .text("api_key", api_key.to_string())
            .part("rockspec", rockspec_part)
            .part("archive", archive_part);

        let client = reqwest::Client::new();
        let response = client
            .post(api_url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| LpmError::Package(format!("Failed to upload to LuaRocks: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LpmError::Package(format!(
                "Failed to upload to LuaRocks: HTTP {} - {}",
                status, body
            )));
        }

        Ok(())
    }
}
