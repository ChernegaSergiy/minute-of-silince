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

    // Keep the same package AUMID, but use the manifest Application Id.
    tauri_winrt_notification::Toast::new(
        "BF570F0A.313786C1665D6_t7fhw3d0jsaty!ua.pp.khvylyna.MinuteOfSilence",
    )
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
