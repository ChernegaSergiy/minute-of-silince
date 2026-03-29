use crate::error::Result;

/// Interface for platform-specific operations.
/// This is our "Abstract Class" for system integration.
pub trait Platform: Send + Sync {
    fn get_volume(&self) -> Result<u8>;
    fn set_volume(&self, level: u8) -> Result<()>;
    fn pause_media(&self) -> Result<()>;
    fn resume_media(&self) -> Result<()>;
}

#[cfg(target_os = "windows")]
pub struct WindowsPlatform;

#[cfg(target_os = "windows")]
impl Platform for WindowsPlatform {
    fn get_volume(&self) -> Result<u8> {
        crate::platform_windows::volume::get_volume()
    }
    fn set_volume(&self, level: u8) -> Result<()> {
        crate::platform_windows::volume::set_volume(level)
    }
    fn pause_media(&self) -> Result<()> {
        crate::platform_windows::media::pause_all()
    }
    fn resume_media(&self) -> Result<()> {
        crate::platform_windows::media::resume_all()
    }
}

#[cfg(target_os = "linux")]
pub struct LinuxPlatform;

#[cfg(target_os = "linux")]
impl Platform for LinuxPlatform {
    fn get_volume(&self) -> Result<u8> {
        crate::platform_linux::volume::get_volume()
    }
    fn set_volume(&self, level: u8) -> Result<()> {
        crate::platform_linux::volume::set_volume(level)
    }
    fn pause_media(&self) -> Result<()> {
        crate::platform_linux::media::pause_all()
    }
    fn resume_media(&self) -> Result<()> {
        crate::platform_linux::media::resume_all()
    }
}

pub fn get_platform() -> Box<dyn Platform> {
    #[cfg(target_os = "windows")]
    return Box::new(WindowsPlatform);
    #[cfg(target_os = "linux")]
    return Box::new(LinuxPlatform);
}
