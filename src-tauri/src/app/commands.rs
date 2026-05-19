//! Tauri IPC commands exposed to the frontend via `invoke()`.

use std::{
    collections::VecDeque,
    fs,
    io::{BufRead, BufReader},
};

use tauri::{AppHandle, Manager, State};

#[allow(unused_imports)]
use crate::{
    core::settings::Settings,
    state::{AppState, StatusSnapshot},
    AppError, Result,
};

// Settings

/// Return the current settings snapshot.
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.lock().settings.clone()
}

/// Sync the persisted autostart setting with the actual platform state.
#[tauri::command]
pub fn sync_autostart_from_system(state: State<'_, AppState>) -> Result<()> {
    crate::platform::sync_autostart_from_system(state)
}

/// Persist updated settings and apply side-effects (e.g. autostart toggle).
#[tauri::command]
#[allow(unused_variables)]
pub fn save_settings(app: AppHandle, state: State<'_, AppState>, settings: Settings) -> Result<()> {
    // Persist to disk.
    settings.save()?;

    // Apply autostart setting.
    crate::platform::apply_autostart_enabled(&app, settings.autostart_enabled);

    // Update in-memory state.
    state.lock().settings = settings.clone();

    // Trigger immediate NTP sync if system time is disabled.
    if !settings.system_time_only {
        let ntp = state.ntp_service.clone();
        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            let _ = ntp.sync().await;
            use tauri::Emitter;
            let _ = app_handle.emit("ntp-synced", ());
        });
    }

    log::info!("Settings saved");
    Ok(())
}

// Status

/// Return a lightweight runtime status snapshot.
#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> StatusSnapshot {
    state.get_snapshot()
}

// Skip / unskip

/// Skip the ceremony for the next calendar day.
#[tauri::command]
pub fn skip_next(state: State<'_, AppState>) -> Result<()> {
    let tomorrow = (chrono::Local::now() + chrono::Duration::days(1)).date_naive();
    let mut inner = state.lock();
    inner.settings.skip_date = Some(tomorrow);
    inner.settings.save()?;
    log::info!("Next ceremony skipped (date: {tomorrow})");
    Ok(())
}

/// Remove the skip flag for the next calendar day.
#[tauri::command]
pub fn unskip_next(state: State<'_, AppState>) -> Result<()> {
    let mut inner = state.lock();
    inner.settings.skip_date = None;
    inner.settings.save()?;
    log::info!("Skip for next ceremony removed");
    Ok(())
}

// Manual trigger

/// Force immediate NTP synchronization.
#[tauri::command]
pub async fn sync_ntp_now(state: State<'_, AppState>) -> Result<StatusSnapshot> {
    log::info!("Manual NTP sync requested");
    let _ = state.ntp_service.sync().await;
    Ok(state.get_snapshot())
}

/// Immediately trigger the ceremony (for testing / demonstration purposes).
#[tauri::command]
pub async fn trigger_ceremony_now(app: AppHandle) -> Result<()> {
    log::info!("Manual ceremony trigger requested");
    crate::core::scheduler::trigger_now(app).await;
    Ok(())
}

/// Finish the ceremony early (called by frontend when audio playback completes).
#[tauri::command]
pub async fn finish_ceremony_now(app: AppHandle) -> Result<()> {
    log::info!("Ceremony finish requested by audio engine");
    let platform = crate::platform::get_platform();
    crate::core::CeremonyManager::finish_ceremony(app, platform).await;
    Ok(())
}

/// Return debug info and a tail of the latest application log file.
#[tauri::command]
pub fn get_log_contents(app: AppHandle) -> Result<String> {
    const MAX_LOG_TAIL_LINES: usize = 200;

    let log_dir = app.path().app_log_dir()?;
    let package = app.package_info();
    let log_path = log_dir.join(format!("{}.log", package.name));

    let mut lines = vec![
        format!("app: {}", package.name),
        format!("version: {}", package.version),
        format!("os: {}", std::env::consts::OS),
        format!("arch: {}", std::env::consts::ARCH),
        format!("log_dir: {}", log_dir.display()),
    ];

    if log_path.is_file() {
        let file = fs::File::open(&log_path)?;
        let reader = BufReader::new(file);
        let mut tail = VecDeque::with_capacity(MAX_LOG_TAIL_LINES);

        for line in reader.lines() {
            let line = line?;
            if tail.len() == MAX_LOG_TAIL_LINES {
                tail.pop_front();
            }
            tail.push_back(line);
        }

        lines.push(format!("log_file: {}", log_path.display()));
        lines.push(format!("lines_copied: {}", tail.len()));
        lines.push(String::from("---"));
        lines.extend(tail);
    } else {
        lines.push(String::from("log_file: not found"));
        lines.push(String::from("lines_copied: 0"));
        lines.push(String::from("---"));
    }

    Ok(lines.join("\n"))
}
