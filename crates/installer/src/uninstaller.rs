//! Cross-platform uninstaller implementation.
//!
//! Provides OS-appropriate uninstallation for each platform:
//! - Windows: Removes files, Start Menu shortcuts, and registry entries
//! - macOS: Removes .app bundle (Launch Services auto-updates)
//! - Linux: Removes binary, desktop entry, and icons

use crate::error::{InstallerError, Result};
use crate::traits::{ProgressCallback, Progress};
use std::path::{Path, PathBuf};

#[cfg(windows)]
use crate::platform::WindowsInstaller;

#[cfg(target_os = "macos")]
use crate::platform::MacOSInstaller;

#[cfg(target_os = "linux")]
use crate::platform::LinuxInstaller;

/// Cross-platform uninstaller.
pub struct Uninstaller {
    install_path: PathBuf,
    version: String,
}

impl Uninstaller {
    /// Create a new uninstaller.
    pub fn new(install_path: PathBuf, version: String) -> Self {
        Self {
            install_path,
            version,
        }
    }

    /// Load uninstaller from metadata file.
    pub fn from_metadata(metadata_path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(metadata_path)?;
        let metadata: serde_json::Value = serde_json::from_str(&content)?;

        let install_path = metadata["install_path"]
            .as_str()
            .ok_or_else(|| InstallerError::Config("Missing install_path in metadata".to_string()))?;

        let version = metadata["version"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            install_path: PathBuf::from(install_path),
            version,
        })
    }

    /// Uninstall the application.
    pub async fn uninstall(self, progress: ProgressCallback) -> Result<()> {
        progress(Progress::new(0.0).with_message("Starting uninstallation..."));

        #[cfg(windows)]
        self.uninstall_windows(progress).await?;

        #[cfg(target_os = "macos")]
        self.uninstall_macos(progress).await?;

        #[cfg(target_os = "linux")]
        self.uninstall_linux(progress).await?;

        Ok(())
    }

    #[cfg(windows)]
    async fn uninstall_windows(self, progress: ProgressCallback) -> Result<()> {
        let installer = WindowsInstaller::new(
            self.install_path,
            self.version,
        );
        installer.uninstall(progress).await
    }

    #[cfg(target_os = "macos")]
    async fn uninstall_macos(self, progress: ProgressCallback) -> Result<()> {
        let installer = MacOSInstaller::new(
            self.install_path,
            self.version,
            "pulsar".to_string(),
        );
        installer.uninstall(progress).await
    }

    #[cfg(target_os = "linux")]
    async fn uninstall_linux(self, progress: ProgressCallback) -> Result<()> {
        // Detect if it was a system install
        let is_system = self.install_path.starts_with("/usr");
        
        let installer = LinuxInstaller::new(
            self.version,
            is_system,
        );
        installer.uninstall(progress).await
    }
}

