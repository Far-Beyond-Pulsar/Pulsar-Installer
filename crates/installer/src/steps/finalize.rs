//! Installation finalization step.
//!
//! This step performs final cleanup and metadata writing.
//! Does NOT modify shell profiles or PATH - that's optional and separate.

use crate::traits::{InstallStep, ProgressCallback, Progress};
use crate::Result;
use async_trait::async_trait;
use std::path::PathBuf;

/// Installation step that performs final setup tasks.
pub struct FinalizeStep {
    install_path: PathBuf,
}

impl FinalizeStep {
    /// Create a new finalization step.
    pub fn new(install_path: PathBuf) -> Self {
        Self {
            install_path,
        }
    }

    /// Write installation completion metadata.
    fn write_installation_info(&self) -> Result<()> {
        let info_path = self.install_path.join("install_info.json");
        let info = serde_json::json!({
            "version": "1.0.0",
            "install_date": chrono::Utc::now().to_rfc3339(),
            "install_path": self.install_path,
            "platform": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
        });
        std::fs::write(info_path, serde_json::to_string_pretty(&info)?)?;
        Ok(())
    }

    /// Verify installation integrity.
    fn verify_installation(&self) -> Result<()> {
        // Check that install directory exists
        if !self.install_path.exists() {
            return Err(crate::error::InstallerError::InvalidPath(self.install_path.clone()));
        }

        // Platform-specific verification
        #[cfg(windows)]
        {
            let exe = self.install_path.join("pulsar.exe");
            if !exe.exists() {
                return Err(crate::error::InstallerError::InvalidPath(exe));
            }
        }

        #[cfg(target_os = "macos")]
        {
            let contents = self.install_path.join("Contents");
            if !contents.exists() {
                return Err(crate::error::InstallerError::InvalidPath(contents));
            }
        }

        #[cfg(target_os = "linux")]
        {
            // For Linux, install_path might be the binary itself or a directory
            // Just check it exists
            if !self.install_path.exists() {
                return Err(crate::error::InstallerError::InvalidPath(self.install_path.clone()));
            }
        }

        Ok(())
    }
}

#[async_trait]
impl InstallStep for FinalizeStep {
    fn name(&self) -> &str {
        "Finalize Installation"
    }

    fn description(&self) -> &str {
        "Completing installation and verifying integrity"
    }

    async fn execute(&self, progress: ProgressCallback) -> Result<()> {
        progress(Progress::new(0.0).with_message("Finalizing installation..."));

        progress(Progress::new(33.0).with_message("Verifying installation..."));
        self.verify_installation()?;

        progress(Progress::new(66.0).with_message("Writing metadata..."));
        self.write_installation_info()?;

        progress(Progress::new(100.0).with_message("Installation complete!"));

        Ok(())
    }
}
