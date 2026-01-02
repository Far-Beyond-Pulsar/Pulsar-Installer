//! Welcome screen view.

use gpui::{
    div, prelude::FluentBuilder as _, App, IntoElement, ParentElement, RenderOnce, Styled, Window,
    px, rgb, Entity,
};
use gpui_component::{
    ActiveTheme, h_flex, v_flex,
    button::{Button, ButtonVariants as _},
};

/// Welcome screen view.
pub struct WelcomeView {
    on_next: Entity<Box<dyn Fn(&mut Window, &mut App)>>,
}

impl WelcomeView {
    /// Create a new welcome view.
    pub fn new(on_next: impl Fn(&mut Window, &mut App) + 'static, cx: &mut App) -> Self {
        Self {
            on_next: cx.new(|_| Box::new(on_next) as Box<dyn Fn(&mut Window, &mut App)>),
        }
    }
}

impl RenderOnce for WelcomeView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let on_next = self.on_next;

        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_8()
            .child(
                // Logo/Header
                v_flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .w(px(128.0))
                            .h(px(128.0))
                            .rounded(px(16.0))
                            .bg(cx.theme().primary)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .text_4xl()
                                    .font_semibold()
                                    .text_color(rgb(0xFFFFFF))
                                    .child("P"),
                            ),
                    )
                    .child(
                        div()
                            .text_4xl()
                            .font_bold()
                            .text_color(cx.theme().primary)
                            .child("Pulsar Game Engine"),
                    )
                    .child(
                        div()
                            .text_lg()
                            .text_color(cx.theme().muted_foreground)
                            .child("Version 1.0.0"),
                    ),
            )
            .child(
                // Description
                v_flex()
                    .w(px(600.0))
                    .gap_3()
                    .items_center()
                    .child(
                        div()
                            .text_base()
                            .text_color(cx.theme().foreground)
                            .text_center()
                            .child("Welcome to the Pulsar Game Engine installer!"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .text_center()
                            .child(
                                "This wizard will guide you through the installation process. \
                                 Pulsar is a modern, high-performance game engine designed for \
                                 creating stunning 2D and 3D games.",
                            ),
                    ),
            )
            .child(
                // Feature highlights
                v_flex()
                    .w(px(600.0))
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(div().text_color(cx.theme().primary).child("✓"))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().foreground)
                                    .child("Cross-platform support (Windows, macOS, Linux)"),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(div().text_color(cx.theme().primary).child("✓"))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().foreground)
                                    .child("Modern rendering pipeline with Vulkan/Metal support"),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(div().text_color(cx.theme().primary).child("✓"))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().foreground)
                                    .child("Integrated editor and development tools"),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(div().text_color(cx.theme().primary).child("✓"))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().foreground)
                                    .child("Rich ecosystem of plugins and assets"),
                            ),
                    ),
            )
            .child(
                // Next button
                Button::new("next-btn")
                    .primary()
                    .large()
                    .label("Get Started")
                    .on_click(move |window, cx| {
                        let on_next = on_next.read(cx);
                        on_next(window, cx);
                    }),
            )
    }
}
