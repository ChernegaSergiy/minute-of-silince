//! Linux theme watcher using XDG Desktop Portal (universal for GNOME, KDE, etc.).

use std::thread;
use tauri::AppHandle;
use zbus::blocking::Connection;
use zbus::proxy;

#[proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait PortalSettings {
    #[zbus(signal)]
    fn setting_changed(
        &self,
        namespace: &str,
        key: &str,
        value: zbus::zvariant::Value<'_>,
    ) -> zbus::Result<()>;
}

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
                if args.namespace() == "org.freedesktop.appearance" && args.key() == "color-scheme"
                {
                    // Skip updates on GNOME because the panel is always dark and we use a light icon.
                    if crate::platform::is_gnome() {
                        continue;
                    }

                    if let Some(tray) = app_handle.tray_by_id("main") {
                        let is_dark = crate::platform::detect_system_theme();
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
