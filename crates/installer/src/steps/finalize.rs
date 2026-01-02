//! Installation finalization step.

use crate::traits::{InstallStep, ProgressCallback};
use crate::Result;
use async_trait::async_trait;
use std::path::PathBuf;

/// Installation step that performs final setup tasks.
pub struct FinalizeStep {
    install_path: PathBuf,
    add_to_path: bool,
}

impl FinalizeStep {
    /// Create a new finalization step.
    pub fn new(install_path: PathBuf, add_to_path: bool) -> Self {
        Self {
            install_path,
            add_to_path,
        }
    }

    #[cfg(windows)]
    fn add_to_windows_path(&self) -> Result<()> {
        // Add to Windows PATH using registry
        // This is a placeholder - real implementation would use winreg crate
        Ok(())
    }

    #[cfg(not(windows))]
    fn add_to_unix_path(&self) -> Result<()> {
        // Add to shell profile (~/.bashrc, ~/.zshrc, etc.)
        // This is a placeholder
        Ok(())
    }

    fn write_installation_info(&self) -> Result<()> {
        let info_path = self.install_path.join("install_info.json");
        let info = serde_json::json!({
            "version": "1.0.0",
            "install_date": chrono::Utc::now().to_rfc3339(),
            "install_path": self.install_path,
        });
        std::fs::write(info_path, serde_json::to_string_pretty(&info)?)?;
        Ok(())
    }
}

#[async_trait]
impl InstallStep for FinalizeStep {
    fn name(&self) -> &str {
        "Finalize Installation"
    }

    fn description(&self) -> &str {
        "Completing installation and configuring system"
    }

    async fn execute(&self, progress: ProgressCallback) -> Result<()> {
        progress(crate::traits::Progress::new(0.0).with_message("Finalizing installation..."));

        // Write installation info
        self.write_installation_info()?;
        progress(crate::traits::Progress::new(33.0));

        // Add to PATH if requested
        if self.add_to_path {
            #[cfg(windows)]
            self.add_to_windows_path()?;

            #[cfg(not(windows))]
            self.add_to_unix_path()?;

            progress(crate::traits::Progress::new(66.0));
        }

        progress(crate::traits::Progress::new(100.0).with_message("Installation complete!"));

        Ok(())
    }
}
