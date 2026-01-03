//! macOS-specific installation implementation.
//!
//! macOS installation follows OS conventions:
//! - Creates a proper .app bundle with Info.plist
//! - User install to ~/Applications/<AppName>.app
//! - System install option to /Applications/<AppName>.app
//! - Relies on Launch Services for registration (no manual database manipulation)

#![cfg(target_os = "macos")]

use crate::error::{InstallerError, Result};
use crate::platform::detector::PlatformDetector;
use crate::traits::{SystemDetector, SystemRequirements, ProgressCallback, Progress};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::fs;
use plist::Value;

const APP_NAME: &str = "Pulsar";
const BUNDLE_IDENTIFIER: &str = "com.pulsarteam.pulsar";

/// macOS platform detector and installer.
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

    /// Get user Applications directory.
    /// macOS convention: ~/Applications for user, /Applications for system
    fn get_user_applications_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/Users/Default"))
            .join("Applications")
    }

    /// Get system Applications directory.
    fn get_system_applications_dir() -> PathBuf {
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
        // Default to user-level install
        Self::get_user_applications_dir().join(format!("{}.app", APP_NAME))
    }

    async fn validate_install_path(&self, path: &Path) -> Result<()> {
        PlatformDetector::validate_path_impl(path).await
    }
}

/// macOS-specific installation operations.
pub struct MacOSInstaller {
    app_bundle_path: PathBuf,
    version: String,
    binary_name: String,
}

impl MacOSInstaller {
    /// Create a new macOS installer.
    pub fn new(app_bundle_path: PathBuf, version: String, binary_name: String) -> Self {
        Self {
            app_bundle_path,
            version,
            binary_name,
        }
    }

    /// Install the application to macOS.
    /// 
    /// Creates a valid .app bundle structure:
    /// Pulsar.app/
    ///   Contents/
    ///     Info.plist
    ///     MacOS/pulsar
    ///     Resources/
    pub async fn install(&self, source_binary: &Path, progress: ProgressCallback) -> Result<()> {
        progress(Progress::new(0.0).with_message("Starting macOS installation..."));

        progress(Progress::new(20.0).with_message("Creating app bundle structure..."));
        self.create_app_bundle_structure()?;

        progress(Progress::new(40.0).with_message("Copying binary..."));
        self.install_binary(source_binary)?;

        progress(Progress::new(60.0).with_message("Creating Info.plist..."));
        self.create_info_plist()?;

        progress(Progress::new(80.0).with_message("Setting permissions..."));
        self.set_permissions()?;

        progress(Progress::new(90.0).with_message("Writing uninstall metadata..."));
        self.write_uninstall_metadata()?;

        progress(Progress::new(100.0).with_message("macOS installation complete"));

        // Note: Launch Services will automatically pick up the app bundle
        // No manual registration required - this is the macOS way

        Ok(())
    }

    /// Create the .app bundle directory structure.
    fn create_app_bundle_structure(&self) -> Result<()> {
        let contents_dir = self.app_bundle_path.join("Contents");
        let macos_dir = contents_dir.join("MacOS");
        let resources_dir = contents_dir.join("Resources");

        fs::create_dir_all(&macos_dir)?;
        fs::create_dir_all(&resources_dir)?;

        Ok(())
    }

    /// Copy binary to Contents/MacOS/.
    fn install_binary(&self, source_binary: &Path) -> Result<()> {
        let dest_binary = self.app_bundle_path
            .join("Contents")
            .join("MacOS")
            .join(&self.binary_name);

        fs::copy(source_binary, &dest_binary)?;

        Ok(())
    }

    /// Create Info.plist with required metadata.
    /// 
    /// This is critical for Launch Services to recognize the app.
    fn create_info_plist(&self) -> Result<()> {
        let plist_path = self.app_bundle_path.join("Contents").join("Info.plist");

        // Build Info.plist dictionary
        let mut dict = plist::Dictionary::new();
        
        // Required keys for Launch Services
        dict.insert("CFBundleIdentifier".to_string(), Value::String(BUNDLE_IDENTIFIER.to_string()));
        dict.insert("CFBundleName".to_string(), Value::String(APP_NAME.to_string()));
        dict.insert("CFBundleDisplayName".to_string(), Value::String(APP_NAME.to_string()));
        dict.insert("CFBundleExecutable".to_string(), Value::String(self.binary_name.clone()));
        dict.insert("CFBundleVersion".to_string(), Value::String(self.version.clone()));
        dict.insert("CFBundleShortVersionString".to_string(), Value::String(self.version.clone()));
        dict.insert("CFBundlePackageType".to_string(), Value::String("APPL".to_string()));
        dict.insert("CFBundleSignature".to_string(), Value::String("????".to_string()));
        dict.insert("LSMinimumSystemVersion".to_string(), Value::String("11.0".to_string()));
        dict.insert("NSHighResolutionCapable".to_string(), Value::Boolean(true));

        // Write plist
        let value = Value::Dictionary(dict);
        value.to_file_xml(&plist_path)?;

        Ok(())
    }

    /// Set executable permissions on the binary.
    fn set_permissions(&self) -> Result<()> {
        let binary_path = self.app_bundle_path
            .join("Contents")
            .join("MacOS")
            .join(&self.binary_name);

        // Make binary executable
        let output = std::process::Command::new("chmod")
            .args(["+x", &binary_path.to_string_lossy()])
            .output()?;

        if !output.status.success() {
            return Err(InstallerError::Platform(
                format!("Failed to set executable permissions: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }

        Ok(())
    }

    /// Write uninstall metadata.
    fn write_uninstall_metadata(&self) -> Result<()> {
        let metadata = serde_json::json!({
            "app_name": APP_NAME,
            "bundle_identifier": BUNDLE_IDENTIFIER,
            "version": self.version,
            "app_bundle_path": self.app_bundle_path,
            "install_date": chrono::Utc::now().to_rfc3339(),
        });

        let metadata_path = self.app_bundle_path.join("Contents").join("uninstall_metadata.json");
        fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;

        Ok(())
    }

    /// Uninstall the application from macOS.
    /// 
    /// Simply removes the .app bundle.
    /// Launch Services will automatically detect removal.
    pub async fn uninstall(&self, progress: ProgressCallback) -> Result<()> {
        progress(Progress::new(0.0).with_message("Starting macOS uninstallation..."));

        progress(Progress::new(50.0).with_message("Removing app bundle..."));
        fs::remove_dir_all(&self.app_bundle_path)?;

        progress(Progress::new(100.0).with_message("macOS uninstallation complete"));

        // Launch Services automatically handles app removal
        // No manual cleanup of databases required

        Ok(())
    }
}
