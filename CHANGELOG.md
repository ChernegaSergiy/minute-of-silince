# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

## [0.9.1] - 2026-05-18

### Added
- UI theme preference: new `useSystemTheme` (follow system) and `uiTheme` (`light` | `dark`) settings exposed to the frontend and persisted by the backend. A new control in the Settings tab allows choosing to follow the system theme or select Light/Dark manually.
- Frontend support for `useSystemTheme`/`uiTheme` in `types.ts`, `App.tsx`, and `SettingsTab.tsx` so the UI respects the persisted preference and system changes.

### Changed
- Memoize Fluent theme creation in the frontend to avoid recreating theme objects on every render and reduce unnecessary reflows.
- i18n: added strings for the new theme controls in `en.json` and `uk.json`.

### Fixed
- Minor localization/encoding fixes in `uk.json` and `snapcraft.yaml`.

## [0.9.0] - 2026-05-17
 
### Added
- `ChangelogTab` with infinite scroll — changelog is parsed from `CHANGELOG.md` at build time via Vite `?raw` import, no network requests required.
- `tabs.changelog` i18n key (uk: `ЗМІНИ`, en: `CHANGELOG`).
- Section headings in `SettingsTab` via new `sections.*` i18n keys.

### Changed
- **Frontend fully migrated from FAST/web-components to Fluent UI React v9** (`@fluentui/react-components`, `@fluentui/react-icons`, `react`, `react-dom`, `react-markdown`, `@vitejs/plugin-react` added).
- UI split into dedicated React component files: `App.tsx`, `SettingsTab.tsx`, `AboutTab.tsx`, `Overlay.tsx`, `ChangelogTab.tsx` (lazy-loaded).
- Navigation replaced with Fluent `NavDrawer` + `NavItem` with icons.
- Main window size increased from 360×460 to 640×520 (min 500×400).
- Autostart sync on startup made non-blocking via `tauri::async_runtime::spawn_blocking`.
- Flatpak `.desktop` and `metainfo.xml`: English set as default locale, Ukrainian as `[uk]` variant — aligns with AppStream/Flathub conventions.
- Flatpak CI: added `ppa:flatpak/stable` to fix lib64 build failures on GitHub Actions.
- MSIX `appxmanifest.xml`: `uap10:Parameters="--hidden"` added to `startupTask`; `DisplayName` attribute removed from `StartupTask`.

### Fixed
- `sync_autostart_from_system` no longer blocks the Tauri `setup` hook, preventing a potential startup hang.

## [0.8.5] - 2026-05-16
 
### Added
- `AnthemVoice` enum (`Default`, `MykhailoKhoma`, `OleksandrPonomarov`) with `snake_case` serialization.
- `anthem_voice` field in `Settings` struct (default: `AnthemVoice::Default`).
- Anthem audio performances by Mykhailo Khoma and Oleksandr Ponomarov.
- `AnthemVoice` parameter in `play_preset` / `AudioEngine` to select the correct anthem file at playback time.
- Anthem voice selection dropdown in the settings UI; hidden automatically when the active preset has no anthem segment.
- `AnthemVoice` TypeScript type in `types.ts`.
- i18n strings for anthem voice control (`controls.anthem_voice.*`) in `en.json` and `uk.json`.
- `sync_autostart` Tauri command for explicit autostart synchronisation on demand.
- `apply_autostart_enabled` and `sync_autostart_from_system` helpers extracted into `platform/mod.rs`.

### Changed
- Autostart state is now synchronised with the actual system state at application startup (`lib.rs`) instead of being applied blindly from the persisted value.
- `get_settings` command syncs the persisted autostart flag against the real platform state before returning, keeping the UI consistent with external changes (e.g. system Settings toggles).

### Fixed
- Flag animation window event listeners now use `once` instead of `listen`, preventing listener accumulation across repeated ceremonies.
- Flag animation window was not closing after ceremony end: the global `on_window_event` handler intercepted `CloseRequested` on all windows and called `hide()` instead of closing. Handler is now scoped to the main window only.

