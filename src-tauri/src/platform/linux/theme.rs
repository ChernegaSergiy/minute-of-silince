//! Linux theme watcher using D-Bus (GNOME).

use std::thread;
use tauri::AppHandle;
use zbus::blocking::Connection;

pub fn start_theme_watcher(app_handle: AppHandle) {
    thread::spawn(move || {
        let conn = match Connection::session() {
            Ok(c) => c,
            Err(e) => {
                log::warn!("Failed to connect to D-Bus: {}", e);
                return;
            }
        };

        let proxy = match zbus::blocking::fdo::PropertiesProxy::builder(&conn)
            .destination("org.gnome.desktop.interface")
            .and_then(|b| b.path("/org/gnome/desktop/interface"))
        {
            Ok(p) => p,
            Err(e) => {
                log::warn!("Failed to create Properties proxy: {}", e);
                return;
            }
        };

        loop {
            match proxy.receive_properties_changed() {
                Ok(changed) => {
                    for (name, _) in changed.iter() {
                        if name == "color-scheme" {
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
                Err(e) => {
                    log::warn!("Failed to receive properties changed: {}", e);
                }
            }
        }
    });
}
