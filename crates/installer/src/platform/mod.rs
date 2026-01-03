//! Platform-specific implementations.

mod detector;
mod windows;
mod macos;
mod linux;

pub use detector::PlatformDetector;

#[cfg(windows)]
pub use windows::{WindowsDetector, WindowsInstaller};

#[cfg(target_os = "macos")]
pub use macos::{MacOSDetector, MacOSInstaller};

#[cfg(target_os = "linux")]
pub use linux::{LinuxDetector, LinuxInstaller};

use crate::traits::SystemDetector;
use std::sync::Arc;

/// Get the appropriate system detector for the current platform.
pub fn get_system_detector() -> Arc<dyn SystemDetector> {
    #[cfg(windows)]
    return Arc::new(windows::WindowsDetector::new());

    #[cfg(target_os = "macos")]
    return Arc::new(macos::MacOSDetector::new());

    #[cfg(target_os = "linux")]
    return Arc::new(linux::LinuxDetector::new());

    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    compile_error!("Unsupported platform");
}
