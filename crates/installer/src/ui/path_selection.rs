//! Installation path selection view.

use gpui::{
    div, prelude::FluentBuilder as _, App, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, px, Entity,
};
use gpui_component::{
    ActiveTheme, h_flex, v_flex,
    button::{Button, ButtonVariants as _},
    input::Input,
};
use std::path::PathBuf;

/// Installation path selection view.
pub struct PathSelectionView {
    install_path: Entity<SharedString>,
    default_path: PathBuf,
    on_back: Entity<Box<dyn Fn(&mut Window, &mut App)>>,
    on_next: Entity<Box<dyn Fn(&mut Window, &mut App)>>,
}

impl PathSelectionView {
    /// Create a new path selection view.
    pub fn new(
        default_path: PathBuf,
        on_back: impl Fn(&mut Window, &mut App) + 'static,
        on_next: impl Fn(&mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Self {
        let path_str: SharedString = default_path.display().to_string().into();
        Self {
            install_path: cx.new(|_| path_str),
            default_path,
            on_back: cx.new(|_| Box::new(on_back) as Box<dyn Fn(&mut Window, &mut App)>),
            on_next: cx.new(|_| Box::new(on_next) as Box<dyn Fn(&mut Window, &mut App)>),
        }
    }

    fn calculate_required_space() -> String {
        "2.5 GB".to_string()
    }

    fn calculate_available_space() -> String {
        "50 GB".to_string() // Placeholder
    }
}

impl RenderOnce for PathSelectionView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let on_back = self.on_back;
        let on_next = self.on_next;
        let install_path = self.install_path.clone();
        let current_path = self.install_path.read(cx).clone();

        v_flex()
            .size_full()
            .gap_6()
            .p_8()
            .child(
                // Header
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_2xl()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("Choose Installation Location"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Select where you would like to install Pulsar"),
                    ),
            )
            .child(
                // Path input
                v_flex()
                    .gap_3()
                    .child(
                        div()
                            .text_sm()
                            .font_medium()
                            .text_color(cx.theme().foreground)
                            .child("Installation Directory"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Input::new("install-path")
                                    .flex_1()
                                    .placeholder("Enter installation path...")
                                    .value(current_path)
                                    .on_change({
                                        let install_path = install_path.clone();
                                        move |value, window, cx| {
                                            install_path.update(window, cx, |path, _window, _cx| {
                                                *path = value;
                                            });
                                        }
                                    }),
                            )
                            .child(
                                Button::new("browse-btn")
                                    .outline()
                                    .label("Browse...")
                                    .on_click(|_window, _cx| {
                                        // File dialog would be implemented here
                                    }),
                            ),
                    ),
            )
            .child(
                // Space information
                div()
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded(px(8.0))
                    .p_4()
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(cx.theme().foreground)
                                    .child("Disk Space Information"),
                            )
                            .child(
                                h_flex()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("Required space:"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_medium()
                                            .text_color(cx.theme().foreground)
                                            .child(Self::calculate_required_space()),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("Available space:"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_medium()
                                            .text_color(cx.theme().success)
                                            .child(Self::calculate_available_space()),
                                    ),
                            ),
                    ),
            )
            .child(
                // Warning/info message
                div()
                    .bg(cx.theme().accent.opacity(0.1))
                    .border_1()
                    .border_color(cx.theme().accent)
                    .rounded(px(8.0))
                    .p_3()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().foreground)
                            .child(
                                "Note: The installation directory will be created if it doesn't exist. \
                                 Make sure you have write permissions to the selected location.",
                            ),
                    ),
            )
            .child(div().flex_1()) // Spacer
            .child(
                // Navigation buttons
                h_flex()
                    .justify_between()
                    .child(
                        Button::new("back-btn")
                            .outline()
                            .label("Back")
                            .on_click(move |window, cx| {
                                let on_back = on_back.read(cx);
                                on_back(window, cx);
                            }),
                    )
                    .child(
                        Button::new("next-btn")
                            .primary()
                            .label("Next")
                            .on_click(move |window, cx| {
                                let on_next = on_next.read(cx);
                                on_next(window, cx);
                            }),
                    ),
            )
    }
}
