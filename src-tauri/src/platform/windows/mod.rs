pub mod autostart;
pub mod media;
pub mod notifications;
pub mod power;
pub mod theme;
pub mod volume;

pub struct WindowsPlatform;

#[async_trait::async_trait]
impl super::Platform for WindowsPlatform {
    fn get_volume(&self) -> crate::error::Result<u8> {
        self::volume::get_volume()
    }
    fn set_volume(&self, level: u8) -> crate::error::Result<()> {
        self::volume::set_volume(level)
    }
    fn is_muted(&self) -> crate::error::Result<bool> {
        self::volume::is_muted()
    }
    fn set_mute(&self, mute: bool) -> crate::error::Result<()> {
        self::volume::set_mute(mute)
    }
    async fn pause_media(&self) -> crate::error::Result<Vec<String>> {
        self::media::pause_all().await
    }
    async fn resume_media(&self, players: Vec<String>) -> crate::error::Result<()> {
        self::media::resume_specific(players).await
    }
}

/// Returns true when the current process is running from an MSIX package
/// (i.e. installed via Microsoft Store or `.msix`/`.msixbundle`).
pub fn is_msix() -> bool {
    use windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
    use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;
    use windows::core::PWSTR;

    let mut length = 0;
    let result = unsafe { GetCurrentPackageFullName(&mut length, PWSTR::null()) };

    // The magic number 15700 is wrapped in the APPMODEL_ERROR_NO_PACKAGE constant in windows-rs.
    // But in practice, we check specifically for ERROR_INSUFFICIENT_BUFFER.
    // If it returned the "insufficient buffer" error (code 122),
    // it means the package definitely exists, we just didn't provide a buffer to write the name.
    match result {
        Err(e) => e.code() == ERROR_INSUFFICIENT_BUFFER.to_hresult(),
        Ok(_) => true,
    }
}
