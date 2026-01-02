//! # Pulsar Installer
//!
//! A modern, cross-platform installer framework for the Pulsar game engine.
//!
//! ## Features
//!
//! - **Cross-Platform**: Works seamlessly on Windows, macOS, and Linux
//! - **Modular Architecture**: Trait-based design for easy customization
//! - **Beautiful UI**: Built with GPUI component library
//! - **Progress Tracking**: Real-time installation progress with detailed feedback
//! - **Async Operations**: Non-blocking downloads and installation
//! - **Verification**: Checksum validation for downloaded files
//! - **Rollback Support**: Automatic cleanup on installation failure
//!
//! ## Architecture
//!
//! The installer is built around several core traits:
//!
//! - [`InstallStep`]: Defines individual installation steps
//! - [`SystemDetector`]: Detects system information and requirements
//! - [`DownloadManager`]: Handles file downloads with progress tracking
//! - [`ComponentInstaller`]: Installs individual components
//! - [`ConfigManager`]: Manages installation configuration

pub mod traits;
pub mod steps;
pub mod platform;
pub mod download;
pub mod config;
pub mod ui;
pub mod error;

pub use traits::*;
pub use config::InstallerConfig;
pub use error::{InstallerError, Result};
