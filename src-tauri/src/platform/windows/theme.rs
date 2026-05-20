//! Windows-specific theme watcher.
//! Listens for system theme changes and updates the tray icon.

use std::thread;
use tauri::AppHandle;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_NOTIFY, REG_NOTIFY_CHANGE_LAST_SET, RegCloseKey,
    RegNotifyChangeKeyValue, RegOpenKeyExW,
};
use windows::core::PCWSTR;

pub fn start_theme_watcher(app_handle: AppHandle) {
    let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
        .encode_utf16()
        .collect();

    thread::spawn(move || unsafe {
        let subkey_pcwstr = PCWSTR(subkey.as_ptr());
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey_pcwstr,
            Some(0),
            KEY_NOTIFY,
            &mut hkey,
        )
        .is_ok()
        {
            loop {
                let result = RegNotifyChangeKeyValue(
                    hkey,
                    false,
                    REG_NOTIFY_CHANGE_LAST_SET,
                    None::<HANDLE>,
                    false,
                );

                if result.is_ok() {
                    if let Some(tray) = app_handle.tray_by_id("main") {
                        let is_dark = crate::platform::detect_system_theme();
                        let icon = if is_dark {
                            tauri::include_image!("icons/tray-icon-32-light.png")
                        } else {
                            tauri::include_image!("icons/tray-icon-32-dark.png")
                        };
                        let _ = tray.set_icon(Some(icon));
                        log::info!("Theme changed, tray icon updated");
                    }
                }
            }
        }
        let _ = RegCloseKey(hkey);
    });
}
