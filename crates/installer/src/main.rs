//! Pulsar Game Engine Installer
//!
//! A modern, cross-platform installer built with GPUI.

use gpui::{App, AppContext, size, px};
use pulsar_installer::ui::InstallerApp;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting Pulsar Installer");

    // Create and run the GPUI application
    App::new().run(|cx: &mut AppContext| {
        // Initialize GPUI components
        gpui_component::init(cx);

        // Set default window bounds
        let window_size = size(px(900.0), px(700.0));

        // Create the main installer window
        cx.open_window(
            gpui::WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds {
                    origin: gpui::Point::default(),
                    size: window_size,
                })),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Pulsar Installer".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                window_background: gpui::WindowBackgroundAppearance::default(),
                focus: true,
                show: true,
                kind: gpui::WindowKind::Normal,
                is_movable: true,
                fullscreen: None,
                window_min_size: Some(size(px(800.0), px(600.0))),
                ..Default::default()
            },
            |window, cx| {
                // Create the installer app view
                let app = InstallerApp::new(cx);
                window.focus(&app.focus_handle(cx), cx);
                app
            },
        );
    });
}
