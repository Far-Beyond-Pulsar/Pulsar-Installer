//! macOS-specific platform detection.

use crate::error::Result;
use crate::platform::detector::PlatformDetector;
use crate::traits::{SystemDetector, SystemRequirements};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// macOS platform detector.
pub struct MacOSDetector {
    base: PlatformDetector,
}

impl MacOSDetector {
    /// Create a new macOS detector.
    pub fn new() -> Self {
        Self {
            base: PlatformDetector::new(
                "macOS".to_string(),
                PlatformDetector::detect_architecture(),
            ),
        }
    }

    /// Get Applications directory.
    fn get_applications_dir() -> PathBuf {
        PathBuf::from("/Applications")
    }
}

impl Default for MacOSDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemDetector for MacOSDetector {
    fn os_name(&self) -> &str {
        "macOS"
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
        Self::get_applications_dir().join("Pulsar.app")
    }

    async fn validate_install_path(&self, path: &Path) -> Result<()> {
        PlatformDetector::validate_path_impl(path).await
    }
}
