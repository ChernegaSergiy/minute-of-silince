//! Ceremony scheduler and execution logic.

use chrono::{Local, NaiveTime, Timelike};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use crate::core::audio::AudioEngine;
use crate::core::CeremonyManager;
use crate::state::AppState;

/// Scheduler for the daily ceremony.
pub struct CeremonyScheduler {
    app: AppHandle,
    audio: Arc<AudioEngine>,
}

impl CeremonyScheduler {
    pub fn new(app: AppHandle) -> Self {
        let audio = app.state::<AppState>().audio.clone();
        Self { app, audio }
    }

    /// Run the main scheduler loop.
    pub async fn run(&self) {
        log::info!("Scheduler loop started");

        // Initial NTP sync
        self.sync_ntp().await;

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

        loop {
            interval.tick().await;

            // Periodic NTP sync (every hour)
            let now_local = Local::now();
            if now_local.minute() == 0 && now_local.second() == 0 {
                self.sync_ntp().await;
            }

            let should_trigger = {
                let state = self.app.state::<AppState>();
                let inner = state.lock();

                if !inner.ceremony_active {
                    let now = self.get_synchronized_now();
                    let ceremony_time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();

                    let grace_minutes = inner.settings.late_start_grace_minutes;
                    if self.is_within_window(now.time(), ceremony_time, grace_minutes) {
                        let today = now.date_naive();
                        let last_activated = inner.last_activation.map(|dt| dt.date_naive());

                        last_activated != Some(today) && inner.skip_date != Some(today)
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            if should_trigger {
                self.trigger_ceremony().await;
            }
        }
    }

    async fn sync_ntp(&self) {
        let state = self.app.state::<AppState>();

        if state.lock().settings.system_time_only {
            state.ntp_service.clear_cache();
            return;
        }

        log::info!("Attempting NTP synchronization...");
        let _ = state.ntp_service.sync().await;
        // Notify frontend (it will call get_status and use NtpService's state)
        let _ = self.app.emit("ntp-synced", ());
    }

    fn is_within_window(&self, now: NaiveTime, target: NaiveTime, grace_minutes: u8) -> bool {
        if now < target {
            return false;
        }
        let elapsed_secs = (now - target).num_seconds();
        elapsed_secs <= (grace_minutes as i64) * 60
    }

    fn get_synchronized_now(&self) -> chrono::DateTime<Local> {
        let state = self.app.state::<AppState>();
        let ntp_offset = state.ntp_service.get_offset();

        if let Some(offset_ms) = ntp_offset {
            let now = Local::now();
            let corrected = now + chrono::Duration::milliseconds(offset_ms);
            return corrected;
        }
        Local::now()
    }

    pub async fn trigger_ceremony(&self) {
        let platform = crate::core::platform::get_platform();
        let manager = CeremonyManager::new(self.app.clone(), platform, Arc::clone(&self.audio));
        manager.run_ceremony().await;
    }
}

pub async fn run(app: AppHandle) {
    let scheduler = CeremonyScheduler::new(app);
    scheduler.run().await;
}

pub async fn trigger_now(app: AppHandle) {
    let scheduler = CeremonyScheduler::new(app);
    scheduler.trigger_ceremony().await;
}
