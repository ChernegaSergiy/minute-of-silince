//! Shared application state.

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

use crate::core::audio::AudioEngine;
use crate::core::ntp_service::NtpService;
use crate::core::settings::Settings;

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
    pub last_activation: Option<DateTime<Local>>,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            settings: Settings::load_or_default(),
            skip_date: None,
            ceremony_active: false,
            last_activation: None,
        }
    }
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        let settings = Settings::load_or_default();
        Self {
            inner: Arc::new(Mutex::new(Inner::default())),
            ntp_service: NtpService::new(settings.ntp_server.clone()),
            audio: Arc::new(AudioEngine::new(app_handle)),
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

impl AppState {
    pub fn get_snapshot(&self) -> StatusSnapshot {
        let inner = self.lock();
        let tomorrow = (Local::now() + chrono::Duration::days(1)).date_naive();

        let ntp_status = if inner.settings.system_time_only {
            Some("Вимкнено (системний час)".to_string())
        } else {
            self.ntp_service
                .last_sync_time()
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .or_else(|| Some("Синхронізація...".to_string()))
        };

        StatusSnapshot {
            ceremony_active: inner.ceremony_active,
            skip_tomorrow: inner.skip_date == Some(tomorrow),
            last_activation: inner
                .last_activation
                .map(|dt| dt.format("%H:%M:%S").to_string()),
            last_ntp_sync: ntp_status,
        }
    }
}
