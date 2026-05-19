use crate::core::audio::AudioEngine;
use crate::core::settings::AudioPreset;
use crate::platform::Platform;
use crate::state::AppState;
use rust_i18n::t;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Listener, Manager, WebviewWindowBuilder};

lazy_static::lazy_static! {
    static ref PREVIOUS_VOLUME: Mutex<Option<u8>> = Mutex::new(None);
    static ref WAS_MUTED: Mutex<Option<bool>> = Mutex::new(None);
    static ref PAUSED_PLAYERS: Mutex<Vec<String>> = Mutex::new(Vec::new());
    static ref ACTIVE_TRIGGER_COUNT: AtomicU32 = AtomicU32::new(0);
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
        let (
            should_pause_players,
            should_resume_players,
            volume_priority,
            auto_unmute,
            target_volume,
            preset,
            announcement_voice,
            anthem_voice,
        ) = {
            let state = self.app.state::<AppState>();
            let inner = state.lock();
            (
                inner.settings.pause_other_players,
                inner.settings.resume_after_ceremony,
                inner.settings.volume_priority,
                inner.settings.auto_unmute,
                inner.settings.volume,
                inner.settings.preset,
                inner.settings.announcement_voice,
                inner.settings.anthem_voice,
            )
        };

        // 1. Mark active
        {
            let state = self.app.state::<AppState>();
            let mut inner = state.lock();
            inner.ceremony_active = true;
            inner.last_activation = Some(chrono::Local::now());
        }
        ACTIVE_TRIGGER_COUNT.fetch_add(1, Ordering::SeqCst);

        // 2. Setup flag animation listeners if conditions are met
        let should_show_flag = {
            let state = self.app.state::<AppState>();
            let inner = state.lock();
            inner.settings.show_flag_animation
                && inner.settings.show_visual_overlay
                && preset.has_anthem()
        };
        if should_show_flag {
            let app_clone = self.app.clone();
            let _ = self.app.once("anthem-start", move |_| {
                let app = app_clone.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = WebviewWindowBuilder::new(
                        &app,
                        "flag-animation",
                        tauri::WebviewUrl::App(std::path::PathBuf::from("flag-animation.html")),
                    )
                    .title(t!("tray_tooltip").as_ref())
                    .fullscreen(true)
                    .decorations(false)
                    .transparent(true)
                    .always_on_top(false)
                    .skip_taskbar(false)
                    .build()
                    {
                        log::warn!("Failed to create flag animation window: {}", e);
                    }
                });
            });

            let app_clone2 = self.app.clone();
            let _ = self.app.once("anthem-end", move |_| {
                let app = app_clone2.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(window) = app.get_webview_window("flag-animation") {
                        let _ = window.close();
                    }
                });
            });
        }

        // 3. Notify UI
        let _ = self.app.emit("ceremony-start", ());

        // 3. Pause players
        if should_pause_players {
            if let Ok(players) = self.platform.pause_media().await {
                if should_resume_players {
                    let mut lock = PAUSED_PLAYERS.lock().unwrap();
                    *lock = players;
                }
            }
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
        let platform_handle = crate::platform::get_platform();

        std::thread::spawn(move || {
            if let Err(e) =
                audio_engine.play_preset(preset, target_volume, announcement_voice, anthem_voice)
            {
                log::error!("Ceremony audio error: {}", e);
            }

            // 7. Finish
            tauri::async_runtime::spawn(async move {
                CeremonyManager::finish_ceremony(app_handle, platform_handle).await;
            });
        });
    }

    pub async fn finish_ceremony(app: AppHandle, platform: Box<dyn Platform>) {
        // Decrement trigger count - only finish if this was the last one
        let count = ACTIVE_TRIGGER_COUNT.fetch_sub(1, Ordering::SeqCst);
        if count == 1 {
            // This was the last trigger, proceed to finish
        } else if count > 1 {
            // More triggers still active, skip restoration and emit
            return;
        } else {
            // count was 0, nothing to do
            return;
        }

        let (volume_priority, auto_unmute, should_resume) = {
            let state = app.state::<AppState>();
            let inner = state.lock();
            if !inner.ceremony_active {
                return;
            }
            (
                inner.settings.volume_priority,
                inner.settings.auto_unmute,
                inner.settings.resume_after_ceremony,
            )
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

        // Restore media playback
        if should_resume {
            let players = {
                let mut lock = PAUSED_PLAYERS.lock().unwrap();
                std::mem::take(&mut *lock)
            };
            if !players.is_empty() {
                let _ = platform.resume_media(players).await;
            }
        }

        // Clear skip_date after successful ceremony
        {
            let state = app.state::<AppState>();
            let mut inner = state.lock();
            if inner.settings.skip_date.is_some() {
                inner.settings.skip_date = None;
                if let Err(e) = inner.settings.save() {
                    log::warn!("Failed to clear skip_date: {}", e);
                }
            }
        }

        let _ = app.emit("ceremony-end", ());
    }
}
