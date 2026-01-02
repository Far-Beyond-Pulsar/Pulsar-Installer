//! Core traits defining the installer's modular architecture.
//!
//! This module contains the fundamental traits that enable the installer's
//! flexibility and extensibility. Each trait represents a distinct concern
//! in the installation process.

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Represents the progress of an operation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Progress {
    /// Current progress value (0.0 - 100.0)
    pub current: f32,
    /// Total size in bytes (if known)
    pub total_bytes: Option<u64>,
    /// Downloaded/processed bytes
    pub processed_bytes: u64,
    /// Current operation message
    pub message: Option<&'static str>,
}

impl Progress {
    /// Create a new progress indicator.
    pub fn new(current: f32) -> Self {
        Self {
            current: current.clamp(0.0, 100.0),
            total_bytes: None,
            processed_bytes: 0,
            message: None,
        }
    }

    /// Set the total bytes.
    pub fn with_total_bytes(mut self, total: u64) -> Self {
        self.total_bytes = Some(total);
        self
    }

    /// Set the processed bytes.
    pub fn with_processed_bytes(mut self, processed: u64) -> Self {
        self.processed_bytes = processed;
        self
    }

    /// Set the message.
    pub fn with_message(mut self, message: &'static str) -> Self {
        self.message = Some(message);
        self
    }
}

/// Callback type for progress updates.
pub type ProgressCallback = Box<dyn Fn(Progress) + Send + Sync>;

/// Represents a single installation step.
///
/// Installation steps are executed sequentially and can report progress.
/// Each step should be idempotent and support rollback on failure.
#[async_trait]
pub trait InstallStep: Send + Sync {
    /// Get a human-readable name for this step.
    fn name(&self) -> &str;

    /// Get a description of what this step does.
    fn description(&self) -> &str;

    /// Check if this step can be executed.
    ///
    /// This allows steps to be skipped based on configuration or system state.
    async fn can_execute(&self) -> Result<bool> {
        Ok(true)
    }

    /// Execute the installation step.
    ///
    /// # Arguments
    ///
    /// * `progress` - Callback to report progress (0.0 - 100.0)
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the step failed.
    async fn execute(&self, progress: ProgressCallback) -> Result<()>;

    /// Rollback this step if installation fails.
    ///
    /// This should attempt to undo any changes made by `execute()`.
    async fn rollback(&self) -> Result<()> {
        Ok(())
    }
}

/// System information and requirements detection.
///
/// Implementors detect system properties like OS, architecture,
/// available disk space, and verify system requirements.
#[async_trait]
pub trait SystemDetector: Send + Sync {
    /// Get the operating system name.
    fn os_name(&self) -> &str;

    /// Get the system architecture (e.g., "x86_64", "aarch64").
    fn architecture(&self) -> &str;

    /// Get available disk space at the specified path in bytes.
    async fn available_space(&self, path: &Path) -> Result<u64>;

    /// Check if system requirements are met.
    ///
    /// # Arguments
    ///
    /// * `requirements` - The requirements to check
    ///
    /// # Returns
    ///
    /// `Ok(())` if requirements are met, or an error describing what's missing.
    async fn check_requirements(&self, requirements: &SystemRequirements) -> Result<()>;

    /// Get the default installation path for this platform.
    fn default_install_path(&self) -> PathBuf;

    /// Validate that a path is suitable for installation.
    async fn validate_install_path(&self, path: &Path) -> Result<()>;
}

/// System requirements specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRequirements {
    /// Minimum required disk space in bytes
    pub min_disk_space: u64,
    /// Minimum RAM in megabytes
    pub min_ram_mb: Option<u32>,
    /// Required OS versions (e.g., "Windows 10+", "macOS 11+")
    pub os_versions: Vec<String>,
    /// Supported architectures
    pub architectures: Vec<String>,
}

/// File download management with progress tracking.
///
/// Handles downloading files from remote sources with progress callbacks,
/// verification, and error handling.
#[async_trait]
pub trait DownloadManager: Send + Sync {
    /// Download a file from a URL to a destination path.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to download from
    /// * `destination` - Where to save the file
    /// * `progress` - Callback for progress updates
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if download fails.
    async fn download(
        &self,
        url: &str,
        destination: &Path,
        progress: ProgressCallback,
    ) -> Result<()>;

    /// Download and verify a file using a checksum.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to download from
    /// * `destination` - Where to save the file
    /// * `expected_checksum` - Expected SHA256 checksum in hex format
    /// * `progress` - Callback for progress updates
    async fn download_with_verification(
        &self,
        url: &str,
        destination: &Path,
        expected_checksum: &str,
        progress: ProgressCallback,
    ) -> Result<()>;

    /// Get the total size of a remote file without downloading it.
    async fn get_file_size(&self, url: &str) -> Result<u64>;
}

/// Component installation handler.
///
/// Components represent optional or required parts of the installation,
/// such as the game engine, tools, documentation, or sample projects.
#[async_trait]
pub trait ComponentInstaller: Send + Sync {
    /// Get the component's unique identifier.
    fn id(&self) -> &str;

    /// Get the component's display name.
    fn name(&self) -> &str;

    /// Get the component's description.
    fn description(&self) -> &str;

    /// Get the size of this component in bytes.
    fn size_bytes(&self) -> u64;

    /// Check if this component is required.
    fn is_required(&self) -> bool {
        false
    }

    /// Install this component.
    ///
    /// # Arguments
    ///
    /// * `install_path` - The base installation directory
    /// * `progress` - Callback for progress updates
    async fn install(&self, install_path: &Path, progress: ProgressCallback) -> Result<()>;

    /// Uninstall this component.
    async fn uninstall(&self, install_path: &Path) -> Result<()>;

    /// Verify the component installation.
    async fn verify(&self, install_path: &Path) -> Result<bool>;
}

/// Installation configuration management.
///
/// Manages user preferences and installation settings.
pub trait ConfigManager: Send + Sync {
    /// Get the installation path.
    fn install_path(&self) -> &Path;

    /// Set the installation path.
    fn set_install_path(&mut self, path: PathBuf);

    /// Get selected components.
    fn selected_components(&self) -> &[String];

    /// Set selected components.
    fn set_selected_components(&mut self, components: Vec<String>);

    /// Check if a component is selected.
    fn is_component_selected(&self, component_id: &str) -> bool {
        self.selected_components().contains(&component_id.to_string())
    }

    /// Get total installation size in bytes.
    fn total_size(&self) -> u64;

    /// Save configuration to disk.
    fn save(&self) -> Result<()>;

    /// Load configuration from disk.
    fn load() -> Result<Self>
    where
        Self: Sized;
}

/// Installation state tracker.
///
/// Tracks the current state of the installation process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallState {
    /// Installation not started
    NotStarted,
    /// Checking system requirements
    CheckingRequirements,
    /// Downloading files
    Downloading,
    /// Installing components
    Installing,
    /// Verifying installation
    Verifying,
    /// Installation completed successfully
    Completed,
    /// Installation failed
    Failed,
    /// Installation cancelled by user
    Cancelled,
}

impl InstallState {
    /// Check if the installation is in progress.
    pub fn is_in_progress(&self) -> bool {
        matches!(
            self,
            Self::CheckingRequirements | Self::Downloading | Self::Installing | Self::Verifying
        )
    }

    /// Check if the installation is finished (either completed, failed, or cancelled).
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}
