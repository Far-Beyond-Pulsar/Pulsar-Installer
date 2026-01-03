//! Windows-specific installation implementation.
//!
//! Windows installation follows OS conventions:
//! - User-level install to %LOCALAPPDATA%\Programs\<AppName>
//! - Start Menu shortcut creation
//! - Add/Remove Programs (ARP) registry entries
//! - Proper uninstall metadata

use crate::error::{InstallerError, Result};
use crate::platform::detector::PlatformDetector;
use crate::traits::{SystemDetector, SystemRequirements, ProgressCallback, Progress};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::fs;
use winreg::enums::*;
use winreg::RegKey;

const APP_NAME: &str = "Pulsar";
const PUBLISHER: &str = "Pulsar Team";
const UNINSTALL_REGISTRY_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Pulsar";

/// Windows platform detector and installer.
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

    /// Get user-level Programs directory.
    /// Windows convention: %LOCALAPPDATA%\Programs\<AppName>
    fn get_user_programs_dir() -> PathBuf {
        std::env::var("LOCALAPPDATA")
            .map(|p| PathBuf::from(p).join("Programs"))
            .unwrap_or_else(|_| {
                dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Local"))
                    .join("Programs")
            })
    }

    /// Get Start Menu directory for current user.
    fn get_start_menu_dir() -> PathBuf {
        std::env::var("APPDATA")
            .map(|p| PathBuf::from(p).join("Microsoft\\Windows\\Start Menu\\Programs"))
            .unwrap_or_else(|_| {
                dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Roaming"))
                    .join("Microsoft\\Windows\\Start Menu\\Programs")
            })
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
        Self::get_user_programs_dir().join(APP_NAME)
    }

    async fn validate_install_path(&self, path: &Path) -> Result<()> {
        PlatformDetector::validate_path_impl(path).await
    }
}

/// Windows-specific installation operations.
pub struct WindowsInstaller {
    install_path: PathBuf,
    version: String,
}

impl WindowsInstaller {
    /// Create a new Windows installer.
    pub fn new(install_path: PathBuf, version: String) -> Self {
        Self {
            install_path,
            version,
        }
    }

    /// Install the application to Windows.
    /// 
    /// This performs the following:
    /// 1. Copies files to %LOCALAPPDATA%\Programs\Pulsar
    /// 2. Creates Start Menu shortcut
    /// 3. Registers in Add/Remove Programs
    pub async fn install(&self, progress: ProgressCallback) -> Result<()> {
        progress(Progress::new(0.0).with_message("Starting Windows installation..."));

        // Files should already be copied by extract step
        // We focus on OS-specific registration here
        
        progress(Progress::new(30.0).with_message("Creating Start Menu shortcut..."));
        self.create_start_menu_shortcut()?;

        progress(Progress::new(60.0).with_message("Registering in Add/Remove Programs..."));
        self.register_arp()?;

        progress(Progress::new(90.0).with_message("Writing uninstall metadata..."));
        self.write_uninstall_metadata()?;

        progress(Progress::new(100.0).with_message("Windows installation complete"));

        Ok(())
    }

    /// Create Start Menu shortcut.
    /// Location: %APPDATA%\Microsoft\Windows\Start Menu\Programs\Pulsar.lnk
    fn create_start_menu_shortcut(&self) -> Result<()> {
        let start_menu_dir = WindowsDetector::get_start_menu_dir();
        fs::create_dir_all(&start_menu_dir)?;

        let shortcut_path = start_menu_dir.join(format!("{}.lnk", APP_NAME));
        let exe_path = self.install_path.join("pulsar.exe");

        // Windows requires COM for .lnk creation
        // For now, we'll use powershell as a fallback
        self.create_shortcut_via_powershell(&shortcut_path, &exe_path)?;

        Ok(())
    }

