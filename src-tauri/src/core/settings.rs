use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// User-configurable settings.  Persisted as JSON in the platform config dir.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Enable daily activation at 09:00.
    pub autostart_enabled: bool,

    /// Run ceremony only on weekdays (Mon-Fri).
    pub weekdays_only: bool,

    /// Selected audio preset (1–5).
    pub preset: AudioPreset,

    /// Master volume for the ceremony audio (0–100).
    pub volume: u8,

    /// Pause other media players before the ceremony.
    pub pause_other_players: bool,

    /// Use NTP to correct the trigger time.
    pub ntp_sync_enabled: bool,

    /// NTP server hostname.
    pub ntp_server: String,

    /// If the system wakes from sleep after 09:00, activate if within this
    /// many minutes of the scheduled time (0 = never activate late).
    pub late_start_grace_minutes: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            autostart_enabled: true,
            weekdays_only: false,
            preset: AudioPreset::VoiceMetronome,
            volume: 80,
            pause_other_players: true,
            ntp_sync_enabled: true,
            ntp_server: "pool.ntp.org".to_string(),
            late_start_grace_minutes: 5,
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

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&path)?;
        let settings = serde_json::from_str(&raw)?;
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
    /// Metronome + national anthem
    MetronomeAnthem,
    /// Bell + 60 s silence + bell
    BellSilenceBell,
    /// Bell + metronome + bell
    BellMetronomeBell,
}

impl std::fmt::Display for AudioPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::VoiceMetronome => "Голос + метроном",
            Self::MetronomeOnly => "Метроном",
            Self::VoiceSilenceBell => "Голос + тиша + дзвін",
            Self::VoiceSilence => "Голос + тиша",
            Self::VoiceMetronomeAnthem => "Голос + метроном + гімн",
            Self::MetronomeAnthem => "Метроном + гімн",
            Self::BellSilenceBell => "Дзвін + тиша + дзвін",
            Self::BellMetronomeBell => "Дзвін + метроном + дзвін",
        };
        write!(f, "{label}")
    }
}
