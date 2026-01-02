//! Main installer application.

use super::*;
use crate::config::InstallerConfig;
use crate::platform;
use gpui::{
    App, AppContext, Context, Div, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, View, Window, px,
};
use gpui_component::{ActiveTheme, Root, v_flex};
use std::sync::Arc;

/// Installer page navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerPage {
    Welcome,
    License,
    PathSelection,
    Components,
    Installation,
    Complete,
}

/// Main installer application state.
pub struct InstallerApp {
    current_page: InstallerPage,
    config: Arc<InstallerConfig>,
    focus_handle: FocusHandle,
}

impl InstallerApp {
    /// Create a new installer application.
    pub fn new(cx: &mut App) -> View<Self> {
        let detector = platform::get_system_detector();
        let default_path = detector.default_install_path();
        let config = Arc::new(InstallerConfig::new(default_path));

        cx.new_view(|cx: &mut App| Self {
            current_page: InstallerPage::Welcome,
            config,
            focus_handle: cx.focus_handle(),
        })
    }

    /// Navigate to a specific page.
    pub fn navigate_to(&mut self, page: InstallerPage, _window: &mut Window, _cx: &mut App) {
        self.current_page = page;
    }

    /// Get the current page.
    pub fn current_page(&self) -> InstallerPage {
        self.current_page
    }

    /// Render the current page content.
    fn render_page(&self, window: &mut Window, cx: &mut App) -> Div {
        match self.current_page {
            InstallerPage::Welcome => {
                let view_cx = cx.view().clone();
                div().child(WelcomeView::new(
                    move |window, cx| {
                        view_cx.update(window, cx, |app, window, cx| {
                            app.navigate_to(InstallerPage::License, window, cx);
                        });
                    },
                    cx,
                ))
            }
            InstallerPage::License => {
                let view_cx_back = cx.view().clone();
                let view_cx_next = cx.view().clone();
                div().child(LicenseView::new(
                    move |window, cx| {
                        view_cx_back.update(window, cx, |app, window, cx| {
                            app.navigate_to(InstallerPage::Welcome, window, cx);
                        });
                    },
                    move |window, cx| {
                        view_cx_next.update(window, cx, |app, window, cx| {
                            app.navigate_to(InstallerPage::PathSelection, window, cx);
                        });
                    },
                    cx,
                ))
            }
            InstallerPage::PathSelection => {
                let view_cx_back = cx.view().clone();
                let view_cx_next = cx.view().clone();
                let default_path = self.config.install_path().to_path_buf();
                div().child(PathSelectionView::new(
                    default_path,
                    move |window, cx| {
                        view_cx_back.update(window, cx, |app, window, cx| {
                            app.navigate_to(InstallerPage::License, window, cx);
                        });
                    },
                    move |window, cx| {
                        view_cx_next.update(window, cx, |app, window, cx| {
                            app.navigate_to(InstallerPage::Components, window, cx);
                        });
                    },
                    cx,
                ))
            }
            InstallerPage::Components => {
                let view_cx_back = cx.view().clone();
                let view_cx_next = cx.view().clone();
                div().child(ComponentsView::new(
                    move |window, cx| {
                        view_cx_back.update(window, cx, |app, window, cx| {
                            app.navigate_to(InstallerPage::PathSelection, window, cx);
                        });
                    },
                    move |window, cx| {
                        view_cx_next.update(window, cx, |app, window, cx| {
                            app.navigate_to(InstallerPage::Installation, window, cx);
                        });
                    },
                    cx,
                ))
            }
            InstallerPage::Installation => div().child(InstallationView::new(cx)),
            InstallerPage::Complete => {
                let view_cx = cx.view().clone();
                div().child(CompleteView::new(
                    move |window, cx| {
                        // Close the application
                        view_cx.update(window, cx, |_app, _window, cx| {
                            cx.quit();
                        });
                    },
                    cx,
                ))
            }
        }
    }
}

impl EventEmitter<()> for InstallerApp {}

impl Focusable for InstallerApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for InstallerApp {
    fn render(&mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        Root::new()
            .size_full()
            .child(
                v_flex()
                    .size_full()
                    .bg(cx.theme().background)
                    .child(self.render_page(window, cx)),
            )
    }
}
