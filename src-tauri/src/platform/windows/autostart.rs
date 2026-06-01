//! Windows autostart management for packaged MSIX builds.
//!
//! MSIX uses a packaged StartupTask so Windows can manage registration and
//! cleanup automatically when the app is removed.

use crate::{AppError, error::Result};

use windows::ApplicationModel::StartupTask;
use windows::core::HSTRING;

const STARTUP_TASK_ID: &str = "MinuteOfSilenceStartupTask";

/// Enable autostart through the packaged MSIX startup task.
#[allow(dead_code)]
pub fn enable_autostart() -> Result<()> {
    let task = StartupTask::GetAsync(&HSTRING::from(STARTUP_TASK_ID))
        .map_err(|e| AppError::Platform(e.to_string()))?
        .join()
        .map_err(|e| AppError::Platform(e.to_string()))?;

    let _ = task
        .RequestEnableAsync()
        .map_err(|e| AppError::Platform(e.to_string()))?
        .join()
        .map_err(|e| AppError::Platform(e.to_string()))?;

    log::info!("Autostart enabled via packaged startup task: {STARTUP_TASK_ID}");
    Ok(())
}

/// Disable autostart through the packaged MSIX startup task.
#[allow(dead_code)]
pub fn disable_autostart() -> Result<()> {
    let task = StartupTask::GetAsync(&HSTRING::from(STARTUP_TASK_ID))
        .map_err(|e| AppError::Platform(e.to_string()))?
        .join()
        .map_err(|e| AppError::Platform(e.to_string()))?;

    task.Disable()
        .map_err(|e| AppError::Platform(e.to_string()))?;

    log::info!("Autostart disabled via packaged startup task: {STARTUP_TASK_ID}");
    Ok(())
}

/// Returns the current autostart state reported by the platform.
pub fn system_autostart_enabled() -> Option<bool> {
    if !crate::platform::is_msix() {
        None
    } else {
        use windows::ApplicationModel::StartupTaskState;

        let task = StartupTask::GetAsync(&HSTRING::from(STARTUP_TASK_ID))
            .ok()?
            .join()
            .ok()?;
        let state = task.State().ok()?;
        Some(state == StartupTaskState::Enabled)
    }
}

/// Apply the requested autostart state to the current platform.
pub fn apply_autostart_enabled(app: &tauri::AppHandle, enabled: bool) {
    if crate::platform::is_msix() {
        if enabled {
            if let Err(e) = enable_autostart() {
                log::error!("Failed to enable autostart for MSIX: {}", e);
            }
        } else {
            if let Err(e) = disable_autostart() {
                log::error!("Failed to disable autostart for MSIX: {}", e);
            }
        }
    } else {
        use tauri_plugin_autostart::ManagerExt;
        let autostart_manager = app.autolaunch();
        if enabled {
            let _ = autostart_manager.enable();
        } else {
            let _ = autostart_manager.disable();
        }
    }
}
