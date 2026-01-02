//! HTTP download manager implementation.

use crate::error::{InstallerError, Result};
use crate::traits::{DownloadManager, Progress, ProgressCallback};
use async_trait::async_trait;
use futures::AsyncWriteExt;
use gpui::http_client::{HttpClient, http, AsyncBody};
use reqwest_client::ReqwestClient;
use std::path::Path;

/// HTTP-based download manager.
pub struct HttpDownloadManager {
    client: ReqwestClient,
}

impl HttpDownloadManager {
    /// Create a new HTTP download manager.
    pub fn new() -> Self {
        Self {
            client: ReqwestClient::user_agent("Pulsar-Installer/1.0").unwrap(),
        }
    }

    /// Download a file with progress tracking.
    async fn download_impl(
        &self,
        url: &str,
        destination: &Path,
        progress: ProgressCallback,
    ) -> Result<()> {
        // Send HTTP request
        let request = http::Request::builder()
            .method("GET")
            .uri(url)
            .body(AsyncBody::default())
            .map_err(|e| InstallerError::Download(format!("Failed to build request: {}", e)))?;

        let response = self
            .client
            .send(request)
            .await
            .map_err(|e| InstallerError::Download(e.to_string()))?;

        if !response.status().is_success() {
            return Err(InstallerError::Download(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        // Get total file size
        let total_size = response.headers().get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        // Create destination file
        let mut file = smol::fs::File::create(destination)
            .await
            .map_err(|e| InstallerError::Io(e))?;

        // Download with progress tracking
        let mut downloaded: u64 = 0;
        let mut body = response.into_body();
        let mut buffer = vec![0u8; 8192];

        loop {
            use futures::AsyncReadExt;
            let n = body.read(&mut buffer)
                .await
                .map_err(|e| InstallerError::Download(format!("Failed to read response: {}", e)))?;

            if n == 0 {
                break;
            }

            file.write_all(&buffer[..n])
                .await
                .map_err(|e| InstallerError::Io(e))?;

            downloaded += n as u64;

            let percent = if total_size > 0 {
                (downloaded as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };

            progress(
                Progress::new(percent)
                    .with_total_bytes(total_size)
                    .with_processed_bytes(downloaded),
            );
        }

        file.flush().await.map_err(|e| InstallerError::Io(e))?;

        Ok(())
    }
}

impl Default for HttpDownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DownloadManager for HttpDownloadManager {
    async fn download(
        &self,
        url: &str,
        destination: &Path,
        progress: ProgressCallback,
    ) -> Result<()> {
        self.download_impl(url, destination, progress).await
    }

    async fn download_with_verification(
        &self,
        url: &str,
        destination: &Path,
        expected_checksum: &str,
        progress: ProgressCallback,
    ) -> Result<()> {
        // Download the file
        self.download_impl(url, destination, progress).await?;

        // Verify checksum
        let verifier = super::FileVerifier::new();
        verifier.verify_sha256(destination, expected_checksum).await?;

        Ok(())
    }

    async fn get_file_size(&self, url: &str) -> Result<u64> {
        let request = http::Request::builder()
            .method("HEAD")
            .uri(url)
            .body(AsyncBody::default())
            .map_err(|e| InstallerError::Download(format!("Failed to build request: {}", e)))?;

        let response = self
            .client
            .send(request)
            .await
            .map_err(|e| InstallerError::Download(e.to_string()))?;

        response.headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or_else(|| InstallerError::Download("Content-Length header missing".to_string()))
    }
}

// Note: This implementation uses reqwest which requires tokio runtime
// The actual installer UI will need to spawn tasks appropriately
