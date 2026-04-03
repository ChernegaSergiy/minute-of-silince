# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

## [0.4.0] - 2026-04-03

### Added
- Visual ceremony overlay with brutalist animations and styles for a more immersive experience.
- `show_visual_overlay` toggle in settings to enable or disable the ceremonial screen.
- Comprehensive Flatpak support with dedicated manifests for Flathub submission.
- AppStream metadata and desktop entry for better integration with Linux software centers.

### Changed
- Upgraded Flatpak runtime to GNOME 50 for improved performance and modern libraries.
- Bumped project version to 0.4.0 across all configuration files.

## [0.3.1] - 2026-04-03

### Fixed
- Resolved Critical Segmentation Fault in Snap package by removing duplicate runtime libraries.
- Fixed `libpxbackend` loading error by adding `glib-networking` to stage-packages.
- Corrected Snap build environment to properly link against GNOME SDK on Ubuntu 24.04 (core24).

### Added
- Integrated `.desktop` entry for proper application visibility in Linux app menus.
- Automatic exclusion of Snap build artifacts and logs in `.gitignore`.

### Changed
- Refocused Snap support on `amd64` architecture for better stability and testing.
- Bumped project version to 0.3.1 across all configuration files.

## [0.3.0] - 2026-04-02

### Added
- Detailed descriptions and info blocks for all settings in the UI for better clarity.
- Auto-unmute feature with a dedicated toggle to restore system sound during the ceremony.
- Ceremony enabled toggle to allow temporary deactivation of the daily schedule.
- Configurable late-start grace window (0-5 minutes) for handling system wake-up after 09:00.
- Snapcraft configuration for Ubuntu App Center with support for amd64 and arm64 architectures.
- Single application instance enforcement (automatically focuses the existing window on relaunch).
- Dynamic frontend versioning based on `package.json`.
- Integrated official application icon for Snap and Linux desktop environments.

### Fixed
- Stabilized NTP synchronization by using Google and Cloudflare servers as reliable fallbacks.
- Fixed autostart plugin configuration by removing the unhandled `--hidden` argument.
- Improved media handling: ensured players are paused before unmuting on Windows.
- Fixed synchronization between the tray menu and the settings window for the "Skip Next" status.
- Resolved audio engine stability issues by migrating to `rodio 0.22` with proper stream handling.
- Fixed various build-time issues, including invalid icon fields in Tauri configuration.

### Changed
- Migrated to Vite 8 and ESLint 10 (Flat Config) for improved development workflow.
- Refactored the CI/CD pipeline into a unified `ci.yml` with comprehensive build and test stages.
- Moved UI styles to a dedicated `style.css` file for better maintainability.
- Updated project dependencies (thiserror, dirs, windows-rs, etc.) to their latest versions.

## [0.2.0] - 2026-03-29

### Added
- Dedicated "About" tab in the user interface with project information.
- Manual NTP synchronization button for immediate time correction.
- Visual indicator for unsaved settings (color change and asterisk).
- Official application logo in the "About" section and updated system icons.
- Clickable repository link using `tauri-plugin-shell`.
- Integrated audio playback engine using `rodio` (backend-driven).

### Changed
- Switched to a high-quality monospace font stack (Ubuntu Mono, JetBrains Mono).
- Expanded tab buttons to full width for a more balanced layout.
- Reduced default late-start grace window from 5 minutes to 1 minute.
- Optimized internal scheduler loop frequency to 1 second.

### Fixed
- Disabled text selection and browser context menu for a native desktop feel.
- Improved UI status synchronization after saving settings.
- Resolved numerous Rust clippy warnings and formatting issues.

## [0.1.0] - 2026-03-29

### Added
- Initial project scaffold (Tauri 2 + Rust + TypeScript/Vite).
- Daily scheduler with NTP time correction and configurable late-start grace window.
- Five audio presets: Voice+Silence+Bell, Voice+Anthem, Voice+Metronome, Voice+Metronome+Anthem, Metronome-only.
- System Tray icon with context menu (Open, Skip Tomorrow, Quit).
- Persistent JSON settings stored in the platform config directory.
- Windows media pause/resume via `SendInput(VK_MEDIA_PLAY_PAUSE)`.
- Linux media pause/resume via `xdotool` fallback (MPRIS D-Bus planned).
- Visual ceremony overlay (brutalist full-screen indicator).
- `WM_POWERBROADCAST` handling for post-sleep scheduler correction (Windows).
- Autostart on system login via `tauri-plugin-autostart`.
- Structured logging with log rotation via `tauri-plugin-log`.
- CI/CD pipeline on GitHub Actions (lint, test, build for Windows + Linux).
- Conventional Commits enforcement documented in CONTRIBUTING.md.

### Notes
- Audio playback engine is **not yet implemented**; the scheduler fires events
  and the overlay appears, but no sound is produced in this release.
