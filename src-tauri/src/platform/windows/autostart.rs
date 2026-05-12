//! Windows autostart management — lightweight Startup-folder `.lnk` fallback.
//!
//! MSIX packages cannot write to HKCU Run; for self-signed MSIX we create a
//! shortcut in the user's Startup folder which is not virtualized.

use crate::{error::Result, AppError};

use std::path::Path;

/// Enable autostart by creating `MinuteOfSilence.lnk` in the user's Startup folder.
#[allow(dead_code)]
pub fn enable_autostart() -> Result<()> {
    let startup_dir = dirs::data_dir()
        .ok_or_else(|| AppError::Platform("Cannot locate AppData".into()))?
        .join("Microsoft\\Windows\\Start Menu\\Programs\\Startup");

    let exe_path = std::env::current_exe().map_err(|e| AppError::Platform(e.to_string()))?;
    let shortcut_path = startup_dir.join("MinuteOfSilence.lnk");

    create_shortcut(&exe_path, &shortcut_path)?;
    log::info!("Autostart enabled via startup folder: {:?}", shortcut_path);
    Ok(())
}

/// Disable autostart by removing the shortcut from the user's Startup folder.
#[allow(dead_code)]
pub fn disable_autostart() -> Result<()> {
    let startup_dir = dirs::data_dir()
        .ok_or_else(|| AppError::Platform("Cannot locate AppData".into()))?
        .join("Microsoft\\Windows\\Start Menu\\Programs\\Startup");

    let shortcut_path = startup_dir.join("MinuteOfSilence.lnk");

    if shortcut_path.exists() {
        std::fs::remove_file(&shortcut_path).map_err(|e| AppError::Platform(e.to_string()))?;
        log::info!("Autostart disabled, shortcut removed");
    }
    Ok(())
}

#[allow(dead_code)]
fn create_shortcut(target: &Path, shortcut: &Path) -> Result<()> {
    mslnk::ShellLink::new(target)
        .map_err(|e| AppError::Platform(e.to_string()))?
        .create_lnk(shortcut)
        .map_err(|e| AppError::Platform(e.to_string()))?;
    Ok(())
}
