//! Installation complete view.

use gpui::{
    div, prelude::FluentBuilder as _, rgb, App, IntoElement, ParentElement, RenderOnce, Styled,
    Window, px, Entity,
};
use gpui_component::{
    ActiveTheme, h_flex, v_flex,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
};

/// Installation complete view.
pub struct CompleteView {
    launch_app: Entity<bool>,
    on_finish: Entity<Box<dyn Fn(&mut Window, &mut App)>>,
}

impl CompleteView {
    /// Create a new complete view.
    pub fn new(on_finish: impl Fn(&mut Window, &mut App) + 'static, cx: &mut App) -> Self {
        Self {
            launch_app: cx.new(|_| true),
            on_finish: cx.new(|_| Box::new(on_finish) as Box<dyn Fn(&mut Window, &mut App)>),
        }
    }
}

impl RenderOnce for CompleteView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let on_finish = self.on_finish;
        let launch_app = self.launch_app.clone();
        let should_launch = *self.launch_app.read(cx);

        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_8()
            .child(
                // Success icon
                div()
                    .w(px(128.0))
                    .h(px(128.0))
                    .rounded_full()
                    .bg(cx.theme().success.opacity(0.1))
                    .border_2()
                    .border_color(cx.theme().success)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_6xl()
                            .text_color(cx.theme().success)
                            .child("✓"),
                    ),
            )
            .child(
                // Success message
                v_flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_3xl()
                            .font_bold()
                            .text_color(cx.theme().foreground)
                            .child("Installation Complete!"),
                    )
                    .child(
                        div()
                            .text_base()
                            .text_color(cx.theme().muted_foreground)
                            .text_center()
                            .child("Pulsar has been successfully installed on your system"),
                    ),
            )
            .child(
                // Installation details
                v_flex()
                    .w(px(600.0))
                    .gap_2()
                    .p_4()
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded(px(8.0))
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("What's Next?"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(div().text_color(cx.theme().primary).child("•"))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().foreground)
                                    .child("Launch Pulsar Editor to start creating games"),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(div().text_color(cx.theme().primary).child("•"))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().foreground)
                                    .child("Check out the example projects to learn the basics"),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(div().text_color(cx.theme().primary).child("•"))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().foreground)
                                    .child("Visit our documentation for tutorials and guides"),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(div().text_color(cx.theme().primary).child("•"))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().foreground)
                                    .child("Join our community to connect with other developers"),
                            ),
                    ),
            )
            .child(
                // Launch option
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        Checkbox::new("launch-app")
                            .checked(should_launch)
                            .on_click({
                                let launch_app = launch_app.clone();
                                move |_event, window, cx| {
                                    launch_app.update(window, cx, |value, _window, _cx| {
                                        *value = !*value;
                                    });
                                }
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child("Launch Pulsar Editor now"),
                    ),
            )
            .child(
                // Finish button
                Button::new("finish-btn")
                    .primary()
                    .large()
                    .label("Finish")
                    .on_click(move |window, cx| {
                        let on_finish = on_finish.read(cx);
                        on_finish(window, cx);
                    }),
            )
    }
}
