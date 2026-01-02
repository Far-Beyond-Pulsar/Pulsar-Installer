//! Component selection view.

use gpui::{
    div, prelude::FluentBuilder as _, App, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, px, Entity,
};
use gpui_component::{
    ActiveTheme, h_flex, v_flex,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
};
use std::collections::HashSet;

/// Component information.
#[derive(Clone, Debug)]
pub struct ComponentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub size_mb: u64,
    pub required: bool,
}

/// Component selection view.
pub struct ComponentsView {
    components: Vec<ComponentInfo>,
    selected: Entity<HashSet<String>>,
    on_back: Entity<Box<dyn Fn(&mut Window, &mut App)>>,
    on_next: Entity<Box<dyn Fn(&mut Window, &mut App)>>,
}

impl ComponentsView {
    /// Create a new components view.
    pub fn new(
        on_back: impl Fn(&mut Window, &mut App) + 'static,
        on_next: impl Fn(&mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Self {
        let components = vec![
            ComponentInfo {
                id: "core".to_string(),
                name: "Pulsar Core Engine".to_string(),
                description: "The main game engine runtime and libraries".to_string(),
                size_mb: 850,
                required: true,
            },
            ComponentInfo {
                id: "editor".to_string(),
                name: "Pulsar Editor".to_string(),
                description: "Visual game editor and development environment".to_string(),
                size_mb: 650,
                required: false,
            },
            ComponentInfo {
                id: "docs".to_string(),
                name: "Documentation".to_string(),
                description: "API documentation and tutorials".to_string(),
                size_mb: 120,
                required: false,
            },
            ComponentInfo {
                id: "examples".to_string(),
                name: "Example Projects".to_string(),
                description: "Sample games and project templates".to_string(),
                size_mb: 450,
                required: false,
            },
            ComponentInfo {
                id: "tools".to_string(),
                name: "Development Tools".to_string(),
                description: "Asset pipeline and build tools".to_string(),
                size_mb: 280,
                required: false,
            },
        ];

        // Select required components by default
        let mut selected_set = HashSet::new();
        for component in &components {
            if component.required {
                selected_set.insert(component.id.clone());
            }
        }

        Self {
            components,
            selected: cx.new(|_| selected_set),
            on_back: cx.new(|_| Box::new(on_back) as Box<dyn Fn(&mut Window, &mut App)>),
            on_next: cx.new(|_| Box::new(on_next) as Box<dyn Fn(&mut Window, &mut App)>),
        }
    }

    fn calculate_total_size(components: &[ComponentInfo], selected: &HashSet<String>) -> u64 {
        components
            .iter()
            .filter(|c| selected.contains(&c.id))
            .map(|c| c.size_mb)
            .sum()
    }
}

impl RenderOnce for ComponentsView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let on_back = self.on_back;
        let on_next = self.on_next;
        let selected = self.selected.clone();
        let selected_set = self.selected.read(cx).clone();
        let total_size = Self::calculate_total_size(&self.components, &selected_set);

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
                            .child("Select Components"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Choose which components to install"),
                    ),
            )
            .child(
                // Component list
                v_flex()
                    .flex_1()
                    .gap_3()
                    .overflow_y_scroll()
                    .children(self.components.iter().map(|component| {
                        let is_selected = selected_set.contains(&component.id);
                        let component_id = component.id.clone();
                        let is_required = component.required;

                        div()
                            .border_1()
                            .border_color(if is_selected {
                                cx.theme().primary
                            } else {
                                cx.theme().border
                            })
                            .rounded(px(8.0))
                            .p_4()
                            .when(is_selected, |this| this.bg(cx.theme().accent.opacity(0.05)))
                            .child(
                                h_flex()
                                    .gap_3()
                                    .items_start()
                                    .child(
                                        Checkbox::new(SharedString::from(format!(
                                            "component-{}",
                                            component.id
                                        )))
                                        .checked(is_selected)
                                        .disabled(is_required)
                                        .on_click({
                                            let selected = selected.clone();
                                            move |_event, window, cx| {
                                                if !is_required {
                                                    selected.update(
                                                        window,
                                                        cx,
                                                        |set, _window, _cx| {
                                                            if set.contains(&component_id) {
                                                                set.remove(&component_id);
                                                            } else {
                                                                set.insert(component_id.clone());
                                                            }
                                                        },
                                                    );
                                                }
                                            }
                                        }),
                                    )
                                    .child(
                                        v_flex()
                                            .flex_1()
                                            .gap_1()
                                            .child(
                                                h_flex()
                                                    .gap_2()
                                                    .items_center()
                                                    .child(
                                                        div()
                                                            .text_base()
                                                            .font_semibold()
                                                            .text_color(cx.theme().foreground)
                                                            .child(component.name.clone()),
                                                    )
                                                    .when(component.required, |this| {
                                                        this.child(
                                                            div()
                                                                .text_xs()
                                                                .px_2()
                                                                .py(px(2.0))
                                                                .rounded(px(4.0))
                                                                .bg(cx.theme().destructive.opacity(0.1))
                                                                .text_color(cx.theme().destructive)
                                                                .child("Required"),
                                                        )
                                                    }),
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(component.description.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(format!("Size: {} MB", component.size_mb)),
                                            ),
                                    ),
                            )
                    })),
            )
            .child(
                // Total size
                div()
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded(px(8.0))
                    .p_4()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(cx.theme().foreground)
                                    .child("Total Download Size:"),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .font_bold()
                                    .text_color(cx.theme().primary)
                                    .child(format!("{} MB ({:.1} GB)", total_size, total_size as f64 / 1024.0)),
                            ),
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
                        Button::new("install-btn")
                            .primary()
                            .label("Install")
                            .on_click(move |window, cx| {
                                let on_next = on_next.read(cx);
                                on_next(window, cx);
                            }),
                    ),
            )
    }
}
