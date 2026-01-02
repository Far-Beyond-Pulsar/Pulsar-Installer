# Pulsar Installer

A modern, cross-platform installer for the Pulsar Game Engine, built with GPUI and Rust.

## Features

- **🎨 Beautiful UI**: Modern, native-looking interface built with GPUI component library
- **🌍 Cross-Platform**: Runs on Windows, macOS, and Linux
- **📦 Modular Architecture**: Trait-based design for easy customization and extension
- **📊 Progress Tracking**: Real-time installation progress with detailed feedback
- **☁️ GitHub Integration**: Automatically fetches latest releases from GitHub
- **🔒 Verification**: SHA256 checksum validation for downloaded files
- **🔄 Rollback Support**: Automatic cleanup on installation failure
- **⚡ Async Operations**: Non-blocking downloads and installation

## Architecture

The installer is built around a modular, trait-based architecture that makes it easy to customize and extend.

### Core Traits

#### `InstallStep`
Defines individual installation steps that can be executed sequentially:
```rust
pub trait InstallStep: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, progress: ProgressCallback) -> Result<()>;
    async fn rollback(&self) -> Result<()>;
}
```

#### `SystemDetector`
Detects system information and validates requirements:
```rust
pub trait SystemDetector: Send + Sync {
    fn os_name(&self) -> &str;
    fn architecture(&self) -> &str;
    async fn available_space(&self, path: &Path) -> Result<u64>;
    async fn check_requirements(&self, requirements: &SystemRequirements) -> Result<()>;
    fn default_install_path(&self) -> PathBuf;
}
```

#### `DownloadManager`
Handles file downloads with progress tracking:
```rust
pub trait DownloadManager: Send + Sync {
    async fn download(&self, url: &str, destination: &Path, progress: ProgressCallback) -> Result<()>;
    async fn download_with_verification(&self, url: &str, destination: &Path,
                                       expected_checksum: &str, progress: ProgressCallback) -> Result<()>;
}
```

#### `ComponentInstaller`
Manages installation of individual components:
```rust
pub trait ComponentInstaller: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn size_bytes(&self) -> u64;
    async fn install(&self, install_path: &Path, progress: ProgressCallback) -> Result<()>;
}
```

### Module Structure

```
crates/installer/
├── src/
│   ├── lib.rs              # Main library entry point
│   ├── main.rs             # Binary entry point
│   ├── error.rs            # Error types
│   ├── traits.rs           # Core trait definitions
│   ├── config.rs           # Configuration management
│   ├── steps/              # Pre-built installation steps
│   │   ├── mod.rs
│   │   ├── check_requirements.rs
│   │   ├── create_directories.rs
│   │   ├── extract_files.rs
│   │   ├── create_shortcuts.rs
│   │   └── finalize.rs
│   ├── platform/           # Platform-specific implementations
│   │   ├── mod.rs
│   │   ├── detector.rs
│   │   ├── windows.rs
│   │   ├── macos.rs
│   │   └── linux.rs
│   ├── download/           # Download management
│   │   ├── mod.rs
│   │   ├── manager.rs
│   │   ├── verifier.rs
│   │   └── github.rs       # GitHub releases integration
│   └── ui/                 # User interface
│       ├── mod.rs
│       ├── app.rs          # Main application
│       ├── welcome.rs
│       ├── license.rs
│       ├── path_selection.rs
│       ├── components.rs
│       ├── installation.rs
│       └── complete.rs
```

## Usage

### Building the Installer

```bash
# Build in release mode
cargo build --release -p pulsar-installer

# Run the installer
./target/release/pulsar-installer
```

### Customizing the Installer

The installer is designed to be easily customizable. Here are some common customization scenarios:

#### Adding a Custom Installation Step

```rust
use pulsar_installer::traits::{InstallStep, ProgressCallback};
use async_trait::async_trait;

struct MyCustomStep {
    // Your fields here
}

#[async_trait]
impl InstallStep for MyCustomStep {
    fn name(&self) -> &str {
        "My Custom Step"
    }

    fn description(&self) -> &str {
        "Doing something custom"
    }

    async fn execute(&self, progress: ProgressCallback) -> Result<()> {
        // Your custom installation logic here
        progress(Progress::new(50.0));
        // ...
        Ok(())
    }
}
```

