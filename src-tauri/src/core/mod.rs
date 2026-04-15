pub mod audio;
pub mod ntp;
pub mod ntp_service;
pub mod platform;
pub mod scheduler;
pub mod settings;

use crate::core::audio::AudioEngine;
use crate::core::platform::Platform;
use crate::core::settings::AudioPreset;
use crate::state::AppState;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

lazy_static::lazy_static! {
    static ref PREVIOUS_VOLUME: Mutex<Option<u8>> = Mutex::new(None);
    static ref WAS_MUTED: Mutex<Option<bool>> = Mutex::new(None);
}

/// Orchestrator for the ceremony.
/// This class manages the sequence of events during the ceremony.
pub struct CeremonyManager {
    app: AppHandle,
    platform: Box<dyn Platform>,
    audio: Arc<AudioEngine>,
}

impl CeremonyManager {
    pub fn new(app: AppHandle, platform: Box<dyn Platform>, audio: Arc<AudioEngine>) -> Self {
        Self {
            app,
            platform,
            audio,
        }
    }

    pub async fn run_ceremony(&self) {
        let (should_pause_players, volume_priority, auto_unmute, target_volume, preset) = {
            let state = self.app.state::<AppState>();
            let inner = state.lock();
            (
                inner.settings.pause_other_players,
                inner.settings.volume_priority,
                inner.settings.auto_unmute,
                inner.settings.volume,
                inner.settings.preset,
            )
        };

        // 1. Mark active
        {
            let state = self.app.state::<AppState>();
            let mut inner = state.lock();
            inner.ceremony_active = true;
            inner.last_activation = Some(chrono::Local::now());
        }

        // 2. Notify UI
        let _ = self.app.emit("ceremony-start", ());

        // 3. Pause players
        if should_pause_players {
            let _ = self.platform.pause_media().await;
        }

        // 4. Handle Volume and Mute (skip for Silence preset)
        if auto_unmute && preset != AudioPreset::Silence {
            // Save mute state and unmute if necessary
            if let Ok(muted) = self.platform.is_muted() {
                if muted {
                    *WAS_MUTED.lock().unwrap() = Some(true);
                    let _ = self.platform.set_mute(false);
                }
            }
        }

        if volume_priority && preset != AudioPreset::Silence {
            // Save and set volume
            if let Ok(vol) = self.platform.get_volume() {
                *PREVIOUS_VOLUME.lock().unwrap() = Some(vol);
                let _ = self.platform.set_volume(target_volume);
            }
        }

        // 6. Play Audio (Stop previous first)
        self.audio.stop();

        let audio_engine = Arc::clone(&self.audio);
        let app_handle = self.app.clone();
        let platform_handle = platform::get_platform();

        std::thread::spawn(move || {
            if let Err(e) = audio_engine.play_preset(preset, target_volume) {
                log::error!("Ceremony audio error: {}", e);
            }

            // 7. Finish
            tauri::async_runtime::spawn(async move {
                CeremonyManager::finish_ceremony(app_handle, platform_handle).await;
            });
        });
    }

    pub async fn finish_ceremony(app: AppHandle, platform: Box<dyn Platform>) {
        let (volume_priority, auto_unmute) = {
            let state = app.state::<AppState>();
            let inner = state.lock();
            if !inner.ceremony_active {
                return;
            }
            (inner.settings.volume_priority, inner.settings.auto_unmute)
        };

        {
            let state = app.state::<AppState>();
            let mut inner = state.lock();
            inner.ceremony_active = false;
        }

        // Restore volume and mute
        if volume_priority {
            let prev_vol = *PREVIOUS_VOLUME.lock().unwrap();
            if let Some(vol) = prev_vol {
                let _ = platform.set_volume(vol);
                *PREVIOUS_VOLUME.lock().unwrap() = None;
            }
        }

        if auto_unmute {
            let was_muted = *WAS_MUTED.lock().unwrap();
            if let Some(true) = was_muted {
                let _ = platform.set_mute(true);
                *WAS_MUTED.lock().unwrap() = None;
            }
        }

        let _ = app.emit("ceremony-end", ());
    }
}
