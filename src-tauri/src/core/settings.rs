//! Persistent application settings and audio presets.
//!
//! Handles loading, saving, and providing default values for user
//! configurations such as volume, selected preset, and scheduling options.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// User-configurable settings.  Persisted as JSON in the platform config dir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Enable daily activation at 09:00.
    pub ceremony_enabled: bool,

    /// Enable app autostart when the system boots.
    pub autostart_enabled: bool,

    /// Run ceremony only on weekdays (Mon-Fri).
    pub weekdays_only: bool,

    /// Selected audio preset (1–8).
    pub preset: AudioPreset,

    /// Master volume for the ceremony audio (0–100).
    pub volume: u8,

    /// Pause other media players before the ceremony.
    pub pause_other_players: bool,

    /// Automatically resume paused players after the ceremony.
    pub resume_after_ceremony: bool,

    /// Show a visual overlay window when the ceremony starts.
    pub show_visual_overlay: bool,

    /// Show the flag animation window when the ceremony starts.
    pub show_flag_animation: bool,

    /// Use system time instead of NTP.
    pub system_time_only: bool,

    /// Prioritize app volume over system controls.
    pub volume_priority: bool,

    /// Automatically unmute system if muted during ceremony.
    pub auto_unmute: bool,

    /// NTP server hostname (used when system_time_only is false).
    pub ntp_server: String,

    /// If the system wakes from sleep after 09:00, activate if within this
    /// many minutes of the scheduled time (0 = never activate late).
    pub late_start_grace_minutes: u8,

    /// Enable reminder notifications.
    pub reminder_enabled: bool,

    /// Show a system notification N minutes before the ceremony (0 = immediately).
    /// Valid range: 0–10.
    pub reminder_minutes_before: u8,

    /// Selected announcement voice/version.
    pub announcement_voice: AnnouncementVoice,

    /// Selected anthem voice/performance.
    pub anthem_voice: AnthemVoice,
    /// Follow the OS theme when true; otherwise use `ui_theme`.
    pub use_system_theme: bool,

    /// Manual UI theme when `use_system_theme` is false.
    pub ui_theme: UiTheme,

    /// Date to skip the next ceremony (one-time skip). Persisted to disk.
    #[serde(default)]
    pub skip_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiTheme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnouncementVoice {
    BohdanHdal,
    SoniaSotnyk,
    DaniaKhomutovskyi,
    RadioBg,
    AirAlert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnthemVoice {
    Default,
    MykhailoKhoma,
    OleksandrPonomarov,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ceremony_enabled: true,
            autostart_enabled: true,
            weekdays_only: false,
            preset: AudioPreset::VoiceMetronome,
            volume: 80,
            pause_other_players: true,
            resume_after_ceremony: false,
            show_visual_overlay: true,
            show_flag_animation: false,
            system_time_only: false,
            volume_priority: false,
            auto_unmute: false,
            ntp_server: "pool.ntp.org".to_string(),
            late_start_grace_minutes: 1,
            reminder_enabled: false,
            reminder_minutes_before: 5,
            announcement_voice: AnnouncementVoice::BohdanHdal,
            anthem_voice: AnthemVoice::Default,
            use_system_theme: true,
            ui_theme: UiTheme::Light,
            skip_date: None,
        }
    }
}

impl Settings {
    /// Load settings from disk, falling back to defaults on any error.
    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_else(|e| {
            log::warn!("Failed to load settings ({e}), using defaults");
            Self::default()
        })
    }

    /// Load settings from the Tauri store, falling back to defaults on any error.
    pub fn load_from_store(app_handle: &tauri::AppHandle) -> Self {
        use tauri_plugin_store::StoreExt;

        let load_impl = || -> std::result::Result<Settings, String> {
            let store = app_handle
                .store("settings.json")
                .map_err(|e| e.to_string())?;
            if let Some(val) = store.get("settings") {
                let settings: Settings = serde_json::from_value(val).map_err(|e| e.to_string())?;
                Ok(settings)
            } else {
                Ok(Settings::default())
            }
        };

        load_impl().unwrap_or_else(|e| {
            log::warn!("Failed to load settings from store: {}, using defaults", e);
            Settings::default()
        })
    }

    /// Save settings to the Tauri store.
    pub fn save_to_store(&self, app_handle: &tauri::AppHandle) -> Result<()> {
        use tauri_plugin_store::StoreExt;

        let store = app_handle
            .store("settings.json")
            .map_err(|e| crate::error::AppError::Settings(e.to_string()))?;

        let val = serde_json::to_value(self)?;
        store.set("settings".to_string(), val);
        store
            .save()
            .map_err(|e| crate::error::AppError::Settings(e.to_string()))?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&path)?;
        let settings: Settings = serde_json::from_str(&raw)?;
        Ok(settings)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    fn path() -> Result<PathBuf> {
        let mut dir = dirs::config_dir()
            .ok_or_else(|| AppError::Settings("Cannot locate config directory".into()))?;
        dir.push("minute-of-silence");
        dir.push("settings.json");
        Ok(dir)
    }
}

/// Available audio presets for the ceremony.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioPreset {
    /// Voice announcement + metronome
    VoiceMetronome,
    /// Metronome only (no voice)
    MetronomeOnly,
    /// Voice announcement + 60 s silence + bell
    VoiceSilenceBell,
    /// Voice announcement + 60 s silence
    VoiceSilence,
    /// Voice announcement + metronome + national anthem
    VoiceMetronomeAnthem,
    /// Voice announcement + metronome + ending
    VoiceMetronomeEnding,
    /// Metronome + national anthem
    MetronomeAnthem,
    /// Bell + 60 s silence + bell
    BellSilenceBell,
    /// Bell + metronome + bell
    BellMetronomeBell,
    /// Silence only (no audio, just visual overlay)
    Silence,
}

impl AudioPreset {
    pub fn has_anthem(self) -> bool {
        matches!(self, Self::VoiceMetronomeAnthem | Self::MetronomeAnthem)
    }
}
