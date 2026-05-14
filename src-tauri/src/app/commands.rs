//! Tauri IPC commands exposed to the frontend via `invoke()`.

use tauri::{AppHandle, State};

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
pub fn skip_next(state: State<'_, AppState>) {
    let tomorrow = (chrono::Local::now() + chrono::Duration::days(1)).date_naive();
    state.lock().skip_date = Some(tomorrow);
    log::info!("Next ceremony skipped (date: {tomorrow})");
}

/// Remove the skip flag for the next calendar day.
#[tauri::command]
pub fn unskip_next(state: State<'_, AppState>) {
    state.lock().skip_date = None;
    log::info!("Skip for next ceremony removed");
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
