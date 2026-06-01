pub mod media;
pub mod volume;

pub struct MacosPlatform;

#[async_trait::async_trait]
impl super::Platform for MacosPlatform {
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

pub mod theme {
    pub fn detect_system_theme() -> bool {
        use std::process::Command;
        let output = Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.trim().to_lowercase().contains("dark")
        } else {
            false
        }
    }
    pub fn is_dark_mode() -> bool {
        detect_system_theme()
    }
}

pub mod autostart {
    pub fn system_autostart_enabled() -> Option<bool> {
        None
    }
    pub fn apply_autostart_enabled(app: &tauri::AppHandle, enabled: bool) {
        use tauri_plugin_autostart::ManagerExt;
        let autostart_manager = app.autolaunch();
        if enabled {
            let _ = autostart_manager.enable();
        } else {
            let _ = autostart_manager.disable();
        }
    }
}
