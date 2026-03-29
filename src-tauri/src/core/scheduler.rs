//! Daily scheduler: fires the ceremony at 09:00 local time, optionally
//! corrected by NTP, and handles the post-sleep "late start" grace period.

use chrono::{Datelike, Local, NaiveTime};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{sleep, Duration};

use crate::AppState;

lazy_static::lazy_static! {
    static ref PREVIOUS_VOLUME: Mutex<Option<u8>> = Mutex::new(None);
}

/// Target time-of-day for the ceremony.
const TRIGGER_TIME: NaiveTime = match NaiveTime::from_hms_opt(9, 0, 0) {
    Some(t) => t,
    None => panic!("Invalid trigger time constant"),
};

/// Granularity of the polling loop.
const POLL_INTERVAL: Duration = Duration::from_secs(10);

/// Emitted to the frontend when the ceremony starts.
pub const EVENT_CEREMONY_START: &str = "ceremony:start";
/// Emitted to the frontend when the ceremony ends.
pub const EVENT_CEREMONY_END: &str = "ceremony:end";

/// Main scheduler loop — runs for the lifetime of the application.
pub async fn run(app: AppHandle) {
    log::info!("Scheduler started");

    // Initialize NTP sync on startup (if not using system time only).
    initialize_ntp(&app).await;

    // Track the last date on which we fired so we don't double-trigger.
    let mut last_fired_date: Option<chrono::NaiveDate> = None;

    loop {
        sleep(POLL_INTERVAL).await;

        let now_local = current_local_time(&app).await;
        let today = now_local.date_naive();
        let time = now_local.time();

        // Already fired today?
        if last_fired_date == Some(today) {
            continue;
        }

        // Check skip flag and weekdays.
        {
            let state = app.state::<AppState>();
            let inner = state.lock();
            if !inner.settings.autostart_enabled {
                continue;
            }
            if inner.skip_date == Some(today) {
                log::info!("Skipping ceremony for {today}");
                last_fired_date = Some(today);
                continue;
            }
            if inner.settings.weekdays_only {
                let weekday = today.weekday();
                if weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun {
                    log::info!("Skipping ceremony for {today} (weekend)");
                    continue;
                }
            }
        }

        let grace = {
            let state = app.state::<AppState>();
            let inner = state.lock();
            inner.settings.late_start_grace_minutes
        };

        let should_fire = is_within_window(time, TRIGGER_TIME, grace);

        if should_fire {
            last_fired_date = Some(today);
            log::info!("Firing ceremony at {now_local}");
            trigger_ceremony(app.clone()).await;
        }
    }
}

/// Initialize NTP synchronization on startup.
async fn initialize_ntp(app: &AppHandle) {
    let state = app.state::<AppState>();
    let ntp_service = state.ntp_service.clone();

    let system_time_only = {
        let inner = state.lock();
        inner.settings.system_time_only
    };

    if system_time_only {
        log::info!("Using system time only (NTP disabled)");
        ntp_service.clear_cache();
        return;
    }

    // Sync once on startup.
    match ntp_service.sync().await {
        Ok(offset) => {
            log::info!("NTP sync successful, offset: {}ms", offset);
            {
                let mut inner = state.lock();
                inner.last_ntp_sync = Some(Local::now());
            }
        }
        Err(e) => {
            log::warn!("NTP sync failed: {e}");
            {
                let mut inner = state.lock();
                inner.last_ntp_sync = Some(Local::now());
            }
        }
    }
}

/// Returns true when `now` is within [target, target + grace_minutes).
fn is_within_window(now: NaiveTime, target: NaiveTime, grace_minutes: u8) -> bool {
    if now < target {
        return false;
    }
    let elapsed_secs = (now - target).num_seconds();
    elapsed_secs <= (grace_minutes as i64) * 60
}

/// Obtain the current local time, correcting with NTP if enabled.
async fn current_local_time(app: &AppHandle) -> chrono::DateTime<Local> {
    let state = app.state::<AppState>();
    let ntp_service = state.ntp_service.clone();

    let system_time_only = {
        let inner = state.lock();
        inner.settings.system_time_only
    };

    if system_time_only {
        return Local::now();
    }

    // Use cached NTP offset if available and not stale.
    if let Some(offset_ms) = ntp_service.get_offset() {
        if !ntp_service.should_sync() {
            return Local::now() + chrono::Duration::milliseconds(offset_ms);
        }
    }

    // Sync if needed.
    match ntp_service.sync().await {
        Ok(offset_ms) => {
            let corrected = Local::now() + chrono::Duration::milliseconds(offset_ms);
            {
                let mut inner = state.lock();
                inner.last_ntp_sync = Some(Local::now());
            }
            return corrected;
        }
        Err(e) => {
            log::warn!("NTP query failed: {e}; falling back to system clock");
        }
    }

    Local::now()
}

