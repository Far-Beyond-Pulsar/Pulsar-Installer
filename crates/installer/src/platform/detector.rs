//! Base platform detector with common functionality.

use crate::error::{InstallerError, Result};
use crate::traits::{SystemDetector, SystemRequirements};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Base platform detector with common functionality.
pub struct PlatformDetector {
    os_name: String,
    architecture: String,
}

impl PlatformDetector {
    /// Create a new platform detector.
    pub fn new(os_name: String, architecture: String) -> Self {
        Self {
            os_name,
            architecture,
        }
    }

    /// Get current architecture string.
    pub fn detect_architecture() -> String {
        std::env::consts::ARCH.to_string()
    }

    /// Check if path has write permissions.
    pub async fn check_write_permission(path: &Path) -> Result<bool> {
        if !path.exists() {
            // Try to create parent directory to test permissions
            if let Some(parent) = path.parent() {
                if parent.exists() {
                    return Ok(parent.metadata()?.permissions().readonly() == false);
                }
            }
            return Ok(false);
        }

        Ok(path.metadata()?.permissions().readonly() == false)
    }

    /// Get available disk space at path.
    pub async fn get_available_space_impl(path: &Path) -> Result<u64> {
        // This is a simplified version - real implementation would use platform-specific APIs
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = std::fs::metadata(path)?;
            // This is a placeholder - real implementation would use statvfs
            Ok(metadata.size() * 100) // Dummy value
        }

        #[cfg(windows)]
        {
            // Use GetDiskFreeSpaceEx on Windows
            Ok(10 * 1024 * 1024 * 1024) // Dummy: 10 GB
        }

        #[cfg(not(any(unix, windows)))]
        Ok(10 * 1024 * 1024 * 1024) // Dummy: 10 GB
    }

    /// Validate installation path.
    pub async fn validate_path_impl(path: &Path) -> Result<()> {
        // Check if path is absolute
        if !path.is_absolute() {
            return Err(InstallerError::InvalidPath(path.to_path_buf()));
        }

        // Check if path is writable
        if path.exists() {
            if !Self::check_write_permission(path).await? {
                return Err(InstallerError::Other(format!(
                    "No write permission for path: {}",
                    path.display()
                )));
            }
        }

        Ok(())
    }

    /// Check system requirements.
    pub async fn check_requirements_impl(
        &self,
        path: &Path,
        requirements: &SystemRequirements,
    ) -> Result<()> {
        // Check disk space
        let available = Self::get_available_space_impl(path).await?;
        if available < requirements.min_disk_space {
            return Err(InstallerError::InsufficientSpace {
                needed: requirements.min_disk_space,
                available,
            });
        }

        // Check architecture
        if !requirements.architectures.contains(&self.architecture) {
            return Err(InstallerError::RequirementsNotMet(format!(
                "Architecture {} is not supported. Supported: {}",
                self.architecture,
                requirements.architectures.join(", ")
            )));
        }

        Ok(())
    }
}
