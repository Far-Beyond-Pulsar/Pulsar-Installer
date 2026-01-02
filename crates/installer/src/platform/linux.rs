//! Linux-specific platform detection.

use crate::error::Result;
use crate::platform::detector::PlatformDetector;
use crate::traits::{SystemDetector, SystemRequirements};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Linux platform detector.
pub struct LinuxDetector {
    base: PlatformDetector,
}

impl LinuxDetector {
    /// Create a new Linux detector.
    pub fn new() -> Self {
        Self {
            base: PlatformDetector::new(
                "Linux".to_string(),
                PlatformDetector::detect_architecture(),
            ),
        }
    }

    /// Get user's local installation directory.
    fn get_local_dir() -> PathBuf {
        std::env::var("HOME")
            .map(|home| PathBuf::from(home).join(".local").join("share"))
            .unwrap_or_else(|_| PathBuf::from("/usr/local/share"))
    }
}

impl Default for LinuxDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemDetector for LinuxDetector {
    fn os_name(&self) -> &str {
        "Linux"
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
        Self::get_local_dir().join("pulsar")
    }

    async fn validate_install_path(&self, path: &Path) -> Result<()> {
        PlatformDetector::validate_path_impl(path).await
    }
}
