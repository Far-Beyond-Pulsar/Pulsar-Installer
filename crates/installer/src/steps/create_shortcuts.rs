//! Shortcut creation step.

use crate::traits::{InstallStep, ProgressCallback};
use crate::Result;
use async_trait::async_trait;
use std::path::PathBuf;

/// Installation step that creates desktop and start menu shortcuts.
pub struct CreateShortcutsStep {
    install_path: PathBuf,
    create_desktop: bool,
    create_start_menu: bool,
}

impl CreateShortcutsStep {
    /// Create a new shortcut creation step.
    pub fn new(install_path: PathBuf, create_desktop: bool, create_start_menu: bool) -> Self {
        Self {
            install_path,
            create_desktop,
            create_start_menu,
        }
    }

    #[cfg(windows)]
    fn create_windows_shortcuts(&self) -> Result<()> {
        // Platform-specific shortcut creation would go here
        // This is a placeholder implementation
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn create_macos_shortcuts(&self) -> Result<()> {
        // Platform-specific shortcut creation would go here
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn create_linux_shortcuts(&self) -> Result<()> {
        // Platform-specific .desktop file creation would go here
        Ok(())
    }
}

#[async_trait]
impl InstallStep for CreateShortcutsStep {
    fn name(&self) -> &str {
        "Create Shortcuts"
    }

    fn description(&self) -> &str {
        "Creating desktop and menu shortcuts"
    }

    async fn can_execute(&self) -> Result<bool> {
        Ok(self.create_desktop || self.create_start_menu)
    }

    async fn execute(&self, progress: ProgressCallback) -> Result<()> {
        progress(crate::traits::Progress::new(0.0).with_message("Creating shortcuts..."));

        #[cfg(windows)]
        self.create_windows_shortcuts()?;

        #[cfg(target_os = "macos")]
        self.create_macos_shortcuts()?;

        #[cfg(target_os = "linux")]
        self.create_linux_shortcuts()?;

        progress(crate::traits::Progress::new(100.0).with_message("Shortcuts created"));

        Ok(())
    }
}
