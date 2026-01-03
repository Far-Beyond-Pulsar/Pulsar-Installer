//! Linux-specific installation implementation.
//!
//! Linux installation follows freedesktop.org conventions:
//! - Binary installation to ~/.local/bin (user) or /usr/bin (system)
//! - Desktop entry creation at ~/.local/share/applications/<appname>.desktop
//! - Icon installation to ~/.local/share/icons/hicolor/<size>/apps/
//! - Relies on desktop environment indexing (no manual cache manipulation)

use crate::error::Result;
use crate::platform::detector::PlatformDetector;
use crate::traits::{SystemDetector, SystemRequirements, ProgressCallback, Progress};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const APP_NAME: &str = "Pulsar";
const APP_NAME_LOWER: &str = "pulsar";
const DESKTOP_ENTRY_NAME: &str = "pulsar.desktop";

/// Linux platform detector and installer.
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

    /// Get user's local binary directory.
    /// freedesktop.org convention: ~/.local/bin
    fn get_user_bin_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/home/default"))
            .join(".local")
            .join("bin")
    }

    /// Get user's local applications directory.
    /// freedesktop.org convention: ~/.local/share/applications
    fn get_user_applications_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("/home/default"))
                    .join(".local")
                    .join("share")
            })
            .join("applications")
    }

    /// Get user's local icon directory.
    fn get_user_icon_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("/home/default"))
                    .join(".local")
                    .join("share")
            })
            .join("icons")
            .join("hicolor")
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
        // For Linux, we return the binary path, not a directory
        Self::get_user_bin_dir().join(APP_NAME_LOWER)
    }

    async fn validate_install_path(&self, path: &Path) -> Result<()> {
        PlatformDetector::validate_path_impl(path).await
    }
}

/// Linux-specific installation operations.
pub struct LinuxInstaller {
    binary_path: PathBuf,
    version: String,
    use_system_directories: bool,
}

impl LinuxInstaller {
    /// Create a new Linux installer.
    /// 
    /// # Arguments
    /// 
    /// * `use_system_directories` - If true, install to /usr/bin instead of ~/.local/bin
    pub fn new(version: String, use_system_directories: bool) -> Self {
        let binary_path = if use_system_directories {
            PathBuf::from("/usr/bin").join(APP_NAME_LOWER)
        } else {
            LinuxDetector::get_user_bin_dir().join(APP_NAME_LOWER)
        };

        Self {
            binary_path,
            version,
            use_system_directories,
        }
    }

    /// Install the application to Linux.
    /// 
    /// Performs:
    /// 1. Binary installation to ~/.local/bin or /usr/bin
    /// 2. Desktop entry creation
    /// 3. Icon installation (if available)
    pub async fn install(&self, source_binary: &Path, progress: ProgressCallback) -> Result<()> {
        progress(Progress::new(0.0).with_message("Starting Linux installation..."));

        progress(Progress::new(20.0).with_message("Installing binary..."));
        self.install_binary(source_binary)?;

        progress(Progress::new(40.0).with_message("Creating desktop entry..."));
        self.create_desktop_entry()?;

        progress(Progress::new(70.0).with_message("Installing icons..."));
        self.install_icons()?;

        progress(Progress::new(85.0).with_message("Updating desktop database..."));
        self.update_desktop_database();

        progress(Progress::new(95.0).with_message("Writing uninstall metadata..."));
        self.write_uninstall_metadata()?;

        progress(Progress::new(100.0).with_message("Linux installation complete"));

        Ok(())
    }

