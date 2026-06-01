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
                    let is_dark = is_dark_mode();

                    if is_gnome() {
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

pub fn detect_system_theme() -> bool {
    use std::process::Command;

    // 1. Primary method: XDG Desktop Portal (Universal)
    if let Ok(conn) = Connection::session() {
        if let Ok(proxy) = PortalSettingsProxyBlocking::new(&conn) {
            if let Ok(val) = proxy.read("org.freedesktop.appearance", "color-scheme") {
                if let Ok(scheme) = u32::try_from(val) {
                    return scheme == 1; // 1 = Prefer Dark
                }
            }
        }
    }

    // 2. Fallback: KDE specific via kreadconfig
    if is_kde() {
        for cmd in ["kreadconfig6", "kreadconfig5"] {
            let output = Command::new(cmd)
                .args([
                    "--file",
                    "kdeglobals",
                    "--group",
                    "KDE",
                    "--key",
                    "ColorScheme",
                ])
                .output();

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
                if !stdout.trim().is_empty() {
                    return stdout.contains("dark");
                }
            }
        }
    }

    // 3. Fallback: GNOME/GSettings
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.contains("dark");
    }

    false
}

pub fn is_gnome() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|v| {
            let v = v.to_lowercase();
            v.contains("gnome") || v.contains("unity")
        })
        .unwrap_or(false)
}

pub fn is_kde() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|v| v.to_lowercase().contains("kde"))
        .unwrap_or(false)
}

pub fn is_dark_mode() -> bool {
    if is_gnome() {
        // On GNOME, the top panel is almost always dark regardless of the application theme.
        return true;
    }
    if is_kde() {
        // On KDE, we rely on the specific theme detection in detect_system_theme.
        return detect_system_theme();
    }
    detect_system_theme()
}
