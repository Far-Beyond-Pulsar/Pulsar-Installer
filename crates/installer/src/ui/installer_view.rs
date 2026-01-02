use gpui::{
    App, AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Styled,
    Window, div, px,
};
use gpui_component::{
    ActiveTheme, Sizable,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    progress::Progress,
    h_flex, v_flex,
};
use crate::download::GitHubReleases;

/// Page state for the installer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Welcome,
    VersionSelection,
    Installing,
    Complete,
}

/// GitHub release information
#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: String,
    pub selected: bool,
}

/// Main installer view
pub struct InstallerView {
    focus_handle: gpui::FocusHandle,
    current_page: Page,
    releases: Vec<ReleaseInfo>,
    loading_releases: bool,
    install_progress: f32,
    install_message: String,
}

impl InstallerView {
    pub fn view(_window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(cx))
    }

    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            current_page: Page::Welcome,
            releases: Vec::new(),
            loading_releases: false,
            install_progress: 0.0,
            install_message: String::new(),
        }
    }

    fn navigate_to(&mut self, page: Page, _window: &mut Window, cx: &mut Context<Self>) {
        self.current_page = page;
        cx.notify();
    }

    fn fetch_releases(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.loading_releases = true;
        cx.notify();

        // Spawn async task to fetch releases
        cx.spawn(|this, mut cx| async move {
            let github = GitHubReleases::new("Far-Beyond-Pulsar", "Pulsar-Native");

            match github.get_all_releases().await {
                Ok(releases) => {
                    let release_infos: Vec<ReleaseInfo> = releases
                        .into_iter()
                        .map(|r| ReleaseInfo {
                            tag_name: r.tag_name.clone(),
                            name: r.name.clone(),
                            selected: false,
                        })
                        .collect();

                    this.update(&mut cx, |this, window, cx| {
                        this.releases = release_infos;
                        this.loading_releases = false;
                        cx.notify();
                    })
                    .ok();
                }
                Err(e) => {
                    tracing::error!("Failed to fetch releases: {}", e);
                    this.update(&mut cx, |this, window, cx| {
                        this.loading_releases = false;
                        cx.notify();
                    })
                    .ok();
                }
            }
        })
        .detach();
    }

    fn toggle_release(&mut self, index: usize, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(release) = self.releases.get_mut(index) {
            release.selected = !release.selected;
            cx.notify();
        }
    }

    fn start_installation(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.install_progress = 0.0;
        self.install_message = "Starting installation...".to_string();
        cx.notify();

        // Simulate installation progress
        cx.spawn(|this, mut cx| async move {
            for i in 0..=100 {
                smol::Timer::after(std::time::Duration::from_millis(50)).await;

                this.update(&mut cx, |this, window, cx| {
                    this.install_progress = i as f32;
                    this.install_message = format!("Installing... {}%", i);
                    cx.notify();
                })
                .ok();
            }

            // Navigate to complete page
            this.update(&mut cx, |this, window, cx| {
                this.navigate_to(Page::Complete, window, cx);
            })
            .ok();
        })
        .detach();
    }
}

impl Focusable for InstallerView {
    fn focus_handle(&self, _: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for InstallerView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.current_page {
            Page::Welcome => self.render_welcome(cx),
            Page::VersionSelection => self.render_version_selection(cx),
            Page::Installing => self.render_installing(cx),
            Page::Complete => self.render_complete(cx),
        }
    }
}

impl InstallerView {
    fn render_welcome(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_6()
            .child(
                // Logo
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
                            .text_3xl()
                            .text_color(cx.theme().primary_foreground)
                            .child("P"),
                    ),
            )
            .child(
                div()
                    .text_3xl()
                    .text_color(cx.theme().foreground)
                    .child("Pulsar Engine Installer"),
            )
            .child(
                div()
                    .text_base()
                    .text_color(cx.theme().muted_foreground)
                    .child("Install and manage Pulsar engine versions"),
            )
            .child(
                Button::new("start-btn")
                    .primary()
                    .label("Get Started")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.navigate_to(Page::VersionSelection, window, cx);
                        this.fetch_releases(window, cx);
                    })),
            )
    }

    fn render_version_selection(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_4()
            .p_6()
            .child(
                div()
                    .text_2xl()
                    .text_color(cx.theme().foreground)
                    .child("Select Versions to Install"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Choose one or more Pulsar engine versions from GitHub releases"),
            )
            .child(if self.loading_releases {
                v_flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_base()
                            .text_color(cx.theme().muted_foreground)
                            .child("Loading releases from GitHub..."),
                    )
                    .into_any_element()
            } else if self.releases.is_empty() {
                v_flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .child(
                        div()
                            .text_base()
                            .text_color(cx.theme().muted_foreground)
                            .child("No releases found or failed to load"),
                    )
                    .child(
                        Button::new("retry-btn")
                            .outline()
                            .label("Retry")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.fetch_releases(window, cx);
                            })),
                    )
                    .into_any_element()
            } else {
                v_flex()
                    .flex_1()
                    .gap_2()
                    .overflow_y_scroll()
                    .children(
                        self.releases
                            .iter()
                            .enumerate()
                            .map(|(idx, release)| {
                                let selected = release.selected;
                                let release_name = release.name.clone();
                                let tag_name = release.tag_name.clone();

                                div()
                                    .p_3()
                                    .border_1()
                                    .border_color(if selected {
                                        cx.theme().primary
                                    } else {
                                        cx.theme().border
                                    })
                                    .rounded(px(6.0))
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_3()
                                            .child(
                                                Checkbox::new(format!("release-{}", idx))
                                                    .checked(selected)
                                                    .on_click(cx.listener(move |this, checked: &bool, window, cx| {
                                                        this.toggle_release(idx, window, cx);
                                                    })),
                                            )
                                            .child(
                                                v_flex()
                                                    .gap_1()
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(cx.theme().foreground)
                                                            .child(release_name),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child(tag_name),
                                                    ),
                                            ),
                                    )
                            }),
                    )
                    .into_any_element()
            })
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        Button::new("back-btn")
                            .outline()
                            .label("Back")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.navigate_to(Page::Welcome, window, cx);
                            })),
                    )
                    .child(
                        Button::new("install-btn")
                            .primary()
                            .label("Install Selected")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.navigate_to(Page::Installing, window, cx);
                                this.start_installation(window, cx);
                            })),
                    ),
            )
    }

    fn render_installing(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_6()
            .child(
                div()
                    .text_2xl()
                    .text_color(cx.theme().foreground)
                    .child("Installing Pulsar Engine"),
            )
            .child(
                v_flex()
                    .w(px(400.0))
                    .gap_3()
                    .child(Progress::new("install-progress").value(self.install_progress))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .text_center()
                            .child(self.install_message.clone()),
                    ),
            )
    }

    fn render_complete(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_6()
            .child(
                div()
                    .w(px(100.0))
                    .h(px(100.0))
                    .rounded_full()
                    .border_2()
                    .border_color(cx.theme().success)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_3xl()
                            .text_color(cx.theme().success)
                            .child("✓"),
                    ),
            )
            .child(
                div()
                    .text_2xl()
                    .text_color(cx.theme().foreground)
                    .child("Installation Complete!"),
            )
            .child(
                div()
                    .text_base()
                    .text_color(cx.theme().muted_foreground)
                    .child("Pulsar engine has been successfully installed"),
            )
            .child(
                Button::new("finish-btn")
                    .primary()
                    .label("Finish")
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.quit();
                    })),
            )
    }
}
