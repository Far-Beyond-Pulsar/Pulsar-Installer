//! OS registration and integration step.
//!
//! This step performs OS-specific registration:
//! - Windows: Creates Start Menu shortcuts and registers in Add/Remove Programs
//! - macOS: Creates .app bundle with Info.plist, relies on Launch Services
//! - Linux: Creates .desktop entry and installs icons per freedesktop.org spec

use crate::traits::{InstallStep, ProgressCallback};
use crate::Result;
use async_trait::async_trait;
use std::path::PathBuf;

#[cfg(windows)]
use crate::platform::WindowsInstaller;

#[cfg(target_os = "macos")]
use crate::platform::MacOSInstaller;

#[cfg(target_os = "linux")]
use crate::platform::LinuxInstaller;

/// Installation step that performs OS-specific registration and integration.
/// 
/// Each platform handles this differently:
/// - Windows: Registry entries + Start Menu
/// - macOS: App bundle creation
/// - Linux: Desktop entry + icons
pub struct CreateShortcutsStep {
    install_path: PathBuf,
    version: String,
    #[cfg(target_os = "linux")]
    use_system_directories: bool,
}

impl CreateShortcutsStep {
    /// Create a new OS registration step.
    #[cfg(not(target_os = "linux"))]
    pub fn new(install_path: PathBuf, version: String) -> Self {
        Self {
            install_path,
            version,
        }
    }

    /// Create a new OS registration step (Linux variant with system install option).
    #[cfg(target_os = "linux")]
    pub fn new(install_path: PathBuf, version: String, use_system_directories: bool) -> Self {
        Self {
            install_path,
            version,
            use_system_directories,
        }
    }

    #[cfg(windows)]
    async fn register_windows(&self, progress: ProgressCallback) -> Result<()> {
        let installer = WindowsInstaller::new(
            self.install_path.clone(),
            self.version.clone(),
        );
        installer.install(progress).await
    }

    #[cfg(target_os = "macos")]
    async fn register_macos(&self, progress: ProgressCallback) -> Result<()> {
        // For macOS, the install_path should be the .app bundle
        // Binary is assumed to be at <install_path>/Contents/MacOS/pulsar
        let binary_name = "pulsar".to_string();
        let source_binary = self.install_path.join("Contents").join("MacOS").join(&binary_name);
        
        let installer = MacOSInstaller::new(
            self.install_path.clone(),
            self.version.clone(),
            binary_name,
        );
        
        // If binary already exists (from extract step), we're just creating metadata
        // Otherwise, we need to know the source binary location
        installer.install(&source_binary, progress).await
    }

    #[cfg(target_os = "linux")]
    async fn register_linux(&self, progress: ProgressCallback) -> Result<()> {
        let installer = LinuxInstaller::new(
            self.version.clone(),
            self.use_system_directories,
        );
        
        // Assume binary is already in place from extract step
        // We're just creating desktop integration
        let source_binary = self.install_path.join("pulsar");
        installer.install(&source_binary, progress).await
    }
}

#[async_trait]
impl InstallStep for CreateShortcutsStep {
    fn name(&self) -> &str {
        "OS Integration"
    }

    fn description(&self) -> &str {
        #[cfg(windows)]
        return "Registering with Windows (Start Menu, Add/Remove Programs)";
        
        #[cfg(target_os = "macos")]
        return "Creating macOS app bundle and registering with Launch Services";
        
        #[cfg(target_os = "linux")]
        return "Creating desktop entry and installing icons (freedesktop.org)";
    }

    async fn can_execute(&self) -> Result<bool> {
        Ok(true)
    }

    async fn execute(&self, progress: ProgressCallback) -> Result<()> {
        #[cfg(windows)]
        self.register_windows(progress).await?;

        #[cfg(target_os = "macos")]
        self.register_macos(progress).await?;

        #[cfg(target_os = "linux")]
        self.register_linux(progress).await?;

        Ok(())
    }

    async fn rollback(&self) -> Result<()> {
        // Platform-specific rollback would go here
        // For now, we rely on uninstall functionality
        Ok(())
    }
}
