//! Error types for the Pulsar installer.

use std::path::PathBuf;

/// Result type alias for installer operations.
pub type Result<T> = std::result::Result<T, InstallerError>;

/// Errors that can occur during installation.
#[derive(Debug, thiserror::Error)]
pub enum InstallerError {
    /// I/O error occurred
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Download failed
    #[error("Download failed: {0}")]
    Download(String),

    /// Checksum verification failed
    #[error("Checksum verification failed for {file}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        file: String,
        expected: String,
        actual: String,
    },

    /// Insufficient disk space
    #[error("Insufficient disk space: need {needed} bytes, available {available} bytes")]
    InsufficientSpace { needed: u64, available: u64 },

    /// System requirements not met
    #[error("System requirements not met: {0}")]
    RequirementsNotMet(String),

    /// Installation path is invalid
    #[error("Invalid installation path: {0}")]
    InvalidPath(PathBuf),

    /// Component installation failed
    #[error("Failed to install component '{component}': {reason}")]
    ComponentFailed { component: String, reason: String },

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Platform not supported
    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}
