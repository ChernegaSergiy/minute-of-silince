#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
use linux as sys;
#[cfg(target_os = "macos")]
use macos as sys;
#[cfg(target_os = "windows")]
use windows as sys;

use crate::error::Result;

// Re-exports from system-specific modules
pub use sys::autostart::{apply_autostart_enabled, system_autostart_enabled};
pub use sys::theme::is_dark_mode;

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
        self::windows::is_msix()
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
