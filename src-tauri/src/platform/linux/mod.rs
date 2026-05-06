pub mod autostart;
pub mod media;
pub mod volume;

pub struct LinuxPlatform;

#[async_trait::async_trait]
impl super::Platform for LinuxPlatform {
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
