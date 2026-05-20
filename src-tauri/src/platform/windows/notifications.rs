//! Windows MSIX toast notifications via WinRT API.
//!
//! This module bypasses tauri-plugin-notification for MSIX packages,
//! which doesn't work due to AppContainer restrictions.

/// Send a toast notification using WinRT API (for MSIX packages).
///
/// This function is only functional when running as MSIX package.
/// For non-MSIX builds, this does nothing.
#[cfg(target_os = "windows")]
pub fn send_toast(title: &str, body: &str) -> Result<(), crate::AppError> {
    if !crate::platform::is_msix() {
        return Ok(());
    }

    // Build AUMID from the current package family name at runtime.
    let package = windows::ApplicationModel::Package::Current()
        .map_err(|e| crate::AppError::Windows(e.to_string()))?;
    let id = package
        .Id()
        .map_err(|e| crate::AppError::Windows(e.to_string()))?;
    let family = id
        .FamilyName()
        .map_err(|e| crate::AppError::Windows(e.to_string()))?;
    let aumid = format!("{}!{}", family, "ua.pp.khvylyna.MinuteOfSilence");

    tauri_winrt_notification::Toast::new(&aumid)
        .scenario(tauri_winrt_notification::Scenario::Reminder)
        .title(title)
        .text1(body)
        .show()
        .map_err(|e| crate::AppError::Windows(e.to_string()))?;

    Ok(())
}

/// Dummy implementation for non-Windows platforms.
#[cfg(not(target_os = "windows"))]
pub fn send_toast(_title: &str, _body: &str) -> Result<(), crate::AppError> {
    Ok(())
}
