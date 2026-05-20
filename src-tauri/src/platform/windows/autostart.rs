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
