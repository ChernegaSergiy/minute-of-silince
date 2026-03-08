# Architecture

## Overview

```
┌─────────────────────────────────────────────────────┐
│                    Tauri Process                     │
│                                                      │
│  ┌──────────────┐        ┌──────────────────────┐   │
│  │  Scheduler   │──emit──▶   Frontend (Vite/TS)  │   │
│  │  (Tokio task)│        │   OverlayController   │   │
│  └──────┬───────┘        └──────────┬────────────┘   │
│         │                           │ invoke()        │
│         ▼                           ▼                 │
│  ┌──────────────┐        ┌──────────────────────┐   │
│  │  NTP module  │        │   commands.rs (IPC)   │   │
│  └──────────────┘        └──────────┬────────────┘   │
│                                     │                 │
│  ┌──────────────────────────────────▼─────────────┐  │
│  │               AppState (Arc<Mutex>)             │  │
│  │  settings · skip_date · ceremony_active · ...   │  │
│  └─────────────────────────────────────────────────┘  │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │       Platform layer (cfg-gated)              │   │
│  │  platform_windows  │  platform_linux          │   │
│  │  SendInput / SMTC  │  xdotool / MPRIS         │   │
│  └──────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
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
