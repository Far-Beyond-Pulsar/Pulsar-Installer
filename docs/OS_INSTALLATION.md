# OS-Specific Installation Implementation

This document describes the correct, native installation behavior for each supported platform.

## Design Principles

1. **No Cross-Platform Abstractions**: Each OS is handled explicitly with native conventions
2. **User-Level by Default**: No elevation required unless system-wide install is chosen
3. **OS-Native Discovery**: Applications are registered through platform mechanisms
4. **Reversible**: Complete uninstallation removes all traces
5. **No Silent Modifications**: No automatic PATH or shell profile changes

## Windows Installation

### Install Location
- **User Install**: `%LOCALAPPDATA%\Programs\Pulsar\`
- Files copied: Executable, resources, supporting files

### Start Menu Registration
- **Location**: `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Pulsar.lnk`
- Shortcut points to installed executable
- Working directory set to install location
- Icon included if available

### Add/Remove Programs (ARP)
- **Registry Key**: `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Pulsar`
- **Required Values**:
  - `DisplayName`: Application name
  - `DisplayVersion`: Version string
  - `Publisher`: Publisher name
  - `InstallLocation`: Installation directory path
  - `UninstallString`: Path to uninstaller
  - `DisplayIcon`: Icon path for ARP display
  - `NoModify`, `NoRepair`: Disable modify/repair options
  - `EstimatedSize`: Size in KB for disk space reporting

### Uninstall
Removes:
1. Installed files and directories
2. Start Menu shortcut
3. Registry entries under Uninstall key

### Implementation
- Module: `crates/installer/src/platform/windows.rs`
- Type: `WindowsInstaller`
- Uses `winreg` crate for registry operations
- Uses PowerShell COM interface for shortcut creation

## macOS Installation

### App Bundle Creation
Creates a valid `.app` bundle with proper structure:

```
Pulsar.app/
  Contents/
    Info.plist          # Bundle metadata
    MacOS/
      pulsar            # Executable binary
    Resources/          # Icons and resources
```

### Install Location
- **User Install**: `~/Applications/Pulsar.app`
- **System Install**: `/Applications/Pulsar.app`

### Info.plist
Required keys for Launch Services recognition:
- `CFBundleIdentifier`: Unique bundle ID (e.g., `com.pulsarteam.pulsar`)
- `CFBundleName`: Short name
- `CFBundleDisplayName`: Display name
- `CFBundleExecutable`: Binary name in MacOS folder
- `CFBundleVersion`: Build version
- `CFBundleShortVersionString`: User-visible version
- `CFBundlePackageType`: Always "APPL" for applications
- `LSMinimumSystemVersion`: Minimum macOS version
- `NSHighResolutionCapable`: Enable Retina support

### Registration
- **No Manual Registration**: Launch Services automatically detects `.app` bundles
- **No Database Edits**: Never manually modify Launch Services database
- Simply placing the `.app` in Applications folder registers it

### Uninstall
- Remove entire `.app` bundle
- Launch Services automatically detects removal
- No cleanup of system databases required

### Implementation
- Module: `crates/installer/src/platform/macos.rs`
- Type: `MacOSInstaller`
- Uses `plist` crate for Info.plist generation

## Linux Installation

Follows [freedesktop.org](https://freedesktop.org) specifications.

### Binary Installation
- **User Install**: `~/.local/bin/pulsar`
- **System Install**: `/usr/bin/pulsar`
- Permissions: `0755` (rwxr-xr-x)

### Desktop Entry
- **Location**: `~/.local/share/applications/pulsar.desktop` (user)
- **System**: `/usr/share/applications/pulsar.desktop`

**Format** (freedesktop.org Desktop Entry Specification):
```ini
[Desktop Entry]
Type=Application
Name=Pulsar
Comment=Pulsar Game Engine Installer
Exec=/home/user/.local/bin/pulsar
Icon=pulsar
Terminal=false
Categories=Development;Game;
Keywords=pulsar;game;engine;installer;
Version=1.0.0
```

### Icon Installation
Following hicolor icon theme specification:
- **Base**: `~/.local/share/icons/hicolor/` (user)
- **System**: `/usr/share/icons/hicolor/`
- **Structure**: `<base>/<size>/apps/pulsar.png`
- **Sizes**: 16x16, 32x32, 48x48, 64x64, 128x128, 256x256

### Desktop Database Updates
- Call `update-desktop-database` if available (optional but recommended)
- Call `gtk-update-icon-cache` for icons (optional)
- Desktop environments will eventually pick up changes without these

### Uninstall
Removes:
1. Binary from bin directory
2. Desktop entry
3. Icons from all size directories

### Implementation
- Module: `crates/installer/src/platform/linux.rs`
- Type: `LinuxInstaller`
- Uses standard filesystem operations
- Uses Unix permissions (chmod)

## Uninstaller

The uninstaller is a cross-platform module that delegates to platform-specific implementations.

### Module
- `crates/installer/src/uninstaller.rs`
- Type: `Uninstaller`

### Usage
```rust
// Load from metadata
let uninstaller = Uninstaller::from_metadata(metadata_path)?;

