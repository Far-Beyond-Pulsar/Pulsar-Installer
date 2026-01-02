//! Installation progress view.

use gpui::{
    div, prelude::FluentBuilder as _, App, IntoElement, ParentElement, RenderOnce, Styled, Window,
    px, Entity,
};
use gpui_component::{
    ActiveTheme, h_flex, v_flex,
    progress::Progress,
    spinner::Spinner,
};

/// Installation step information.
#[derive(Clone, Debug)]
pub struct StepInfo {
    pub name: String,
    pub status: StepStatus,
}

/// Step execution status.
#[derive(Clone, Debug, PartialEq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

/// Installation progress view.
pub struct InstallationView {
    steps: Entity<Vec<StepInfo>>,
    current_progress: Entity<f32>,
    current_message: Entity<String>,
}

impl InstallationView {
    /// Create a new installation view.
    pub fn new(cx: &mut App) -> Self {
        let steps = vec![
            StepInfo {
                name: "Checking system requirements".to_string(),
                status: StepStatus::Pending,
            },
            StepInfo {
                name: "Creating installation directories".to_string(),
                status: StepStatus::Pending,
            },
            StepInfo {
                name: "Downloading components".to_string(),
                status: StepStatus::Pending,
            },
            StepInfo {
                name: "Extracting files".to_string(),
                status: StepStatus::Pending,
            },
            StepInfo {
                name: "Creating shortcuts".to_string(),
                status: StepStatus::Pending,
            },
            StepInfo {
                name: "Finalizing installation".to_string(),
                status: StepStatus::Pending,
            },
        ];

        Self {
            steps: cx.new(|_| steps),
            current_progress: cx.new(|_| 0.0),
            current_message: cx.new(|_| "Preparing installation...".to_string()),
        }
    }
}

impl RenderOnce for InstallationView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let steps = self.steps.read(cx).clone();
        let progress_value = *self.current_progress.read(cx);
        let message = self.current_message.read(cx).clone();

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
                            .child("Installing Pulsar"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Please wait while Pulsar is being installed..."),
                    ),
            )
            .child(
                // Overall progress
                v_flex()
                    .gap_3()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .text_color(cx.theme().foreground)
                                    .child(message),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(cx.theme().primary)
                                    .child(format!("{:.0}%", progress_value)),
                            ),
                    )
                    .child(Progress::new("overall-progress").value(progress_value)),
            )
            .child(
                // Step list
                div()
                    .flex_1()
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded(px(8.0))
                    .p_4()
                    .overflow_y_scroll()
                    .child(
                        v_flex()
                            .gap_3()
                            .children(steps.iter().map(|step| {
                                let (icon, color) = match &step.status {
                                    StepStatus::Pending => ("○", cx.theme().muted_foreground),
                                    StepStatus::InProgress => ("◐", cx.theme().primary),
                                    StepStatus::Completed => ("✓", cx.theme().success),
                                    StepStatus::Failed(_) => ("✗", cx.theme().destructive),
                                };

                                h_flex()
                                    .gap_3()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_lg()
                                            .text_color(color)
                                            .child(icon),
                                    )
                                    .child(
                                        v_flex()
                                            .flex_1()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_medium()
                                                    .text_color(match &step.status {
                                                        StepStatus::Completed => cx.theme().success,
                                                        StepStatus::Failed(_) => cx.theme().destructive,
                                                        StepStatus::InProgress => cx.theme().primary,
                                                        _ => cx.theme().foreground,
                                                    })
                                                    .child(step.name.clone()),
                                            )
                                            .when_some(
                                                match &step.status {
                                                    StepStatus::Failed(msg) => Some(msg.clone()),
                                                    _ => None,
                                                },
                                                |this, error_msg| {
                                                    this.child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(cx.theme().destructive)
                                                            .child(error_msg),
                                                    )
                                                },
                                            ),
                                    )
                                    .when(step.status == StepStatus::InProgress, |this| {
                                        this.child(Spinner::new("step-spinner"))
                                    })
                            })),
                    ),
            )
            .child(
                // Info message
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
                                "This may take several minutes depending on your internet connection. \
                                 Please do not close this window.",
                            ),
                    ),
            )
    }
}
