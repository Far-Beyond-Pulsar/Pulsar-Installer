//! GitHub releases integration.

use crate::error::{InstallerError, Result};
use serde::{Deserialize, Serialize};
use gpui::http_client::{HttpClient, http, AsyncBody};
use reqwest_client::ReqwestClient;
use futures::AsyncReadExt;

/// GitHub release information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub assets: Vec<GitHubAsset>,
    pub prerelease: bool,
}

/// GitHub release asset information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// GitHub releases client.
pub struct GitHubReleases {
    client: ReqwestClient,
    owner: String,
    repo: String,
}

impl GitHubReleases {
    /// Create a new GitHub releases client.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (e.g., "pulsar-engine")
    /// * `repo` - Repository name (e.g., "pulsar")
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            client: ReqwestClient::user_agent("Pulsar-Installer/1.0").unwrap(),
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    /// Get the latest release from GitHub.
    pub async fn get_latest_release(&self) -> Result<GitHubRelease> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.owner, self.repo
        );

        let request = http::Request::builder()
            .method("GET")
            .uri(&url)
            .body(AsyncBody::default())
            .map_err(|e| InstallerError::Download(format!("Failed to build request: {}", e)))?;

        let response = self
            .client
            .send(request)
            .await
            .map_err(|e| InstallerError::Download(e.to_string()))?;

        if !response.status().is_success() {
            return Err(InstallerError::Download(format!(
                "Failed to fetch latest release: HTTP {}",
                response.status()
            )));
        }

        let mut body = response.into_body();
        let mut bytes = Vec::new();
        body.read_to_end(&mut bytes).await
            .map_err(|e| InstallerError::Download(format!("Failed to get response body: {}", e)))?;

        let release: GitHubRelease = serde_json::from_slice(&bytes)
            .map_err(|e| InstallerError::Download(format!("Failed to parse release JSON: {}", e)))?;

        Ok(release)
    }

    /// Get all releases from GitHub.
    pub async fn get_all_releases(&self) -> Result<Vec<GitHubRelease>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases",
            self.owner, self.repo
        );

        let request = http::Request::builder()
            .method("GET")
            .uri(&url)
            .body(AsyncBody::default())
            .map_err(|e| InstallerError::Download(format!("Failed to build request: {}", e)))?;

        let response = self
            .client
            .send(request)
            .await
            .map_err(|e| InstallerError::Download(e.to_string()))?;

        if !response.status().is_success() {
            return Err(InstallerError::Download(format!(
                "Failed to fetch releases: HTTP {}",
                response.status()
            )));
        }

        let mut body = response.into_body();
        let mut bytes = Vec::new();
        body.read_to_end(&mut bytes).await
            .map_err(|e| InstallerError::Download(format!("Failed to get response body: {}", e)))?;

        let releases: Vec<GitHubRelease> = serde_json::from_slice(&bytes)
            .map_err(|e| InstallerError::Download(format!("Failed to parse releases JSON: {}", e)))?;

        Ok(releases)
    }

    /// Find a binary asset for the current platform and architecture.
    ///
    /// This function looks for assets matching the pattern:
    /// `pulsar-{os}-{arch}.{ext}`
    ///
    /// Where:
    /// - `os` is "windows", "macos", or "linux"
    /// - `arch` is "x86_64" or "aarch64"
    /// - `ext` is "exe" for Windows, "tar.gz" for Unix
    pub async fn find_platform_binary(&self) -> Result<GitHubAsset> {
        let release = self.get_latest_release().await?;

        let (os_name, arch, extension) = Self::get_platform_info();

        // Try different naming patterns
        let patterns = vec![
            format!("pulsar-{}-{}.{}", os_name, arch, extension),
            format!("pulsar_{}_{}. {}", os_name, arch, extension),
            format!("{}-{}.{}", os_name, arch, extension),
            format!("{}_{}.{}", os_name, arch, extension),
        ];

        for pattern in &patterns {
            if let Some(asset) = release
                .assets
                .iter()
                .find(|a| a.name.to_lowercase().contains(&pattern.to_lowercase()))
            {
                return Ok(asset.clone());
            }
        }

        // Fallback: try to find any asset matching OS and arch
        for asset in &release.assets {
            let name_lower = asset.name.to_lowercase();
            if name_lower.contains(&os_name) && name_lower.contains(&arch) {
                return Ok(asset.clone());
            }
        }

        Err(InstallerError::Download(format!(
            "No binary found for platform: {}-{}. Available assets: {}",
            os_name,
            arch,
            release
                .assets
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )))
    }

    /// Get platform information for binary matching.
    fn get_platform_info() -> (String, String, String) {
        let os_name = if cfg!(windows) {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            "unknown"
        };

        let arch = std::env::consts::ARCH;
        let extension = if cfg!(windows) {
            "exe"
        } else {
            "tar.gz"
        };

        (os_name.to_string(), arch.to_string(), extension.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_info() {
        let (os, arch, ext) = GitHubReleases::get_platform_info();
        assert!(!os.is_empty());
        assert!(!arch.is_empty());
        assert!(!ext.is_empty());
    }
}
