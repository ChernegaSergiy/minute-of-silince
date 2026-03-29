//! Shared application state.

use std::sync::{Arc, Mutex};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::core::ntp_service::NtpService;
use crate::core::settings::Settings;
use crate::core::audio::AudioEngine;

/// Runtime state shared between the scheduler, commands, and tray.
#[derive(Debug)]
pub struct AppState {
    inner: Arc<Mutex<Inner>>,
    pub ntp_service: NtpService,
    pub audio: Arc<AudioEngine>,
}

#[derive(Debug)]
pub struct Inner {
    pub settings: Settings,
    pub skip_date: Option<chrono::NaiveDate>,
    pub ceremony_active: bool,
    pub last_ntp_sync: Option<DateTime<Local>>,
    pub last_activation: Option<DateTime<Local>>,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            settings: Settings::load_or_default(),
            skip_date: None,
            ceremony_active: false,
            last_ntp_sync: None,
            last_activation: None,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        let settings = Settings::load_or_default();
        Self {
            inner: Arc::new(Mutex::new(Inner::default())),
            ntp_service: NtpService::new(settings.ntp_server.clone()),
            audio: Arc::new(AudioEngine::new()),
        }
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().expect("AppState mutex poisoned")
    }
}

// ── Serialisable snapshot for the frontend ──────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatusSnapshot {
    pub ceremony_active: bool,
    pub skip_tomorrow: bool,
    pub last_activation: Option<String>,
    pub last_ntp_sync: Option<String>,
}

impl From<&Inner> for StatusSnapshot {
    fn from(inner: &Inner) -> Self {
        let tomorrow = (Local::now() + chrono::Duration::days(1)).date_naive();
        Self {
            ceremony_active: inner.ceremony_active,
            skip_tomorrow: inner.skip_date == Some(tomorrow),
            last_activation: inner
                .last_activation
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            last_ntp_sync: inner
                .last_ntp_sync
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}
