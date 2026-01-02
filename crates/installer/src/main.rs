//! Pulsar Engine Installer
//!
//! Downloads and installs Pulsar engine from GitHub releases.

use gpui::{App, AppContext, Bounds, Size, WindowBounds, WindowKind, WindowOptions, px, size};
use pulsar_installer::ui::InstallerView;
use gpui_component::Root;

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

        let window_size = size(px(800.0), px(600.0));
        let window_bounds = Bounds::centered(None, window_size, cx);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(window_bounds)),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("Pulsar Installer".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            window_min_size: Some(Size {
                width: px(600.0),
                height: px(480.0),
            }),
            kind: WindowKind::Normal,
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            // Create the installer view
            let installer_view = InstallerView::view(window, cx);

            // Focus the installer view
            let focus_handle = installer_view.focus_handle(cx);
            window.defer(cx, move |window, cx| {
                focus_handle.focus(window, cx);
            });

            // Wrap in Root following the story crate pattern
            cx.new(|cx| Root::new(installer_view, window, cx))
        })
        .expect("Failed to open installer window");
    });
}