#### Customizing the GitHub Repository

The installer fetches binaries from GitHub releases. To change the repository:

```rust
use pulsar_installer::download::GitHubReleases;

let releases = GitHubReleases::new("your-org", "your-repo");
let binary = releases.find_platform_binary().await?;
```

#### Binary Naming Convention

The installer expects binaries in GitHub releases to follow this naming pattern:

```
pulsar-{os}-{arch}.{ext}
```

Where:
- `{os}` is `windows`, `macos`, or `linux`
- `{arch}` is `x86_64`, `aarch64`, etc.
- `{ext}` is `exe` for Windows, `tar.gz` for Unix systems

Examples:
- `pulsar-windows-x86_64.exe`
- `pulsar-macos-aarch64.tar.gz`
- `pulsar-linux-x86_64.tar.gz`

#### Adding Custom UI Pages

To add a new page to the installer:

1. Create a new view in `src/ui/`:

```rust
use gpui::*;
use gpui_component::*;

pub struct MyCustomView {
    // Your state here
}

impl RenderOnce for MyCustomView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .child(div().child("My Custom Page"))
            // Your UI here
    }
}
```

2. Add it to the `InstallerPage` enum in `src/ui/app.rs`:

```rust
pub enum InstallerPage {
    Welcome,
    License,
    MyCustomPage,  // Add your page here
    // ...
}
```

3. Handle it in the `render_page` method.

## Installation Steps

The default installation process includes these steps:

1. **Check System Requirements**: Verifies disk space, OS version, and architecture
2. **Create Directories**: Sets up the installation directory structure
3. **Download Components**: Fetches selected components from GitHub releases
4. **Extract Files**: Unpacks downloaded archives
5. **Create Shortcuts**: Adds desktop and start menu shortcuts (optional)
6. **Finalize Installation**: Configures PATH and writes installation metadata

## Configuration

The installer uses `InstallerConfig` to manage settings:

```rust
pub struct InstallerConfig {
    pub install_path: PathBuf,
    pub selected_components: Vec<String>,
    pub create_desktop_shortcut: bool,
    pub create_start_menu_shortcut: bool,
    pub add_to_path: bool,
    pub requirements: SystemRequirements,
}
```

## Error Handling

All errors are handled through the `InstallerError` enum:

```rust
pub enum InstallerError {
    Io(std::io::Error),
    Download(String),
    ChecksumMismatch { file: String, expected: String, actual: String },
    InsufficientSpace { needed: u64, available: u64 },
    RequirementsNotMet(String),
    InvalidPath(PathBuf),
    ComponentFailed { component: String, reason: String },
    Config(String),
    UnsupportedPlatform(String),
    Other(String),
}
```

## Platform Support

### Windows
- Default install path: `C:\Program Files\Pulsar`
- Creates `.exe` shortcuts
- Optionally adds to system PATH via registry

### macOS
- Default install path: `/Applications/Pulsar.app`
- Creates `.app` bundle
- Optionally adds to user PATH

### Linux
- Default install path: `~/.local/share/pulsar`
- Creates `.desktop` files
- Optionally adds to `~/.bashrc` or `~/.zshrc`

## Dependencies

Key dependencies:
- **GPUI**: UI framework
- **gpui-component**: UI component library
- **reqwest**: HTTP client for downloads
- **tokio**: Async runtime
- **tar**: Archive extraction
- **sha2**: Checksum verification
- **serde**: Configuration serialization

## Development

### Running Tests

```bash
cargo test -p pulsar-installer
```

### Running with Logging

```bash
RUST_LOG=debug cargo run -p pulsar-installer
```

## License

Licensed under the Apache License 2.0. See LICENSE-APACHE for details.

## Contributing

See CONTRIBUTING.md for guidelines on how to contribute to this project.
