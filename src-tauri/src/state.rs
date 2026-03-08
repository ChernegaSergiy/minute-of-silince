use std::sync::{Arc, Mutex};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::core::settings::Settings;

/// Runtime state shared between the scheduler, commands, and tray.
#[derive(Debug, Default)]
pub struct AppState(Arc<Mutex<Inner>>);

#[derive(Debug)]
pub struct Inner {
    pub settings: Settings,
    /// When set, the scheduler will skip the activation on this calendar day.
    pub skip_date: Option<chrono::NaiveDate>,
    /// Whether the ceremony is currently in progress.
    pub ceremony_active: bool,
    /// Timestamp of the last successful NTP sync.
    pub last_ntp_sync: Option<DateTime<Local>>,
    /// Timestamp of the last ceremony activation.
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
        Self(Arc::new(Mutex::new(Inner::default())))
    }

    /// Acquire the inner lock.
    ///
    /// Panics only if the mutex is poisoned (i.e. a previous thread panicked
    /// while holding it), which is treated as an unrecoverable error.
    pub fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.0.lock().expect("AppState mutex poisoned")
    }

    pub fn clone_arc(&self) -> Arc<Mutex<Inner>> {
        Arc::clone(&self.0)
    }
}

// ── Serialisable snapshot for the frontend ──────────────────────────────────

/// A read-only snapshot of the runtime status sent to the UI.
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
