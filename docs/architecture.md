# Architecture

## Overview

```mermaid
graph TD
    Scheduler["Scheduler\n(Tokio task)"]
    NTP["NTP module"]
    Frontend["Frontend\n(Vite / TS)"]
    Overlay["OverlayController"]
    Commands["commands.rs\n(IPC)"]
    State["AppState\nArc&lt;Mutex&lt;Inner&gt;&gt;\nsettings · skip_date · ceremony_active"]
    Platform["Platform layer\ncfg-gated"]
    Windows["platform_windows\nSendInput / SMTC"]
    Linux["platform_linux\nxdotool / MPRIS"]

    Scheduler -->|emit| Frontend
    Frontend --> Overlay
    Frontend -->|invoke| Commands
    Scheduler --> NTP
    Scheduler --> State
    Commands --> State
    State --> Platform
    Platform --> Windows
    Platform --> Linux
```

## Key design decisions

### Why Tauri instead of Electron?
Tauri's Rust backend gives us direct access to Win32 and Linux system APIs
without an extra IPC layer, and the resulting binary is ~5 MB vs ~150 MB for
an equivalent Electron app.

### Shared state via `Arc<Mutex<Inner>>`
The scheduler runs as a long-lived `tokio` task on the async runtime.  Tauri
commands run on the Tauri thread pool.  A single `Arc<Mutex<Inner>>` wrapped
in the `AppState` newtype is the simplest correct approach for this scale.

### Why `SendInput(VK_MEDIA_PLAY_PAUSE)` instead of per-process muting?
`VK_MEDIA_PLAY_PAUSE` works for every media app without requiring per-app
integration.  `IAudioSessionControl` (Core Audio) is available as a future
enhancement for cases where targeted muting is needed without pausing.

### Settings persistence
Settings are serialised as pretty-printed JSON to the platform config
directory (`%APPDATA%\minute-of-silence\settings.json` on Windows,
`~/.config/minute-of-silence/settings.json` on Linux).  No registry, no
SQLite — a single file is sufficient and trivially inspectable.

## Data flow: ceremony trigger

```
Scheduler loop (every 10 s)
  └─ current_local_time()          ← NTP-corrected or system clock
       └─ is_within_window()       ← [09:00, 09:00 + grace)
            └─ trigger_ceremony()
                 ├─ platform::media::pause_all()
                 ├─ emit("ceremony:start")   → Frontend shows overlay
                 ├─ audio engine plays preset (TODO)
                 └─ finish_ceremony()
                      ├─ platform::media::resume_all()
                      └─ emit("ceremony:end")  → Frontend hides overlay
```
