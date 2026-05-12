//! Windows autostart management for MSIX packages.
//!
//! MSIX applications cannot directly modify the standard Run registry key,
//! so we write to the virtualized registry which is properly handled by the OS.

use crate::error::Result;
use winreg::enums::*;
use winreg::RegKey;

/// Enable autostart by writing to the virtualized registry.
pub fn enable_autostart() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let (key, _) = hkcu.create_subkey(path)?;

    // Get the current executable path
    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy().to_string();

    key.set_value("MinuteOfSilence", &exe_path_str)?;
    log::info!("Autostart enabled for MSIX: {}", exe_path_str);

    Ok(())
}

/// Disable autostart by removing the registry entry.
pub fn disable_autostart() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";

    if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_WRITE) {
        let _ = key.delete_value("MinuteOfSilence");
        log::info!("Autostart disabled for MSIX");
    }

    Ok(())
}
