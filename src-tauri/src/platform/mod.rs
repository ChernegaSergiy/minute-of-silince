#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

use crate::error::Result;

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
    async fn pause_media(&self) -> Result<()>;
}

pub fn get_platform() -> Box<dyn Platform> {
    #[cfg(target_os = "windows")]
    return Box::new(windows::WindowsPlatform);
    #[cfg(target_os = "linux")]
    return Box::new(linux::LinuxPlatform);
}
