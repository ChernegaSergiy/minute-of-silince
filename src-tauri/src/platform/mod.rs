#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

use crate::error::Result;

#[cfg(target_os = "windows")]
pub fn detect_system_theme() -> bool {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) =
        hkcu.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")
    {
        if let Ok(value) = key.get_value::<u32, _>("AppsUseLightTheme") {
            return value == 0;
        }
    }
    false
}

#[cfg(target_os = "linux")]
pub fn detect_system_theme() -> bool {
    use crate::platform::linux::PortalSettingsProxyBlocking;
    use std::process::Command;
    use zbus::blocking::Connection;

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

#[cfg(target_os = "linux")]
pub fn is_gnome() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|v| {
            let v = v.to_lowercase();
            v.contains("gnome") || v.contains("unity")
        })
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
pub fn is_kde() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|v| v.to_lowercase().contains("kde"))
        .unwrap_or(false)
}

#[cfg(not(target_os = "linux"))]
pub fn is_gnome() -> bool {
    false
}

#[cfg(not(target_os = "linux"))]
pub fn is_kde() -> bool {
    false
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn detect_system_theme() -> bool {
    false
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

/// Returns the current autostart state reported by the platform.
///
/// This is used to keep the persisted setting aligned with changes made
/// outside the app, such as system UI toggles.
pub fn system_autostart_enabled() -> Option<bool> {
    #[cfg(target_os = "windows")]
    {
        if !is_msix() {
            None
        } else {
            use ::windows::ApplicationModel::{StartupTask, StartupTaskState};
            use ::windows::core::HSTRING;

            let task = StartupTask::GetAsync(&HSTRING::from("MinuteOfSilenceStartupTask"))
                .ok()?
                .join()
                .ok()?;
            let state = task.State().ok()?;
            Some(state == StartupTaskState::Enabled)
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(snap_user_data) = std::env::var("SNAP_USER_DATA") {
            let desktop_path = std::path::PathBuf::from(snap_user_data)
                .join(".config/autostart/minute-of-silence.desktop");
            Some(desktop_path.exists())
        } else if let Ok(flatpak_id) = std::env::var("FLATPAK_ID") {
            let home = std::env::var("HOME").ok()?;
            let desktop_path = std::path::PathBuf::from(home)
                .join(".config/autostart")
                .join(format!("{}.desktop", flatpak_id));
            Some(desktop_path.exists())
        } else {
            None
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        None
    }
}

/// Apply the requested autostart state to the current platform.
pub fn apply_autostart_enabled(app: &tauri::AppHandle, enabled: bool) {
    let _ = (app, enabled);

    let is_snap = std::env::var("SNAP").is_ok();
    let is_flatpak = std::env::var("FLATPAK_ID").is_ok();

    if is_snap || is_flatpak {
        #[cfg(target_os = "linux")]
        crate::platform::linux::autostart::manage(enabled);
    } else {
        #[cfg(not(test))]
        {
            let is_msix = is_msix();

            if is_msix {
                #[cfg(target_os = "windows")]
                {
                    if enabled {
                        if let Err(e) = crate::platform::windows::autostart::enable_autostart() {
                            log::error!("Failed to enable autostart for MSIX: {}", e);
                        }
                    } else {
                        if let Err(e) = crate::platform::windows::autostart::disable_autostart() {
                            log::error!("Failed to disable autostart for MSIX: {}", e);
                        }
                    }
                }
            } else {
                use tauri_plugin_autostart::ManagerExt;
                let autostart_manager = app.autolaunch();
                if enabled {
                    let _ = autostart_manager.enable();
                } else {
                    let _ = autostart_manager.disable();
                }
            }
        }
    }
}

/// Sync the persisted autostart setting with the actual platform state.
pub fn sync_autostart_from_system(state: tauri::State<'_, crate::AppState>) -> Result<()> {
    let mut guard = state.lock();
    let mut settings = guard.settings.clone();

    if let Some(system_enabled) = system_autostart_enabled() {
        if system_enabled != settings.autostart_enabled {
            settings.autostart_enabled = system_enabled;
            settings.save_to_store(&state.app_handle)?;
            guard.settings = settings;
            log::info!("Autostart setting synced from system: {}", system_enabled);
        }
    }

    Ok(())
}

/// Returns true when the current process is running from an MSIX package
/// (i.e. installed via Microsoft Store or `.msix`/`.msixbundle`).
#[allow(dead_code)]
pub fn is_msix() -> bool {
    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn GetCurrentPackageFullName(
                packageFullNameLength: *mut u32,
                packageFullName: *mut u16,
            ) -> i32;
        }
        let mut length = 0;
        let rc = unsafe { GetCurrentPackageFullName(&mut length, std::ptr::null_mut()) };
        rc != 15700 // 15700 is APPMODEL_ERROR_NO_PACKAGE
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// Returns true if the application should perform automatic update checks.
///
/// Updates are disabled for sandboxed distributions (Snap, Flatpak, MSIX)
/// where updates are managed by their respective stores/package managers.
pub fn should_check_for_updates() -> bool {
    let is_snap = std::env::var("SNAP").is_ok();
    let is_flatpak = std::env::var("FLATPAK_ID").is_ok();
    let is_msix = is_msix();

    !is_snap && !is_flatpak && !is_msix
}

#[async_trait::async_trait]
pub trait Platform: Send + Sync {
    fn get_volume(&self) -> Result<u8>;
    fn set_volume(&self, level: u8) -> Result<()>;
    fn is_muted(&self) -> Result<bool>;
    fn set_mute(&self, mute: bool) -> Result<()>;
    async fn pause_media(&self) -> Result<Vec<String>>;
    async fn resume_media(&self, players: Vec<String>) -> Result<()>;
}

pub fn get_platform() -> Box<dyn Platform> {
    #[cfg(target_os = "windows")]
    return Box::new(windows::WindowsPlatform);
    #[cfg(target_os = "linux")]
    return Box::new(linux::LinuxPlatform);
    #[cfg(target_os = "macos")]
    return Box::new(macos::MacosPlatform);
}
