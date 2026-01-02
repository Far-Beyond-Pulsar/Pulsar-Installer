//! Main installer view following Story crate patterns.

use gpui::{
    App, AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Styled, Window, div, px,
};
use gpui_component::{
    ActiveTheme,
    Disableable as _,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    progress::Progress,
    scroll::ScrollableElement as _,
    h_flex, v_flex,
};
use crate::download::{GitHubReleases, HttpDownloadManager, GitHubRelease};
use crate::traits::{DownloadManager as _,  Progress as ProgressTrait};
use std::path::PathBuf;
use gpui_component::Disableable;
use gpui::prelude::FluentBuilder;

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
    loading_more: bool,
    current_releases_page: u32,
    has_more_releases: bool,
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
            loading_more: false,
            current_releases_page: 0,
            has_more_releases: true,
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
        self.current_releases_page = 1;
        self.releases.clear();
        cx.notify();

        // Spawn async task to fetch first page of releases
        cx.spawn(async move |this, cx| {
            let github = GitHubReleases::new("Far-Beyond-Pulsar", "Pulsar-Native");

            match github.get_releases_page(1, 30).await {
                Ok(releases) => {
                    let has_more = releases.len() >= 30;
                    let release_infos: Vec<ReleaseInfo> = releases
                        .into_iter()
                        .map(|r| ReleaseInfo {
                            tag_name: r.tag_name.clone(),
                            name: r.name.clone(),
                            selected: false,
                        })
                        .collect();

                    this.update(cx, |this, cx| {
                        this.releases = release_infos;
                        this.loading_releases = false;
                        this.has_more_releases = has_more;
                        cx.notify();
                    })
                    .ok();
                }
                Err(e) => {
                    tracing::error!("Failed to fetch releases: {}", e);
                    this.update(cx, |this, cx| {
                        this.loading_releases = false;
                        cx.notify();
                    })
                    .ok();
                }
            }
        })
        .detach();
    }

    fn load_more_releases(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.loading_more || !self.has_more_releases {
            return;
        }

        self.loading_more = true;
        self.current_releases_page += 1;
        let page = self.current_releases_page;
        cx.notify();

        // Spawn async task to fetch next page of releases
        cx.spawn(async move |this, cx| {
            let github = GitHubReleases::new("Far-Beyond-Pulsar", "Pulsar-Native");

            match github.get_releases_page(page, 30).await {
                Ok(releases) => {
                    let has_more = releases.len() >= 30;
                    let release_infos: Vec<ReleaseInfo> = releases
                        .into_iter()
                        .map(|r| ReleaseInfo {
                            tag_name: r.tag_name.clone(),
                            name: r.name.clone(),
                            selected: false,
                        })
                        .collect();

                    this.update(cx, |this, cx| {
                        this.releases.extend(release_infos);
                        this.loading_more = false;
                        this.has_more_releases = has_more;
                        cx.notify();
                    })
                    .ok();
                }
                Err(e) => {
                    tracing::error!("Failed to fetch more releases: {}", e);
                    this.update(cx, |this, cx| {
                        this.loading_more = false;
                        this.current_releases_page -= 1; // Revert page increment
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

        // Get selected releases
        let selected_releases: Vec<GitHubRelease> = self.releases.iter()
            .filter(|r| r.selected)
            .map(|r| GitHubRelease {
                tag_name: r.tag_name.clone(),
                name: r.name.clone(),
                body: String::new(),
                assets: Vec::new(),
                prerelease: false,
            })
            .collect();

        if selected_releases.is_empty() {
            self.install_message = "No versions selected".to_string();
            cx.notify();
            return;
        }

        let total_releases = selected_releases.len();

        // Start installation
        cx.spawn(async move |this, cx| {
            let download_manager = HttpDownloadManager::new();
            let github = GitHubReleases::new("Far-Beyond-Pulsar", "Pulsar-Native");

            // Create download directory
            let download_dir = std::env::temp_dir().join("pulsar-installer");
            if let Err(e) = std::fs::create_dir_all(&download_dir) {
                this.update(cx, |this, cx| {
                    this.install_message = format!("Failed to create download directory: {}", e);
                    cx.notify();
                })
                .ok();
                return;
            }

            // Get full release details and calculate total size
            let mut releases_with_assets = Vec::new();
            let mut total_size = 0u64;

            for selected_release in &selected_releases {
                match github.get_all_releases().await {
                    Ok(releases) => {
                        if let Some(full_release) = releases.into_iter().find(|r| r.tag_name == selected_release.tag_name) {
                            if let Some(asset) = full_release.assets.first() {
                                total_size += asset.size;
                                releases_with_assets.push((full_release, asset.clone()));
                            }
                        }
                    }
                    Err(e) => {
                        this.update(cx, |this, cx| {
                            this.install_message = format!("Failed to fetch release details: {}", e);
                            cx.notify();
                        })
                        .ok();
                        return;
                    }
                }
            }

            let mut downloaded_bytes = 0u64;

            for (idx, (release, asset)) in releases_with_assets.iter().enumerate() {
                let release_num = idx + 1;
                let release_name = release.name.clone();
                let asset_name = asset.name.clone();

                // Update status
                this.update(cx, |this, cx| {
                    this.install_message = format!(
                        "Downloading {} of {}: {}",
                        release_num, releases_with_assets.len(), release_name
                    );
                    cx.notify();
                })
                .ok();

                let file_path = download_dir.join(&asset.name);
                let url = asset.browser_download_url.clone();
                let base_downloaded = downloaded_bytes;

                // Download with progress tracking
                let result = download_manager
                    .download(&url, &file_path, Box::new(move |prog| {
                        let current_bytes = base_downloaded + prog.processed_bytes;
                        let overall_progress = if total_size > 0 {
                            (current_bytes as f32 / total_size as f32) * 100.0
                        } else {
                            0.0
                        };

                        // Update UI with current progress
                        this.update(cx, |this, cx| {
                            this.install_progress = overall_progress;
                            this.install_message = format!(
                                "Downloading {} ({:.1}%)",
                                asset_name,
                                prog.current
                            );
                            cx.notify();
                        })
                        .ok();
                    }))
                    .await;

                match result {
                    Ok(_) => {
                        downloaded_bytes += asset.size;

                        // Install the downloaded file
                        this.update(cx, |this, cx| {
                            this.install_message = format!("Installing {}...", release_name);
                            cx.notify();
                        })
                        .ok();

                        let install_result = Self::install_release(&file_path, &release.tag_name).await;

                        match install_result {
                            Ok(_install_path) => {
                                this.update(cx, |this, cx| {
                                    this.install_message = format!("Installed: {}", release_name);
                                    cx.notify();
                                })
                                .ok();
                            }
                            Err(e) => {
                                this.update(cx, |this, cx| {
                                    this.install_message = format!("Installation failed for {}: {}", release_name, e);
                                    cx.notify();
                                })
                                .ok();
                            }
                        }
                    }
                    Err(e) => {
                        this.update(cx, |this, cx| {
                            this.install_message = format!("Download failed: {}", e);
                            cx.notify();
                        })
                        .ok();
                        continue;
                    }
                }
            }

            // Navigate to complete page
            this.update(cx, |this, cx| {
                this.install_progress = 100.0;
                this.current_page = Page::Complete;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    async fn install_release(archive_path: &PathBuf, version: &str) -> crate::error::Result<PathBuf> {
        use std::fs;

        // Determine installation directory
        let install_base = if cfg!(windows) {
            PathBuf::from("C:\\Program Files\\Pulsar")
        } else if cfg!(target_os = "macos") {
            PathBuf::from("/Applications/Pulsar")
        } else {
            dirs::home_dir()
                .ok_or_else(|| crate::error::InstallerError::Other("Could not determine home directory".to_string()))?
                .join(".local/share/pulsar")
        };

        let install_dir = install_base.join(version);
        fs::create_dir_all(&install_dir)
            .map_err(|e| crate::error::InstallerError::Io(e))?;

        // Extract archive
        let file = fs::File::open(archive_path)
            .map_err(|e| crate::error::InstallerError::Io(e))?;

        if archive_path.extension().and_then(|s| s.to_str()) == Some("exe") {
            // Windows executable - just copy it
            let dest = install_dir.join(archive_path.file_name().unwrap());
            fs::copy(archive_path, &dest)
                .map_err(|e| crate::error::InstallerError::Io(e))?;
        } else if archive_path.to_str().map(|s| s.ends_with(".tar.gz")).unwrap_or(false) {
            // Extract tar.gz archive
            let tar = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(tar);
            archive.unpack(&install_dir)
                .map_err(|e| crate::error::InstallerError::Io(e))?;
        } else if archive_path.extension().and_then(|s| s.to_str()) == Some("zip") {
            // Extract zip archive
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| crate::error::InstallerError::Other(e.to_string()))?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)
                    .map_err(|e| crate::error::InstallerError::Other(e.to_string()))?;
                let outpath = install_dir.join(file.mangled_name());

                if file.name().ends_with('/') {
                    fs::create_dir_all(&outpath)
                        .map_err(|e| crate::error::InstallerError::Io(e))?;
                } else {
                    if let Some(p) = outpath.parent() {
                        fs::create_dir_all(p)
                            .map_err(|e| crate::error::InstallerError::Io(e))?;
                    }
                    let mut outfile = fs::File::create(&outpath)
                        .map_err(|e| crate::error::InstallerError::Io(e))?;
                    std::io::copy(&mut file, &mut outfile)
                        .map_err(|e| crate::error::InstallerError::Io(e))?;
                }
            }
        }

        // Create start menu shortcut on Windows
        #[cfg(windows)]
        {
            Self::create_windows_shortcut(&install_dir, version)?;
        }

        Ok(install_dir)
    }

    #[cfg(windows)]
    fn create_windows_shortcut(install_dir: &PathBuf, version: &str) -> crate::error::Result<()> {
        use std::fs;
        use std::io::Write;

        // Find the main executable
        let exe_path = std::fs::read_dir(install_dir)
            .map_err(|e| crate::error::InstallerError::Io(e))?
            .filter_map(|entry| entry.ok())
            .find(|entry| {
                entry.path().extension().and_then(|s| s.to_str()) == Some("exe")
            })
            .map(|entry| entry.path());

        if let Some(exe_path) = exe_path {
            // Create start menu shortcut directory
            let start_menu = dirs::data_dir()
                .ok_or_else(|| crate::error::InstallerError::Other("Could not find start menu".to_string()))?
                .join("Microsoft\\Windows\\Start Menu\\Programs\\Pulsar");

            fs::create_dir_all(&start_menu)
                .map_err(|e| crate::error::InstallerError::Io(e))?;

            // Create a simple .bat file as a launcher (proper .lnk would require winapi)
            let shortcut_path = start_menu.join(format!("Pulsar {}.bat", version));
            let mut file = fs::File::create(&shortcut_path)
                .map_err(|e| crate::error::InstallerError::Io(e))?;

            writeln!(file, "@echo off")
                .map_err(|e| crate::error::InstallerError::Io(e))?;
            writeln!(file, "start \"\" \"{}\"", exe_path.display())
                .map_err(|e| crate::error::InstallerError::Io(e))?;
        }

        Ok(())
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
            Page::Welcome => self.render_welcome(cx).into_any_element(),
            Page::VersionSelection => self.render_version_selection(cx).into_any_element(),
            Page::Installing => self.render_installing(cx).into_any_element(),
            Page::Complete => self.render_complete(cx).into_any_element(),
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
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(if self.loading_releases {
                        v_flex()
                            .size_full()
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
                            .size_full()
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
                            .size_full()
                            .gap_2()
                            .overflow_y_scrollbar()
                            .children(
                                self.releases
                                    .iter()
                                    .enumerate()
                                    .map(|(idx, release): (usize, &ReleaseInfo)| {
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
                                                            .on_click(cx.listener(move |this, _checked: &bool, window, cx| {
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
                            .when(self.has_more_releases || self.loading_more, |this| {
                                this.child(
                                    div()
                                        .p_3()
                                        .flex()
                                        .justify_center()
                                        .child(
                                            Button::new("load-more-btn")
                                                .outline()
                                                .label(if self.loading_more { "Loading..." } else { "Load More" })
                                                .disabled(self.loading_more)
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.load_more_releases(window, cx);
                                                }))
                                        )
                                )
                            })
                            .into_any_element()
                    })
            )
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
