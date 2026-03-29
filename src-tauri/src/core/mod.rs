pub mod audio;
pub mod ntp;
pub mod ntp_service;
pub mod platform;
pub mod scheduler;
pub mod settings;

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use crate::state::AppState;
use crate::core::platform::Platform;
use crate::core::audio::AudioEngine;

lazy_static::lazy_static! {
    static ref PREVIOUS_VOLUME: Mutex<Option<u8>> = Mutex::new(None);
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
        Self { app, platform, audio }
    }

    pub async fn run_ceremony(&self) {
        let (should_pause_players, volume_priority, target_volume, preset) = {
            let state = self.app.state::<AppState>();
            let inner = state.lock();
            (
                inner.settings.pause_other_players,
                inner.settings.volume_priority,
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

        // 3. Handle Volume
        if volume_priority {
            if let Ok(vol) = self.platform.get_volume() {
                *PREVIOUS_VOLUME.lock().unwrap() = Some(vol);
                let _ = self.platform.set_volume(target_volume);
            }
        }

        // 4. Pause players
        if should_pause_players {
            let _ = self.platform.pause_media();
        }

        // 5. Play Audio (Stop previous first)
        self.audio.stop();
        
        let audio_engine = Arc::clone(&self.audio);
        let app_handle = self.app.clone();
        let platform_handle = platform::get_platform(); // Need a fresh one for the thread or make it cloneable

        std::thread::spawn(move || {
            let _ = audio_engine.play_preset(preset, target_volume);
            
            // 6. Finish
            tauri::async_runtime::spawn(async move {
                CeremonyManager::finish_ceremony(app_handle, platform_handle).await;
            });
        });
    }

    pub async fn finish_ceremony(app: AppHandle, platform: Box<dyn Platform>) {
        let (should_resume_players, volume_priority) = {
            let state = app.state::<AppState>();
            let inner = state.lock();
            if !inner.ceremony_active { return; }
            (inner.settings.pause_other_players, inner.settings.volume_priority)
        };

        {
            let state = app.state::<AppState>();
            let mut inner = state.lock();
            inner.ceremony_active = false;
        }

        // Restore volume
        if volume_priority {
            let prev = *PREVIOUS_VOLUME.lock().unwrap();
            if let Some(vol) = prev {
                let _ = platform.set_volume(vol);
                *PREVIOUS_VOLUME.lock().unwrap() = None;
            }
        }

        // Resume media
        if should_resume_players {
            let _ = platform.resume_media();
        }

        let _ = app.emit("ceremony-end", ());
    }
}