// Or create directly
let uninstaller = Uninstaller::new(install_path, version);

// Uninstall
uninstaller.uninstall(progress_callback).await?;
```

### Metadata
Each installation writes `uninstall_metadata.json` containing:
- Install path
- Version
- Platform-specific paths (shortcuts, registry keys, desktop entries)
- Installation date

## Explicit Non-Goals

### PATH Modification
- **Not done automatically**
- PATH modification is optional and should be user-initiated
- Each OS has different PATH semantics:
  - Windows: Registry `Environment` key
  - macOS/Linux: Shell profiles (~/.bashrc, ~/.zshrc, etc.)
- Modifying these silently violates user expectations

### Shell Profile Modifications
- Never automatically edit ~/.bashrc, ~/.zshrc, or similar
- These are user-managed configuration files
- Automated changes can break user setups

### System-Wide Installation Without Elevation
- System directories require elevation on all platforms
- We default to user-level install (no privileges required)
- System install should be an explicit user choice with elevation

### Package Manager Integration
- Linux: No integration with apt, dnf, pacman, etc.
- macOS: No Homebrew integration
- Windows: No Chocolatey/Scoop integration
- This is runtime installation, not package management

## File Structure

```
crates/installer/src/
├── platform/
│   ├── mod.rs              # Platform detection and exports
│   ├── detector.rs         # System information detection
│   ├── windows.rs          # Windows-specific installation
│   ├── macos.rs            # macOS-specific installation
│   └── linux.rs            # Linux-specific installation
├── steps/
│   ├── create_shortcuts.rs # OS integration step (renamed from shortcuts)
│   └── finalize.rs         # Final verification and metadata
├── uninstaller.rs          # Cross-platform uninstaller
└── lib.rs                  # Public API
```

## Testing Platform-Specific Code

### Windows
```powershell
# Test installation
cargo run --release

# Verify registry
reg query HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Pulsar

# Check Start Menu
ls "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Pulsar.lnk"
```

### macOS
```bash
# Test installation
cargo run --release

# Verify bundle
ls -la ~/Applications/Pulsar.app/Contents/

# Check Info.plist
plutil -p ~/Applications/Pulsar.app/Contents/Info.plist

# Test launch
open ~/Applications/Pulsar.app
```

### Linux
```bash
# Test installation
cargo run --release

# Verify binary
ls -la ~/.local/bin/pulsar

# Check desktop entry
cat ~/.local/share/applications/pulsar.desktop

# Verify desktop integration
desktop-file-validate ~/.local/share/applications/pulsar.desktop

# Update database manually if needed
update-desktop-database ~/.local/share/applications/
```

## Why This Approach is Correct

### Windows
- **%LOCALAPPDATA%** is the correct user-level install location per Microsoft guidelines
- **Registry ARP entries** are how Windows tracks installed applications
- **Start Menu shortcuts** are the primary discovery mechanism (not PATH)

### macOS
- **.app bundles** are the only correct way to distribute macOS applications
- **Info.plist** is mandatory for Launch Services recognition
- **Launch Services** automatically handles registration - no manual work needed

### Linux
- **freedesktop.org specs** ensure compatibility across all major desktop environments
- **.desktop files** are the standard application registration mechanism
- **hicolor icon theme** is universally supported

## Migration from Old Code

The old implementation had several issues:

1. **Wrong install locations**:
   - Windows: Used Program Files (requires elevation)
   - Linux: Used ~/.local/share/pulsar (not a binary location)

2. **Missing OS registration**:
   - Windows: No registry entries
   - macOS: No Info.plist
   - Linux: No desktop entries

3. **Cross-platform abstractions** that hid platform differences

4. **Placeholder implementations** that didn't actually work

The new implementation completely replaces these with correct, working, OS-native code.