/// Orchestrate the ceremony sequence (audio handled in backend via rodio).
pub async fn trigger_ceremony(app: AppHandle) {
    let (should_pause_players, volume_priority, target_volume, preset) = {
        let state = app.state::<AppState>();
        let inner = state.lock();
        (
            inner.settings.pause_other_players,
            inner.settings.volume_priority,
            inner.settings.volume,
            inner.settings.preset,
        )
    };

    // Mark ceremony as active.
    {
        let state = app.state::<AppState>();
        let mut inner = state.lock();
        inner.ceremony_active = true;
        inner.last_activation = Some(Local::now());
    }

    // Notify the frontend that ceremony started.
    let _ = app.emit(EVENT_CEREMONY_START, ());

    // Volume priority: save current volume and set to target.
    if volume_priority {
        #[cfg(target_os = "windows")]
        {
            match crate::platform_windows::volume::get_volume() {
                Ok(vol) => {
                    *PREVIOUS_VOLUME.lock().unwrap() = Some(vol);
                    if let Err(e) = crate::platform_windows::volume::set_volume(target_volume) {
                        log::warn!("Could not set volume: {e}");
                    } else {
                        log::info!("Volume priority: saved {}%, set to {}%", vol, target_volume);
                    }
                }
                Err(e) => log::warn!("Could not get current volume: {e}"),
            }
        }
        #[cfg(target_os = "linux")]
        {
            match crate::platform_linux::volume::get_volume() {
                Ok(vol) => {
                    *PREVIOUS_VOLUME.lock().unwrap() = Some(vol);
                    if let Err(e) = crate::platform_linux::volume::set_volume(target_volume) {
                        log::warn!("Could not set volume: {e}");
                    } else {
                        log::info!("Volume priority: saved {}%, set to {}%", vol, target_volume);
                    }
                }
                Err(e) => log::warn!("Could not get current volume: {e}"),
            }
        }
    }

    // Pause other media players before playback starts (if enabled).
    if should_pause_players {
        #[cfg(target_os = "windows")]
        {
            if let Err(e) = crate::platform_windows::media::pause_all() {
                log::warn!("Could not pause media players: {e}");
            }
        }
        #[cfg(target_os = "linux")]
        {
            if let Err(e) = crate::platform_linux::media::pause_all() {
                log::warn!("Could not pause media players: {e}");
            }
        }
    }

    // Stop any previous audio playback before starting new one.
    crate::core::audio::stop();

    // Play audio in backend (blocking, runs in spawned task so doesn't block scheduler).
    let app_clone = app.clone();
    std::thread::spawn(move || {
        log::info!("Starting audio playback for preset: {:?}", preset);
        match crate::core::audio::play_preset(preset, target_volume) {
            Ok(()) => {
                log::info!("Audio playback completed");
            }
            Err(e) => {
                log::error!("Audio playback failed: {}", e);
            }
        }

        // Notify that ceremony finished.
        tauri::async_runtime::spawn(async move {
            finish_ceremony(app_clone).await;
        });
    });
}

/// Called when the ceremony audio sequence completes.
pub async fn finish_ceremony(app: AppHandle) {
    let (should_resume_players, volume_priority) = {
        let state = app.state::<AppState>();
        let inner = state.lock();
        if !inner.ceremony_active {
            return; // Already finished by the frontend early return.
        }
        (
            inner.settings.pause_other_players,
            inner.settings.volume_priority,
        )
    };

    {
        let state = app.state::<AppState>();
        let mut inner = state.lock();
        inner.ceremony_active = false;
    }

    // Restore previous volume if volume priority was enabled.
    if volume_priority {
        let prev_volume = *PREVIOUS_VOLUME.lock().unwrap();
        if let Some(vol) = prev_volume {
            #[cfg(target_os = "windows")]
            {
                if let Err(e) = crate::platform_windows::volume::set_volume(vol) {
                    log::warn!("Could not restore volume: {e}");
                } else {
                    log::info!("Volume restored to {}%", vol);
                }
            }
            #[cfg(target_os = "linux")]
            {
                if let Err(e) = crate::platform_linux::volume::set_volume(vol) {
                    log::warn!("Could not restore volume: {e}");
                } else {
                    log::info!("Volume restored to {}%", vol);
                }
            }
            *PREVIOUS_VOLUME.lock().unwrap() = None;
        }
    }

    if should_resume_players {
        #[cfg(target_os = "windows")]
        {
            if let Err(e) = crate::platform_windows::media::resume_all() {
                log::warn!("Could not resume media players: {e}");
            }
        }
        #[cfg(target_os = "linux")]
        {
            if let Err(e) = crate::platform_linux::media::resume_all() {
                log::warn!("Could not resume media players: {e}");
            }
        }
    }

    let _ = app.emit(EVENT_CEREMONY_END, ());
    log::info!("Ceremony finished");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fires_exactly_at_target() {
        let target = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        assert!(is_within_window(target, target, 5));
    }

    #[test]
    fn fires_within_grace() {
        let target = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let now = NaiveTime::from_hms_opt(9, 4, 59).unwrap();
        assert!(is_within_window(now, target, 5));
    }

    #[test]
    fn does_not_fire_after_grace() {
        let target = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let now = NaiveTime::from_hms_opt(9, 5, 1).unwrap();
        assert!(!is_within_window(now, target, 5));
    }

    #[test]
    fn does_not_fire_before_target() {
        let target = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let now = NaiveTime::from_hms_opt(8, 59, 59).unwrap();
        assert!(!is_within_window(now, target, 5));
    }

    #[test]
    fn grace_zero_fires_only_at_exact_second() {
        let target = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        assert!(is_within_window(target, target, 0));
        let one_sec_later = NaiveTime::from_hms_opt(9, 0, 1).unwrap();
        assert!(!is_within_window(one_sec_later, target, 0));
    }
}
