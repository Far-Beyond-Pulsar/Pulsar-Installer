//! License agreement view.

use gpui::{
    div, prelude::FluentBuilder as _, App, IntoElement, ParentElement, RenderOnce, Styled, Window,
    px, Entity,
};
use gpui_component::{
    ActiveTheme, h_flex, v_flex,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    scroll::ScrollableElement as _,
};

/// License agreement view.
pub struct LicenseView {
    accepted: Entity<bool>,
    on_back: Entity<Box<dyn Fn(&mut Window, &mut App)>>,
    on_next: Entity<Box<dyn Fn(&mut Window, &mut App)>>,
}

impl LicenseView {
    /// Create a new license view.
    pub fn new(
        on_back: impl Fn(&mut Window, &mut App) + 'static,
        on_next: impl Fn(&mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Self {
        Self {
            accepted: cx.new(|_| false),
            on_back: cx.new(|_| Box::new(on_back) as Box<dyn Fn(&mut Window, &mut App)>),
            on_next: cx.new(|_| Box::new(on_next) as Box<dyn Fn(&mut Window, &mut App)>),
        }
    }

    fn license_text() -> &'static str {
        r#"Apache License 2.0

Copyright (c) 2025 Pulsar Contributors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

TERMS AND CONDITIONS

1. Grant of Copyright License
   Subject to the terms and conditions of this License, each Contributor
   hereby grants to You a perpetual, worldwide, non-exclusive, no-charge,
   royalty-free, irrevocable copyright license to reproduce, prepare
   Derivative Works of, publicly display, publicly perform, sublicense,
   and distribute the Work and such Derivative Works in Source or Object form.

2. Grant of Patent License
   Subject to the terms and conditions of this License, each Contributor
   hereby grants to You a perpetual, worldwide, non-exclusive, no-charge,
   royalty-free, irrevocable (except as stated in this section) patent license
   to make, have made, use, offer to sell, sell, import, and otherwise
   transfer the Work.

By installing this software, you agree to these terms and conditions."#
    }
}

impl RenderOnce for LicenseView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let accepted = self.accepted.clone();
        let on_back = self.on_back;
        let on_next = self.on_next;
        let is_accepted = self.accepted.read(cx);

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
                            .child("License Agreement"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Please read and accept the license agreement to continue"),
                    ),
            )
            .child(
                // License text
                div()
                    .flex_1()
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded(px(8.0))
                    .p_4()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .font_mono()
                            .text_xs()
                            .text_color(cx.theme().foreground)
                            .whitespace_pre_wrap()
                            .child(Self::license_text()),
                    ),
            )
            .child(
                // Accept checkbox
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        Checkbox::new("accept-license")
                            .checked(*is_accepted)
                            .on_click({
                                let accepted = accepted.clone();
                                move |_event, window, cx| {
                                    accepted.update(window, cx, |value, _window, _cx| {
                                        *value = !*value;
                                    });
                                }
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child("I accept the terms and conditions of the license agreement"),
                    ),
            )
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
                            .disabled(!is_accepted)
                            .on_click(move |window, cx| {
                                let on_next = on_next.read(cx);
                                on_next(window, cx);
                            }),
                    ),
            )
    }
}
