#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

use crate::error::Result;

#[cfg(target_os = "windows")]
pub fn detect_system_theme() -> bool {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

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
            use ::windows::core::HSTRING;
            use ::windows::ApplicationModel::{StartupTask, StartupTaskState};

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

/// Returns true when the current process is running from an MSIX package
/// (i.e. installed via Microsoft Store or `.msix`/`.msixbundle`).
#[allow(dead_code)]
pub fn is_msix() -> bool {
    #[cfg(target_os = "windows")]
    {
        std::env::current_exe()
            .map(|p| {
                let s = p.to_string_lossy().to_ascii_lowercase();
                s.contains("\\windowsapps\\")
            })
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
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
}
