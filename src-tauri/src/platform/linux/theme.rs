//! Linux theme watcher using D-Bus (GNOME).

use std::thread;
use tauri::AppHandle;

pub fn start_theme_watcher(app_handle: AppHandle) {
    thread::spawn(move || {
        let conn = match zbus::blocking::Connection::session() {
            Ok(c) => c,
            Err(e) => {
                log::warn!("Failed to connect to D-Bus: {}", e);
                return;
            }
        };

        let props = match zbus::blocking::fdo::PropertiesProxy::builder(&conn)
            .destination("org.gnome.desktop.interface")
            .path("/org/gnome/desktop/interface")
            .build()
        {
            Ok(p) => p,
            Err(e) => {
                log::warn!("Failed to create Properties proxy: {}", e);
                return;
            }
        };

        loop {
            if let Ok(changed) = props.receive_properties_changed(None) {
                if let Ok(props) = changed.body() {
                    for (name, _) in props {
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
            }
        }
    });
}