## [0.8.4] - 2026-05-13

### Added
- Flag animation window (`flag-animation.html`) — a fullscreen animated Ukrainian flag rendered via canvas that appears during the anthem segment of the ceremony.
- New `showFlagAnimation` setting (default: `false`) to toggle the flag animation independently of the visual overlay.
- `AudioPreset::has_anthem()` helper to check whether a preset includes the anthem segment.
- `anthem-start` / `anthem-end` Tauri events emitted by `AudioEngine` for precise flag window lifecycle control.
- MSIX packaged `StartupTask` (`uap5` extension) in `appxmanifest.xml` for proper autostart support on packaged Windows builds.

### Changed
- Windows autostart for MSIX now uses `ApplicationModel::StartupTask` API instead of a Startup-folder `.lnk` shortcut; `mslnk` dependency removed.
- `winreg` dependency bumped from `0.55` to `0.56`.
- Settings struct now derives `#[serde(default)]` so missing fields in persisted JSON fall back gracefully.
- Visual overlay toggle now also controls the visibility of the flag animation row in the UI.
- Dev dependency updates: `@tauri-apps/cli` → 2.11.1, `vite` → 8.0.12, `typescript-eslint` → 8.59.3, `rolldown` → 1.0.0 (stable).

### Fixed
- Anthem playback timing: metronome end is now detected by waiting for the player queue rather than a hardcoded 30-second sleep, ensuring the anthem starts precisely when the metronome finishes.

## [0.8.3] - 2026-05-12

### Changed
- MSIX autostart now uses Startup-folder shortcut (`.lnk` file) instead of manifest extension, providing better compatibility and control.

### Fixed
- Removed invalid MSIX startup task extension from manifest that was using incorrect `desktop` namespace approach.

## [0.8.2] - 2026-05-12

### Fixed
- MSIX startup task now uses correct `desktop` namespace instead of invalid `uap5` for FullTrust applications.

## [0.8.1] - 2026-05-11

### Added
- Adaptive tray icon that automatically switches between light and dark versions to match the system theme.
- Real-time theme watching for Windows (via registry) and Linux (via XDG Desktop Portal).
- Unified system theme detection supporting Windows, GNOME, and KDE Plasma.
- Specialized handling for GNOME to ensure tray icon visibility on its dark top panel.

### Changed
- Replaced the static tray icon with theme-aware assets (`-light` and `-dark` versions).

## [0.8.0] - 2026-05-10

### Added
- Flatpak support with autostart via `.desktop` file in `~/.config/autostart/`.
- Unified autostart detection for Snap, Flatpak, and standard installations.

### Fixed
- Tray icon now uses dedicated `tray-icon-32.png` instead of default window icon.

### Changed
- MSIX identifier changed to `ua.pp.khvylyna.MinuteOfSilence` for proper store submission.
- Binary name changed from `minute-of-silence` to `MinuteOfSilence`.

## [0.7.8] - 2026-05-06

### Added
- Optional "Resume playback" setting to restore paused media after the ceremony.
- Targeted media handling: the app now tracks which specific players were paused and only resumes them if they are still paused.
- Real-time post-sleep handling on Windows via `WM_POWERBROADCAST` hook to trigger immediate clock sync upon resume.

### Fixed
- "Weekdays only" logic now correctly applied to both the ceremony trigger and notifications.
- Improved test coverage for `Silence` and `VoiceMetronomeEnding` presets in `settings_test.rs`.

### Changed
- Cleaned up redundant Win32 features in `Cargo.toml`.
- Updated dependencies: Tauri v2.11.0, Tokio v1.52.2, Vite v8.0.10.

## [0.7.7] - 2026-05-03

### Added
- Overlay text i18n support (title and subtitle localized).
- Windows MSIX localization for app name in system language.

### Changed
- Updated appxmanifest to use ms-resource:AppName for localized DisplayName.

## [0.7.6] - 2026-04-30

