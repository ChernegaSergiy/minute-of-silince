# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

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
