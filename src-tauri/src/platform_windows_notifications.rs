//! Windows MSIX toast notifications via WinRT API.
//!
//! This module bypasses tauri-plugin-notification for MSIX packages,
//! which doesn't work due to AppContainer restrictions.

#[cfg(target_os = "windows")]
use windows::{
    core::HSTRING,
    Data::Xml::Dom::XmlDocument,
    UI::Notifications::{ToastNotification, ToastNotificationManager},
};

/// Send a toast notification using WinRT API (for MSIX packages).
///
/// This function is only functional when running as MSIX package.
/// For non-MSIX builds, this does nothing.
#[cfg(target_os = "windows")]
pub fn send_toast(title: &str, body: &str) -> Result<(), crate::AppError> {
    if !crate::platform_scheduler_task::is_msix_package() {
        return Ok(());
    }

    // Get correct AUMID from installed package
    let aumid = HSTRING::from("BF570F0A.313786C1665D6_t7fhw3d0jsaty!minuteOfSilence");

    let xml_str = format!(
        r#"<toast><visual><binding template="ToastGeneric">
            <text>{}</text>
            <text>{}</text>
        </binding></visual></toast>"#,
        title, body
    );

    let xml = XmlDocument::new().map_err(|e| crate::AppError::Windows(e.to_string()))?;
    xml.LoadXml(&HSTRING::from(xml_str.as_str()))
        .map_err(|e| crate::AppError::Windows(e.to_string()))?;

    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&aumid)
        .map_err(|e| crate::AppError::Windows(e.to_string()))?;
    let toast = ToastNotification::CreateToastNotification(&xml)
        .map_err(|e| crate::AppError::Windows(e.to_string()))?;
    notifier
        .Show(&toast)
        .map_err(|e| crate::AppError::Windows(e.to_string()))?;

    Ok(())
}

/// Dummy implementation for non-Windows platforms.
#[cfg(not(target_os = "windows"))]
pub fn send_toast(_title: &str, _body: &str) -> Result<(), crate::AppError> {
    Ok(())
}