    /// Create shortcut using PowerShell COM interface.
    fn create_shortcut_via_powershell(&self, shortcut_path: &Path, target_path: &Path) -> Result<()> {
        let ps_script = format!(
            r#"$WScript = New-Object -ComObject WScript.Shell; $Shortcut = $WScript.CreateShortcut('{}'); $Shortcut.TargetPath = '{}'; $Shortcut.WorkingDirectory = '{}'; $Shortcut.Save()"#,
            shortcut_path.display(),
            target_path.display(),
            self.install_path.display()
        );

        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_script])
            .output()?;

        if !output.status.success() {
            return Err(InstallerError::Platform(
                format!("Failed to create shortcut: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }

        Ok(())
    }

    /// Register in Add/Remove Programs.
    /// Registry location: HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Pulsar
    fn register_arp(&self) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu.create_subkey(UNINSTALL_REGISTRY_KEY)?;

        let exe_path = self.install_path.join("pulsar.exe");
        let uninstall_path = self.install_path.join("uninstall.exe");

        // Required registry values for Add/Remove Programs
        key.set_value("DisplayName", &APP_NAME)?;
        key.set_value("DisplayVersion", &self.version)?;
        key.set_value("Publisher", &PUBLISHER)?;
        key.set_value("InstallLocation", &self.install_path.to_string_lossy().as_ref())?;
        key.set_value("UninstallString", &format!("\"{}\"", uninstall_path.display()))?;
        key.set_value("DisplayIcon", &exe_path.to_string_lossy().as_ref())?;
        key.set_value("NoModify", &1u32)?;
        key.set_value("NoRepair", &1u32)?;

        // Optional: estimate size
        if let Ok(size) = self.estimate_install_size() {
            key.set_value("EstimatedSize", &(size / 1024))?; // Size in KB
        }

        Ok(())
    }

    /// Write uninstall metadata for easy cleanup.
    fn write_uninstall_metadata(&self) -> Result<()> {
        let metadata = serde_json::json!({
            "app_name": APP_NAME,
            "version": self.version,
            "install_path": self.install_path,
            "start_menu_shortcut": WindowsDetector::get_start_menu_dir().join(format!("{}.lnk", APP_NAME)),
            "registry_key": UNINSTALL_REGISTRY_KEY,
            "install_date": chrono::Utc::now().to_rfc3339(),
        });

        let metadata_path = self.install_path.join("uninstall_metadata.json");
        fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;

        Ok(())
    }

    /// Estimate installation size in bytes.
    fn estimate_install_size(&self) -> Result<u64> {
        let mut total_size = 0u64;
        
        if let Ok(entries) = fs::read_dir(&self.install_path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }

        Ok(total_size)
    }

    /// Uninstall the application from Windows.
    /// 
    /// Removes:
    /// - Installed files
    /// - Start Menu shortcut
    /// - Registry entries
    pub async fn uninstall(&self, progress: ProgressCallback) -> Result<()> {
        progress(Progress::new(0.0).with_message("Starting Windows uninstallation..."));

        progress(Progress::new(25.0).with_message("Removing Start Menu shortcut..."));
        self.remove_start_menu_shortcut()?;

        progress(Progress::new(50.0).with_message("Unregistering from Add/Remove Programs..."));
        self.unregister_arp()?;

        progress(Progress::new(75.0).with_message("Removing files..."));
        fs::remove_dir_all(&self.install_path)?;

        progress(Progress::new(100.0).with_message("Windows uninstallation complete"));

        Ok(())
    }

    /// Remove Start Menu shortcut.
    fn remove_start_menu_shortcut(&self) -> Result<()> {
        let shortcut_path = WindowsDetector::get_start_menu_dir().join(format!("{}.lnk", APP_NAME));
        
        if shortcut_path.exists() {
            fs::remove_file(shortcut_path)?;
        }

        Ok(())
    }

    /// Unregister from Add/Remove Programs.
    fn unregister_arp(&self) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        
        // Delete the entire uninstall key
        if let Err(e) = hkcu.delete_subkey_all(UNINSTALL_REGISTRY_KEY) {
            // Only error if key exists but deletion failed
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(e.into());
            }
        }

        Ok(())
    }
}