    /// Install binary to the appropriate directory.
    fn install_binary(&self, source_binary: &Path) -> Result<()> {
        // Create parent directory if needed
        if let Some(parent) = self.binary_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy binary
        fs::copy(source_binary, &self.binary_path)?;

        // Make executable (chmod +x)
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&self.binary_path)?.permissions();
            perms.set_mode(0o755); // rwxr-xr-x
            fs::set_permissions(&self.binary_path, perms)?;
        }

        Ok(())
    }

    /// Create .desktop file following freedesktop.org specification.
    fn create_desktop_entry(&self) -> Result<()> {
        let desktop_dir = if self.use_system_directories {
            PathBuf::from("/usr/share/applications")
        } else {
            LinuxDetector::get_user_applications_dir()
        };

        fs::create_dir_all(&desktop_dir)?;

        let desktop_file_path = desktop_dir.join(DESKTOP_ENTRY_NAME);

        // Create desktop entry content
        let desktop_entry = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name={}\n\
             Comment=Pulsar Game Engine Installer\n\
             Exec={}\n\
             Icon={}\n\
             Terminal=false\n\
             Categories=Development;Game;\n\
             Keywords=pulsar;game;engine;installer;\n\
             Version={}\n",
            APP_NAME,
            self.binary_path.display(),
            APP_NAME_LOWER,
            self.version
        );

        fs::write(desktop_file_path, desktop_entry)?;

        Ok(())
    }

    /// Install icons to hicolor icon theme directories.
    /// 
    /// This follows the freedesktop.org icon theme specification.
    /// Icons should be placed in: ~/.local/share/icons/hicolor/<size>/apps/<appname>.png
    fn install_icons(&self) -> Result<()> {
        let icon_base_dir = if self.use_system_directories {
            PathBuf::from("/usr/share/icons/hicolor")
        } else {
            LinuxDetector::get_user_icon_dir()
        };

        // Common icon sizes: 16, 22, 24, 32, 48, 64, 128, 256, 512
        let icon_sizes = ["16x16", "32x32", "48x48", "64x64", "128x128", "256x256"];

        for size in &icon_sizes {
            let icon_dir = icon_base_dir.join(size).join("apps");
            fs::create_dir_all(&icon_dir)?;

            // If icon files exist in the source, copy them
            // For now, we'll just create the directory structure
            // Real implementation would copy actual icon files
        }

        Ok(())
    }

    /// Update desktop database if the utility is available.
    /// 
    /// This is optional - desktop environments will eventually pick up changes.
    /// We call it for immediate effect if available.
    fn update_desktop_database(&self) {
        let desktop_dir = if self.use_system_directories {
            PathBuf::from("/usr/share/applications")
        } else {
            LinuxDetector::get_user_applications_dir()
        };

        // Try to run update-desktop-database, but don't fail if it's not available
        let _ = std::process::Command::new("update-desktop-database")
            .arg(desktop_dir)
            .output();

        // Also try to update icon cache if available
        let icon_dir = if self.use_system_directories {
            PathBuf::from("/usr/share/icons/hicolor")
        } else {
            LinuxDetector::get_user_icon_dir()
        };

        let _ = std::process::Command::new("gtk-update-icon-cache")
            .arg(icon_dir)
            .output();
    }

    /// Write uninstall metadata.
    fn write_uninstall_metadata(&self) -> Result<()> {
        let metadata_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("pulsar");

        fs::create_dir_all(&metadata_dir)?;

        let desktop_dir = if self.use_system_directories {
            PathBuf::from("/usr/share/applications")
        } else {
            LinuxDetector::get_user_applications_dir()
        };

        let metadata = serde_json::json!({
            "app_name": APP_NAME,
            "version": self.version,
            "binary_path": self.binary_path,
            "desktop_entry": desktop_dir.join(DESKTOP_ENTRY_NAME),
            "icon_dir": if self.use_system_directories {
                PathBuf::from("/usr/share/icons/hicolor")
            } else {
                LinuxDetector::get_user_icon_dir()
            },
            "system_install": self.use_system_directories,
            "install_date": chrono::Utc::now().to_rfc3339(),
        });

        let metadata_path = metadata_dir.join("uninstall_metadata.json");
        fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;

        Ok(())
    }

    /// Uninstall the application from Linux.
    /// 
    /// Removes:
    /// - Binary
    /// - Desktop entry
    /// - Icons
    pub async fn uninstall(&self, progress: ProgressCallback) -> Result<()> {
        progress(Progress::new(0.0).with_message("Starting Linux uninstallation..."));

        progress(Progress::new(25.0).with_message("Removing binary..."));
        if self.binary_path.exists() {
            fs::remove_file(&self.binary_path)?;
        }

        progress(Progress::new(50.0).with_message("Removing desktop entry..."));
        self.remove_desktop_entry()?;

        progress(Progress::new(75.0).with_message("Removing icons..."));
        self.remove_icons()?;

        progress(Progress::new(90.0).with_message("Updating desktop database..."));
        self.update_desktop_database();

        progress(Progress::new(100.0).with_message("Linux uninstallation complete"));

        Ok(())
    }

    /// Remove desktop entry.
    fn remove_desktop_entry(&self) -> Result<()> {
        let desktop_dir = if self.use_system_directories {
            PathBuf::from("/usr/share/applications")
        } else {
            LinuxDetector::get_user_applications_dir()
        };

        let desktop_file = desktop_dir.join(DESKTOP_ENTRY_NAME);
        if desktop_file.exists() {
            fs::remove_file(desktop_file)?;
        }

        Ok(())
    }

    /// Remove icons.
    fn remove_icons(&self) -> Result<()> {
        let icon_base_dir = if self.use_system_directories {
            PathBuf::from("/usr/share/icons/hicolor")
        } else {
            LinuxDetector::get_user_icon_dir()
        };

        let icon_sizes = ["16x16", "32x32", "48x48", "64x64", "128x128", "256x256"];

        for size in &icon_sizes {
            let icon_file = icon_base_dir
                .join(size)
                .join("apps")
                .join(format!("{}.png", APP_NAME_LOWER));

            if icon_file.exists() {
                fs::remove_file(icon_file)?;
            }
        }

        Ok(())
    }
}
