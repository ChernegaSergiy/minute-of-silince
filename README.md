# Minute of Silence

[![CI](https://github.com/ChernegaSergiy/minute-of-silence/actions/workflows/ci.yml/badge.svg)](https://github.com/ChernegaSergiy/minute-of-silence/actions)
[![License: CSSM Unlimited License v2.0](https://img.shields.io/badge/License-CSSM%20Unlimited%20License%20v2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-black)](https://github.com/ChernegaSergiy/minute-of-silence/releases)

A lightweight desktop application for the daily Minute of Silence at 09:00.  
Built with [Tauri 2](https://tauri.app) (Rust + TypeScript).

## Overview

Minute of Silence runs silently in the system tray and activates every day at 09:00. When the time comes, it pauses any playing media, plays an audio sequence according to the selected preset, and displays a full-screen visual indicator. Once the ceremony concludes, media playback is restored automatically.

The trigger time is corrected against an NTP server, ensuring accuracy regardless of local clock drift. A one-shot skip option allows the user to suppress the next activation without disabling the feature permanently.

## Features

- **Automatic Daily Activation**: Activates at 09:00 with NTP time correction.
- **Five Audio Presets**: Voice announcement combined with silence, bell, metronome, and/or national anthem.
- **Media Management**: Pauses Spotify, browser video, VLC, and other players before the ceremony; resumes them after.
- **Visual Overlay**: A full-screen indicator appears on top of all windows during the ceremony.
- **Skip Next**: Suppresses a single upcoming activation via the tray menu or the main window.
- **Post-Sleep Handling**: If the system wakes from sleep after 09:00, a configurable grace window decides whether to activate late or skip.
- **Persistent Settings**: Stored as JSON in the platform config directory; no registry writes.
- **Autostart on Login**: Registers with the OS login mechanism on first launch.
- **Structured Logging**: Rotating log files written to the platform log directory.

## Audio Presets

| # | Preset | Description |
|---|--------|-------------|
| 1 | Voice + Silence + Bell | Announcement, 60 s of silence, closing bell |
| 2 | Voice + Anthem | Announcement followed by the national anthem |
| 3 | Voice + Metronome | Announcement with a metronome for the silence duration |
| 4 | Voice + Metronome + Anthem | Announcement, metronome, then anthem |
| 5 | Metronome only | No voice announcement |

> [!NOTE]
> Audio playback is not yet implemented. The scheduler, overlay, and media pause/resume are fully functional. Audio assets and the playback engine are planned for the next release.

## Installation

### Windows

Download the `.msi` or `.exe` installer from the [Releases](https://github.com/your-org/minute-of-silence/releases) page and run it. The application will start in the system tray and register itself for autostart.

### Linux (Ubuntu / Debian)

```bash
# Debian package
sudo dpkg -i minute-of-silence_0.1.0_amd64.deb

# AppImage
chmod +x minute-of-silence_0.1.0_amd64.AppImage
./minute-of-silence_0.1.0_amd64.AppImage
```

## Building from Source

### Prerequisites

| Tool | Minimum version |
|------|----------------|
| Rust | 1.75 |
| Node.js | 20 LTS |
| Tauri CLI | 2.x |

Install the Tauri CLI:

```bash
npm install -g @tauri-apps/cli
```

**Linux only** — install required system libraries:

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev libappindicator3-dev \
  librsvg2-dev patchelf libasound2-dev
```

### Development

```bash
git clone https://github.com/your-org/minute-of-silence.git
cd minute-of-silence
npm install
npm run tauri dev
```

### Release build

```bash
npm run tauri build
```

Artifacts are written to `src-tauri/target/release/bundle/`.

## Project Structure

```
minute-of-silence/
├── src/                        TypeScript frontend (Vite)
│   ├── api.ts                  Typed wrappers around Tauri invoke()
│   ├── app.ts                  Root UI controller
│   ├── overlay.ts              Full-screen ceremony overlay
│   └── types.ts                Shared types, mirrors Rust structs
├── src-tauri/
│   ├── src/
│   │   ├── core/
│   │   │   ├── scheduler.rs    Daily trigger loop with NTP correction
│   │   │   ├── ntp.rs          NTP offset query
│   │   │   └── settings.rs     Persistent settings and audio presets
│   │   ├── commands.rs         Tauri IPC command handlers
│   │   ├── tray.rs             System tray icon and context menu
│   │   ├── state.rs            Shared application state (Arc<Mutex>)
│   │   ├── error.rs            Unified error type
│   │   ├── platform_windows.rs Win32 API — media control, power events
│   │   └── platform_linux.rs   MPRIS / xdotool — media control
│   └── tests/                  Rust integration tests
├── docs/
│   └── architecture.md         System design and data flow
├── .github/
│   ├── workflows/ci.yml        CI/CD pipeline (lint, test, build)
│   └── ISSUE_TEMPLATE/         Bug report and feature request forms
├── CHANGELOG.md
├── CONTRIBUTING.md
└── index.html                  App shell with embedded CSS
```

## Contributing

Contributions are welcome and appreciated! Here's how you can contribute:

1. Fork the project
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

Please make sure to update tests as appropriate and adhere to the existing coding style.

## License

This library is licensed under the CSSM Unlimited License v2.0 (CSSM-ULv2). See the [LICENSE](LICENSE) file for details.
