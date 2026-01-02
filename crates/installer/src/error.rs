//! Error types for the Pulsar installer.

use std::path::PathBuf;

/// Result type alias for installer operations.
pub type Result<T> = std::result::Result<T, InstallerError>;

/// Errors that can occur during installation.
#[derive(Debug)]
pub enum InstallerError {
    /// I/O error occurred
    Io(std::io::Error),

    /// Download failed
    Download(String),

    /// Checksum verification failed
    ChecksumMismatch {
        file: String,
        expected: String,
        actual: String,
    },

    /// Insufficient disk space
    InsufficientSpace { needed: u64, available: u64 },

    /// System requirements not met
    RequirementsNotMet(String),

    /// Installation path is invalid
    InvalidPath(PathBuf),

    /// Component installation failed
    ComponentFailed { component: String, reason: String },

    /// Configuration error
    Config(String),

    /// Platform not supported
    UnsupportedPlatform(String),

    /// JSON error
    Json(String),

    /// Generic error
    Other(String),
}

impl std::fmt::Display for InstallerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Download(s) => write!(f, "Download failed: {}", s),
            Self::ChecksumMismatch { file, expected, actual } => {
                write!(f, "Checksum mismatch for {}: expected {}, got {}", file, expected, actual)
            }
            Self::InsufficientSpace { needed, available } => {
                write!(f, "Insufficient disk space: need {} bytes, available {} bytes", needed, available)
            }
            Self::RequirementsNotMet(s) => write!(f, "System requirements not met: {}", s),
            Self::InvalidPath(p) => write!(f, "Invalid installation path: {}", p.display()),
            Self::ComponentFailed { component, reason } => {
                write!(f, "Failed to install component '{}': {}", component, reason)
            }
            Self::Config(s) => write!(f, "Configuration error: {}", s),
            Self::UnsupportedPlatform(s) => write!(f, "Platform not supported: {}", s),
            Self::Json(s) => write!(f, "JSON error: {}", s),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for InstallerError {}

impl From<std::io::Error> for InstallerError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for InstallerError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e.to_string())
    }
}