### Added
- Window title localization (uk/en).
- NTP status strings localization.

### Changed
- Improved locale detection using navigator.language.

## [0.7.5] - 2026-04-30

### Changed
- Platform code refactored: `platform_windows.rs` + `platform_linux.rs` → `platform/{linux,windows}/` directories.
- `commands.rs` + `tray.rs` moved to `app/` module.
- `CeremonyManager` moved to separate `core/ceremony.rs` file.
- `is_msix()` function moved to `platform/mod.rs`.

## [0.7.4] - 2026-04-29

### Changed
- Removed redundant Snap interfaces (`home`, `login-session-observe`, `upower-observe`, `hardware-observe`, `system-observe`) to streamline Store approval.
- Cleaned up `plugs` list in `snapcraft.yaml`.

## [0.7.3] - 2026-04-29

### Added
- Robust manual Snap autostart management via `.desktop` file in `$SNAP_USER_DATA/.config/autostart`.
- `home` and `login-session-observe` plugs to `snapcraft.yaml` for better integration and autostart support.

### Fixed
- Snap autostart logic to use official `snapd` mechanism for confined applications.
- `StartupWMClass` in desktop file to match application binary name.

### Changed
- Moved Snap autostart logic to a dedicated `platform_linux::autostart` module.
- Bumped dependencies: `i18next`, `typescript`, `typescript-eslint`, `eslint`, `zbus`.
- Updated `Cargo.lock` and `package-lock.json`.

## [0.7.2] - 2026-04-28

### Added
- Window state preservation before ceremony and restoration after ceremony ends.

### Fixed
- Window permissions for minimize/hide operations in MSIX packages.

### Changed
- Updated version to 0.7.2.

## [0.7.1] - 2026-04-26

### Added
- Auto-show window when ceremony starts, even if minimized or hidden.
- Window permissions for MSIX packages (unminimize/show/set-focus).

### Fixed
- Ceremony-end flashing when restarting ceremony multiple times via test button.
- Ceremony status now updates before command returns.
- Fixed "повноекранний екран" → "оверлей вшанування на весь екран" in Ukrainian locale.

### Changed
- Moved `is_msix_package()` to dedicated `is_msix.rs` module.
- Removed unused Task Scheduler autostart functions from `platform_scheduler_task.rs`.
- Updated project tree in README.md.

## [0.7.0] - 2026-04-24

### Added
- MSIX Toast Notifications support via WinRT API for Microsoft Store packages.

### Fixed
- Reminder notifications not working in MSIX packages due to AppContainer restrictions.

## [0.6.6] - 2026-04-23

### Added
- MSIX StartupTask extension for autostart in Microsoft Store packages.
- Windows Task Scheduler module (platform_scheduler_task.rs).

### Changed
- Improved voice announcements from Sonia Sotnyk and Daria Khomutovskyi.
- Version bump to 0.6.6.
- Bump i18next, typescript-eslint, tokio, and vitest dependencies.

### Fixed
- Ceremony not triggering in grace window when started after 09:00.
- Audio file discovery in Snap sandbox.

## [0.6.5] - 2026-04-19

### Added
- New ending voice from Bohdan Hdal with "Slava Ukrajini".

### Changed
- Updated ending voice for Bohdan Hdal to use new "Slava Ukrajini" recording.

### Removed
- Removed unused ending_heroes.ogg file.

## [0.6.4] - 2026-04-19

### Added
- Voice selection for announcement with 4 options:
  - Bohdan Hdal (original)
  - Sonia Sotnyk (Vshanui)
  - Dania Khomutovskyi (Vshanui)
  - Air Alert app
- New audio presets with endings: "Voice + metronome + ending"
- Ending voice files: "Vichna slava Herojam" and "Slava Ukrajini"

### Fixed
- Voice duration now uses selected voice instead of hardcoded announcement.
- Wait for voice announcement to finish before playing metronome.
- Skip ending for Air Alert voice in VoiceMetronomeEnding preset.
- Voice durations cached at startup to avoid repeated file reads.

