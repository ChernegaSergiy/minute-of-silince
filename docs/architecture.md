# Architecture

## Overview

```mermaid
graph TD
    Scheduler["Scheduler\n(Tokio task)"]
    NTP["NTP module"]
    Frontend["Frontend\n(Vite / TS)"]
    Overlay["OverlayController"]
    Commands["commands.rs\n(IPC)"]
    State["AppState\nArc<Mutex<Inner>>\nsettings · skip_date · ceremony_active"]
    Platform["Platform layer\ncfg-gated"]
    Windows["platform_windows\nSMTC / Win32"]
    Linux["platform_linux\nMPRIS (zbus)"]
    Macos["platform_macos\nNSWorkspace / AppleScript"]

    Scheduler -->|emit| Frontend
    Frontend --> Overlay
    Frontend -->|invoke| Commands
    Scheduler --> NTP
    Scheduler --> State
    Commands --> State
    State --> Platform
    Platform --> Windows
    Platform --> Linux
    Platform --> Macos
```

## Key design decisions

### Why Tauri instead of Electron?
Tauri's Rust backend gives us direct access to Win32, Linux, and macOS system APIs
without an extra IPC layer, and the resulting binary is ~5 MB vs ~150 MB for
an equivalent Electron app.

### Shared state via `Arc<Mutex<Inner>>`
The scheduler runs as a long-lived `tokio` task on the async runtime.  Tauri
commands run on the Tauri thread pool.  A single `Arc<Mutex<Inner>>` wrapped
in the `AppState` newtype is the simplest correct approach for this scale.

### Targeted media pausing instead of system-wide key emulation
Earlier designs simulated media key events (such as `VK_MEDIA_PLAY_PAUSE`), but this was prone to toggling state on already-paused players or failing under certain window focus conditions. Now, the app uses native platform APIs to query active media sessions and pause/resume them selectively:
- **Windows:** System Media Transport Controls (`GlobalSystemMediaTransportControlsSessionManager`)
- **Linux:** MPRIS D-Bus interface (`zbus`)
- **macOS:** AppleScript control of running media apps (`osascript` via `NSWorkspace`)
Active players are tracked dynamically and restored precisely when the ceremony ends.

### Settings persistence
Settings are serialised as pretty-printed JSON to the platform config
directory:
- **Windows:** `%APPDATA%\minute-of-silence\settings.json`
- **Linux:** `~/.config/minute-of-silence/settings.json`
- **macOS:** `~/Library/Application Support/minute-of-silence/settings.json`

### NTP Synchronization Strategy
The app supports both system clock and NTP-corrected time. 
A manual synchronization feature is provided via a dedicated `sync_ntp_now` 
IPC command that updates the shared state and triggers immediate correction.

### Native Look & Feel
To ensure the application feels like a native desktop tool:
- **Text Selection:** Disabled globally via CSS.
- **Context Menu:** Standard browser context menu is blocked.
- **Typography:** Uses a high-quality monospace font stack (Ubuntu Mono, JetBrains Mono, etc.).

## Data flow: ceremony trigger

```
Scheduler loop (every 1 s)
  ├─ sync_ntp() if needed          ← hourly or on sleep resume
  └─ get_synchronized_now()        ← NTP-corrected or system clock
       └─ Should trigger today?
            ├─ Compensation window: [09:00 - compensation, 09:00)  ← voice/bell early start
            └─ Grace window: [09:00, 09:00 + grace_minutes)        ← late start / wake up
                 └─ trigger_ceremony() (CeremonyManager)
                      ├─ Set ceremony_active = true & last_activation = now
                      ├─ Setup flag animation webview window (if enabled & preset has anthem)
                      ├─ emit("ceremony-start", { duration_ms })   → Frontend shows overlay
                      ├─ platform.pause_media()                    → Pause other players (if enabled)
                      ├─ Auto-unmute & set system volume priority
                      ├─ audio_engine.play_preset(...)             → Runs on a dedicated thread
                      └─ finish_ceremony()                         ← Called when audio ends
                           ├─ Restore system volume & mute states
                           ├─ platform.resume_media(...)           → Resume players (if enabled)
                           ├─ Clear settings.skip_date from store
                           └─ emit("ceremony-end")                 → Frontend hides overlay
```
