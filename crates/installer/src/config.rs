//! Installation configuration management.

use crate::error::{InstallerError, Result};
use crate::traits::{ConfigManager, SystemRequirements};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main configuration for the Pulsar installer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerConfig {
    /// Installation path
    pub install_path: PathBuf,
    /// Selected component IDs
    pub selected_components: Vec<String>,
    /// Create desktop shortcut
    pub create_desktop_shortcut: bool,
    /// Create start menu shortcut
    pub create_start_menu_shortcut: bool,
    /// Add to PATH environment variable
    pub add_to_path: bool,
    /// System requirements
    pub requirements: SystemRequirements,
    /// Total installation size in bytes
    #[serde(skip)]
    total_size: u64,
}

impl InstallerConfig {
    /// Create a new installer configuration with default values.
    pub fn new(default_path: PathBuf) -> Self {
        Self {
            install_path: default_path,
            selected_components: Vec::new(),
            create_desktop_shortcut: true,
            create_start_menu_shortcut: true,
            add_to_path: true,
            requirements: SystemRequirements::default_requirements(),
            total_size: 0,
        }
    }

    /// Calculate total installation size based on selected components.
    pub fn calculate_total_size(&mut self, components: &[&dyn crate::traits::ComponentInstaller]) {
        self.total_size = components
            .iter()
            .filter(|c| self.selected_components.contains(&c.id().to_string()))
            .map(|c| c.size_bytes())
            .sum();
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.selected_components.is_empty() {
            return Err(InstallerError::Config(
                "No components selected for installation".to_string(),
            ));
        }

        if !self.install_path.is_absolute() {
            return Err(InstallerError::InvalidPath(self.install_path.clone()));
        }

        Ok(())
    }
}

impl ConfigManager for InstallerConfig {
    fn install_path(&self) -> &Path {
        &self.install_path
    }

    fn set_install_path(&mut self, path: PathBuf) {
        self.install_path = path;
    }

    fn selected_components(&self) -> &[String] {
        &self.selected_components
    }

    fn set_selected_components(&mut self, components: Vec<String>) {
        self.selected_components = components;
    }

    fn total_size(&self) -> u64 {
        self.total_size
    }

    fn save(&self) -> Result<()> {
        // In a real implementation, this would save to a file
        // For now, we'll just validate
        self.validate()
    }

    fn load() -> Result<Self> {
        // In a real implementation, this would load from a file
        Err(InstallerError::Config(
            "Config file not found".to_string(),
        ))
    }
}

impl SystemRequirements {
    /// Get default system requirements for Pulsar.
    pub fn default_requirements() -> Self {
        Self {
            min_disk_space: 2 * 1024 * 1024 * 1024, // 2 GB
            min_ram_mb: Some(4096),                  // 4 GB RAM
            os_versions: vec![
                "Windows 10+".to_string(),
                "macOS 11+".to_string(),
                "Linux (kernel 5.0+)".to_string(),
            ],
            architectures: vec!["x86_64".to_string(), "aarch64".to_string()],
        }
    }
}
