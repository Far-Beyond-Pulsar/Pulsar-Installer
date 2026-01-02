//! Windows-specific platform detection.

use crate::error::Result;
use crate::platform::detector::PlatformDetector;
use crate::traits::{SystemDetector, SystemRequirements};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Windows platform detector.
pub struct WindowsDetector {
    base: PlatformDetector,
}

impl WindowsDetector {
    /// Create a new Windows detector.
    pub fn new() -> Self {
        Self {
            base: PlatformDetector::new(
                "Windows".to_string(),
                PlatformDetector::detect_architecture(),
            ),
        }
    }

    /// Get Program Files directory.
    fn get_program_files_dir() -> PathBuf {
        std::env::var("ProgramFiles")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\Program Files"))
    }
}

impl Default for WindowsDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemDetector for WindowsDetector {
    fn os_name(&self) -> &str {
        "Windows"
    }

    fn architecture(&self) -> &str {
        std::env::consts::ARCH
    }

    async fn available_space(&self, path: &Path) -> Result<u64> {
        PlatformDetector::get_available_space_impl(path).await
    }

    async fn check_requirements(&self, requirements: &SystemRequirements) -> Result<()> {
        self.base
            .check_requirements_impl(&self.default_install_path(), requirements)
            .await
    }

    fn default_install_path(&self) -> PathBuf {
        Self::get_program_files_dir().join("Pulsar")
    }

    async fn validate_install_path(&self, path: &Path) -> Result<()> {
        PlatformDetector::validate_path_impl(path).await
    }
}
