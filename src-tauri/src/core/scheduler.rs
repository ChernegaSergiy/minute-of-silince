//! Daily scheduler: fires the ceremony at 09:00 local time, optionally
//! corrected by NTP, and handles the post-sleep "late start" grace period.

use chrono::{Local, NaiveTime};
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{sleep, Duration};

use crate::{core::ntp, AppState};

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

        // Check skip flag.
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

/// Returns true when `now` is within [target, target + grace_minutes).
fn is_within_window(now: NaiveTime, target: NaiveTime, grace_minutes: u8) -> bool {
    if now < target {
        return false;
    }
    let elapsed_secs = (now - target).num_seconds();
    elapsed_secs < (grace_minutes as i64) * 60
}

/// Obtain the current local time, correcting with NTP if enabled.
async fn current_local_time(app: &AppHandle) -> chrono::DateTime<Local> {
    let (ntp_enabled, server) = {
        let state = app.state::<AppState>();
        let inner = state.lock();
        (
            inner.settings.ntp_sync_enabled,
            inner.settings.ntp_server.clone(),
        )
    };

    if ntp_enabled {
        match ntp::query_offset(&server).await {
            Ok(offset_ms) => {
                let corrected = Local::now() + chrono::Duration::milliseconds(offset_ms);
                {
                    let state = app.state::<AppState>();
                    let mut inner = state.lock();
                    inner.last_ntp_sync = Some(Local::now());
                }
                return corrected;
            }
            Err(e) => {
                log::warn!("NTP query failed: {e}; falling back to system clock");
            }
        }
    }

    Local::now()
}

/// Orchestrate the ceremony sequence (audio handled elsewhere; here we emit
/// Tauri events and update state).
pub async fn trigger_ceremony(app: AppHandle) {
    // Mark ceremony as active.
    {
        let state = app.state::<AppState>();
        let mut inner = state.lock();
        inner.ceremony_active = true;
        inner.last_activation = Some(Local::now());
    }

    // Pause other media players before playback starts.
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = crate::platform_windows::media::pause_all() {
            log::warn!("Could not pause media players: {e}");
        }
    }

    // Notify the frontend.
    let _ = app.emit(EVENT_CEREMONY_START, ());

    // The audio engine drives the actual ceremony length; here we wait for
    // the maximum possible duration (voice + anthem ≈ 3 min) as a fallback.
    // In a full implementation the audio module calls `finish_ceremony`.
    sleep(Duration::from_secs(180)).await;

    finish_ceremony(app).await;
}

/// Called when the ceremony audio sequence completes.
pub async fn finish_ceremony(app: AppHandle) {
    {
        let state = app.state::<AppState>();
        let mut inner = state.lock();
        inner.ceremony_active = false;
    }

    #[cfg(target_os = "windows")]
    {
        if let Err(e) = crate::platform_windows::media::resume_all() {
            log::warn!("Could not resume media players: {e}");
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
