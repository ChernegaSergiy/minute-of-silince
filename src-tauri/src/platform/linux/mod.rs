pub mod autostart;
pub mod media;
pub mod theme;
pub mod volume;

use zbus::proxy;

#[proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
pub trait PortalSettings {
    fn read(&self, namespace: &str, key: &str) -> zbus::Result<zbus::zvariant::OwnedValue>;

    #[zbus(signal)]
    fn setting_changed(
        &self,
        namespace: &str,
        key: &str,
        value: zbus::zvariant::Value<'_>,
    ) -> zbus::Result<()>;
}

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
