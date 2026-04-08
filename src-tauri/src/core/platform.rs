use crate::error::Result;

#[tauri::async_trait]
pub trait Platform: Send + Sync {
    fn get_volume(&self) -> Result<u8>;
    fn set_volume(&self, level: u8) -> Result<()>;
    fn is_muted(&self) -> Result<bool>;
    fn set_mute(&self, mute: bool) -> Result<()>;
    async fn pause_media(&self) -> Result<()>;
}

#[cfg(target_os = "windows")]
pub struct WindowsPlatform;

#[cfg(target_os = "windows")]
#[tauri::async_trait]
impl Platform for WindowsPlatform {
    fn get_volume(&self) -> Result<u8> {
        crate::platform_windows::volume::get_volume()
    }
    fn set_volume(&self, level: u8) -> Result<()> {
        crate::platform_windows::volume::set_volume(level)
    }
    fn is_muted(&self) -> Result<bool> {
        crate::platform_windows::volume::is_muted()
    }
    fn set_mute(&self, mute: bool) -> Result<()> {
        crate::platform_windows::volume::set_mute(mute)
    }
    async fn pause_media(&self) -> Result<()> {
        crate::platform_windows::media::pause_all().await
    }
}

#[cfg(target_os = "linux")]
pub struct LinuxPlatform;

#[cfg(target_os = "linux")]
#[tauri::async_trait]
impl Platform for LinuxPlatform {
    fn get_volume(&self) -> Result<u8> {
        crate::platform_linux::volume::get_volume()
    }
    fn set_volume(&self, level: u8) -> Result<()> {
        crate::platform_linux::volume::set_volume(level)
    }
    fn is_muted(&self) -> Result<bool> {
        crate::platform_linux::volume::is_muted()
    }
    fn set_mute(&self, mute: bool) -> Result<()> {
        crate::platform_linux::volume::set_mute(mute)
    }
    async fn pause_media(&self) -> Result<()> {
        crate::platform_linux::media::pause_all().await
    }
}

pub fn get_platform() -> Box<dyn Platform> {
    #[cfg(target_os = "windows")]
    return Box::new(WindowsPlatform);
    #[cfg(target_os = "linux")]
    return Box::new(LinuxPlatform);
}