## [0.6.3] - 2026-04-17

### Added
- Precise timing for presets with voice announcement to ensure silence starts at exactly 09:00.

### Changed
- Audio duration detection now uses symphonia for accurate OGG metadata reading.
- Updated Linux audio dependencies: alsa 0.11 and cpal 0.17.3.

### Fixed
- Removed unnecessary dynamic import of syncNtpNow in frontend.

### Removed
- Unused audio files from the package.

## [0.6.2] - 2026-04-15

### Added
- New audio preset that shows visual overlay and pauses other players without playing any sound.
- Preset names are now translatable via i18n system.

## [0.6.1] - 2026-04-15

### Fixed
- Fixed "immediately" option not sending notifications.

## [0.6.0] - 2026-04-15

### Added
- Added credits to Bohdan Hdal's original mobile app in README.md and About tab.

### Dependencies
- Updated Rust dependencies: rust-i18n 4.0 (up to 3x performance improvement), windows-future 0.3.2, tokio 1.52. MSRV updated to Rust 1.80.

## [0.5.3] - 2026-04-14

### Fixed
- Audio playback in MSIX packages: switched to Tauri's native path resolver for cross-platform resource paths.

### Refactored
- Frontend version display now uses Tauri's `getVersion()` API instead of duplicating version in `package.json`.

## [0.5.2] - 2026-04-14

### Added
- Microsoft Store identity and localized display name ("Хвилина мовчання") for official Store distribution.

## [0.5.1] - 2026-04-12

### Added
- Full localization (i18n) with Ukrainian and English support via `i18next` and `rust-i18n`.

### Changed
- Snap audio handling: refactored to use native PulseAudio/PipeWire bridge for reliable playback.
- Unified application identifiers and fixed icon paths for correct display in Linux menus and system trays.
- Updated official website URL to `khvylyna.pp.ua`.

### Fixed
- Windows build compatibility with latest Windows SDK (windows-rs v0.62).
- Media control reliability: asynchronous pausing prevents backend panics on Linux.
- UI badge localization and dashboard status indicator updates.

### Technical
- Bumped dependencies: `zbus` (v5.14.0), `cpal` (v0.17.1), `windows` (v0.62.2).

## [0.5.0] - 2026-04-07

### Added
- System reminder notifications before the ceremony (configurable: immediately or up to 10 minutes before).
- User interface controls for enabling and configuring reminder notifications.

### Changed
- Improved Windows media pausing by migrating from legacy `WM_APPCOMMAND` to the modern `GlobalSystemMediaTransportControlsSessionManager` API for reliable playback control.
- Bumped project version to 0.5.0 across all configuration files.

### Fixed
- Fixed window restoration from the system tray by explicitly unminimizing the window before showing it.

## [0.4.2] - 2026-04-05

### Changed
- Windows media pause: switched from `SendInput(VK_MEDIA_PLAY_PAUSE)` to `WM_APPCOMMAND(APPCOMMAND_MEDIA_PAUSE)` for more reliable pausing.
- Media players no longer resume after the ceremony ends — they stay paused.

### Fixed
- Fixed missing `zbus::proxy` import in platform_linux.rs.

### Added
- MSIX packaging support via `winapp` CLI tool.
- Application assets (icons, tiles) for Microsoft Store distribution.
- `appxmanifest.xml` and `winapp.yaml` configuration files.

## [0.4.1] - 2026-04-04

### Fixed
- Resolved 'Connection refused' error on Snap autostart by using native Snap command wrapper.
- Fixed Snap build validation by switching to a Proprietary license in snapcraft.yaml.
- Simplified autostart logic and resolved duplicate argument issues in Linux environments.

### Added
- Rich README layout with application screenshots for better visual documentation.
- Official screenshot assets stored in the repository.

### Changed
- Bumped project version to 0.4.1 across all configuration files.
- Synchronized Cargo.lock and project dependencies.
- Improved code style and formatting in src-tauri/src/lib.rs.

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
