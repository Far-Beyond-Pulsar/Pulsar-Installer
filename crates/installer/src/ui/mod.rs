//! Installer user interface components.
//!
//! This module contains all UI views for the installer application,
//! built using the GPUI component library.

mod welcome;
mod license;
mod path_selection;
mod components;
mod installation;
mod complete;
mod app;

pub use welcome::WelcomeView;
pub use license::LicenseView;
pub use path_selection::PathSelectionView;
pub use components::ComponentsView;
pub use installation::InstallationView;
pub use complete::CompleteView;
pub use app::{InstallerApp, InstallerPage};
