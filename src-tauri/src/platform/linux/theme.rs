//! Linux theme watcher using XDG Desktop Portal (universal for GNOME, KDE, etc.).

use crate::platform::linux::PortalSettingsProxyBlocking;
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

        let proxy = match PortalSettingsProxyBlocking::new(&conn) {
            Ok(p) => p,
            Err(e) => {
                log::warn!("Failed to create Portal Settings proxy: {}", e);
                return;
            }
        };

        let iter = match proxy.receive_setting_changed() {
            Ok(it) => it,
            Err(e) => {
                log::warn!("Failed to watch for Portal settings changes: {}", e);
                return;
            }
        };

        for signal in iter {
            if let Ok(args) = signal.args() {
                if *args.namespace() == "org.freedesktop.appearance"
                    && *args.key() == "color-scheme"
                {
                    // is_dark_mode() already handles the GNOME exception (forcing dark mode).
                    let is_dark = crate::platform::is_dark_mode();

                    if crate::platform::is_gnome() {
                        // Skip redundant updates on GNOME as the panel is always dark.
                        continue;
                    }

                    if let Some(tray) = app_handle.tray_by_id("main") {
                        let icon = if is_dark {
                            tauri::include_image!("icons/tray-icon-32-light.png")
                        } else {
                            tauri::include_image!("icons/tray-icon-32-dark.png")
                        };
                        let _ = tray.set_icon(Some(icon));
                        log::info!("System theme changed via Portal, tray icon updated");
                    }
                }
            }
        }
    });
}
