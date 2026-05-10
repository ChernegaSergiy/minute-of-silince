#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

use crate::error::Result;
use std::sync::OnceLock;

static DARK_MODE: OnceLock<bool> = OnceLock::new();

#[cfg(target_os = "windows")]
fn detect_system_theme() -> bool {
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
fn detect_system_theme() -> bool {
    use std::process::Command;

    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.contains("dark");
    }

    false
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn detect_system_theme() -> bool {
    false
}

pub fn is_dark_mode() -> bool {
    *DARK_MODE.get_or_init(detect_system_theme)
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
