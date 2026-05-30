//! Shared application state.

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

use crate::app::next_skip_date;
use crate::core::audio::AudioEngine;
use crate::core::ntp_service::NtpService;
use crate::core::settings::Settings;

rust_i18n::i18n!("locales");

use rust_i18n::t;

/// Runtime state shared between the scheduler, commands, and tray.
#[derive(Debug)]
pub struct AppState {
    inner: Arc<Mutex<Inner>>,
    pub ntp_service: NtpService,
    pub audio: Arc<AudioEngine>,
    pub app_handle: AppHandle,
}

#[derive(Debug, Default)]
pub struct Inner {
    pub settings: Settings,
    pub ceremony_active: bool,
    pub last_activation: Option<DateTime<Local>>,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        let settings = Settings::load_from_store(&app_handle);
        Self::new_with_settings(app_handle, settings)
    }

    pub fn new_with_settings(app_handle: AppHandle, settings: Settings) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                settings: settings.clone(),
                ceremony_active: false,
                last_activation: None,
            })),
            ntp_service: NtpService::new(settings.ntp_server.clone()),
            audio: Arc::new(AudioEngine::new(app_handle.clone())),
            app_handle,
        }
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().expect("AppState mutex poisoned")
    }
}

// Serialisable snapshot for the frontend

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
        let skip_target = next_skip_date(Local::now());

        let ntp_status = if inner.settings.system_time_only {
            Some(t!("ntp_disabled").to_string())
        } else {
            self.ntp_service
                .last_sync_time()
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .or_else(|| Some(t!("ntp_syncing").to_string()))
        };

        StatusSnapshot {
            ceremony_active: inner.ceremony_active,
            skip_tomorrow: inner.settings.skip_date == Some(skip_target),
            last_activation: inner
                .last_activation
                .map(|dt| dt.format("%H:%M:%S").to_string()),
            last_ntp_sync: ntp_status,
        }
    }
}
