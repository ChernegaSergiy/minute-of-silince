//! Backend audio playback engine.

use rodio::{Decoder, DeviceSinkBuilder, Player};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

use crate::core::settings::{AnnouncementVoice, AnthemVoice, AudioPreset};
use crate::error::{AppError, Result};

/// Audio engine for ceremony playback.
/// This acts as a class responsible for all audio operations.
#[derive(Debug)]
pub struct AudioEngine {
    app_handle: AppHandle,
    stop_counter: AtomicU64,
}

/// A single step in a preset playback sequence.
enum Step {
    File(String),
    Pause(Duration),
    Wait,           // wait for the current player to drain
    Anthem(String), // like File but emits anthem-start/anthem-end events
}

impl AudioEngine {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            stop_counter: AtomicU64::new(0),
        }
    }

    /// Stop any current playback by incrementing the counter.
    pub fn stop(&self) {
        self.stop_counter.fetch_add(1, Ordering::SeqCst);
    }

    fn is_stopped(&self, start_counter: u64) -> bool {
        self.stop_counter.load(Ordering::SeqCst) > start_counter
    }

    fn sleep_interruptible(&self, duration: Duration, start_counter: u64) -> bool {
        let start = Instant::now();
        while start.elapsed() < duration {
            if self.is_stopped(start_counter) {
                return true;
            }
            thread::sleep(Duration::from_millis(50));
        }
        false
    }

    fn wait_player_interruptible(&self, player: &Player, start_counter: u64) -> bool {
        while !player.empty() {
            if self.is_stopped(start_counter) {
                player.stop();
                return true;
            }
            thread::sleep(Duration::from_millis(50));
        }
        false
    }

    pub fn get_duration(&self, filename: &str) -> Result<Duration> {
        use symphonia::core::formats::FormatOptions;
        use symphonia::core::formats::TrackType;
        use symphonia::core::formats::probe::Hint;
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;

        let path = self.get_path(filename)?;
        let mss = MediaSourceStream::new(Box::new(File::open(&path)?), Default::default());

        let mut hint = Hint::new();
        hint.with_extension("ogg");

        let format = symphonia::default::get_probe()
            .probe(
                &hint,
                mss,
                FormatOptions::default(),
                MetadataOptions::default(),
            )
            .map_err(|e| AppError::Audio(format!("Failed to probe audio file: {e}")))?;

        let track = format
            .default_track(TrackType::Audio)
            .ok_or_else(|| AppError::Audio(format!("No audio track in {filename}")))?;

        let (duration, time_base) = track
            .duration
            .zip(track.time_base)
            .ok_or_else(|| AppError::Audio(format!("No duration metadata in {filename}")))?;

        let secs =
            duration.get() as f64 * time_base.numer.get() as f64 / time_base.denom.get() as f64;
        Ok(Duration::from_secs_f64(secs))
    }

    pub fn play_preset(
        &self,
        preset: AudioPreset,
        volume: u8,
        voice: AnnouncementVoice,
        anthem_voice: AnthemVoice,
    ) -> Result<()> {
        let start_counter = self.stop_counter.load(Ordering::SeqCst);

        let sink = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| AppError::Audio(format!("Failed to open audio stream: {e}")))?;

        let mixer = sink.mixer();
        let player = Player::connect_new(mixer);

        let volume_float = volume as f32 / 100.0;
        player.set_volume(volume_float);

        // Build the playback step sequence once and execute it.
        let steps = self.preset_steps(preset, voice, anthem_voice);

        for step in steps {
            if self.is_stopped(start_counter) {
                return Ok(());
            }

            match step {
                Step::File(fname) => {
                    let path = self.get_path(&fname)?;
                    if let Ok(source) = Decoder::new(BufReader::new(File::open(&path)?)) {
                        player.append(source);
                    }
                }
                Step::Pause(dur) => {
                    if self.sleep_interruptible(dur, start_counter) {
                        return Ok(());
                    }
                }
                Step::Wait => {
                    if self.wait_player_interruptible(&player, start_counter) {
                        return Ok(());
                    }
                }
                Step::Anthem(fname) => {
                    let anthem_path = self.get_path(&fname)?;
                    let _ = self.app_handle.emit("anthem-start", ());
                    if let Ok(source) = Decoder::new(BufReader::new(File::open(&anthem_path)?)) {
                        player.append(source);
                    }
                    if self.wait_player_interruptible(&player, start_counter) {
                        let _ = self.app_handle.emit("anthem-end", ());
                        return Ok(());
                    }
                    let _ = self.app_handle.emit("anthem-end", ());
                }
            }
        }

        self.wait_player_interruptible(&player, start_counter);
        Ok(())
    }

    /// Return a list of ordered steps for the given preset.
    fn preset_steps(
        &self,
        preset: AudioPreset,
        voice: AnnouncementVoice,
        anthem_voice: AnthemVoice,
    ) -> Vec<Step> {
        use AudioPreset::*;

        let mut out = Vec::new();

        match preset {
            VoiceMetronome => {
                out.push(Step::File(self.get_announcement_filename(voice)));
                out.push(Step::Wait);
                out.push(Step::Pause(Duration::from_secs(1)));
                out.push(Step::File("metronome.ogg".to_string()));
            }
            MetronomeOnly => {
                out.push(Step::File("metronome.ogg".to_string()));
            }
            VoiceSilenceBell => {
                out.push(Step::File(self.get_announcement_filename(voice)));
                out.push(Step::Wait);
                out.push(Step::Pause(Duration::from_secs(60)));
                out.push(Step::File("bell.ogg".to_string()));
            }
            VoiceSilence => {
                out.push(Step::File(self.get_announcement_filename(voice)));
                out.push(Step::Wait);
                out.push(Step::Pause(Duration::from_secs(60)));
            }
            VoiceMetronomeAnthem => {
                out.push(Step::File(self.get_announcement_filename(voice)));
                out.push(Step::Wait);
                out.push(Step::Pause(Duration::from_secs(1)));
                out.push(Step::File("metronome.ogg".to_string()));
                out.push(Step::Wait);
                out.push(Step::Anthem(self.get_anthem_filename(anthem_voice)));
            }
            VoiceMetronomeEnding => {
                out.push(Step::File(self.get_announcement_filename(voice)));
                out.push(Step::Wait);
                out.push(Step::Pause(Duration::from_secs(1)));
                out.push(Step::File("metronome.ogg".to_string()));
                out.push(Step::File(self.get_ending_filename(voice)));
            }
            MetronomeAnthem => {
                out.push(Step::File("metronome.ogg".to_string()));
                out.push(Step::Wait);
                out.push(Step::Anthem(self.get_anthem_filename(anthem_voice)));
            }
            BellSilenceBell => {
                out.push(Step::File("bell.ogg".to_string()));
                out.push(Step::Wait);
                out.push(Step::Pause(Duration::from_secs(60)));
                out.push(Step::File("bell.ogg".to_string()));
            }
            BellMetronomeBell => {
                out.push(Step::File("bell.ogg".to_string()));
                out.push(Step::Wait);
                out.push(Step::Pause(Duration::from_secs(1)));
                out.push(Step::File("metronome.ogg".to_string()));
                out.push(Step::File("bell.ogg".to_string()));
            }
            Silence => {
                out.push(Step::Pause(Duration::from_secs(60)));
            }
        }

        out
    }

    /// Estimate total duration of a preset by summing file durations and explicit pauses.
    pub fn estimate_preset_duration(
        &self,
        preset: AudioPreset,
        voice: AnnouncementVoice,
        anthem_voice: AnthemVoice,
    ) -> Result<Duration> {
        let mut total = Duration::from_secs(0);
        let steps = self.preset_steps(preset, voice, anthem_voice);
        for step in &steps {
            match step {
                Step::File(f) | Step::Anthem(f) => {
                    let dur = self.get_duration(f)?;
                    total = total.checked_add(dur).unwrap_or(total);
                }
                Step::Pause(d) => {
                    total = total.checked_add(*d).unwrap_or(total);
                }
                Step::Wait => {
                    // no-op: waits are covered by file durations
                }
            }
        }
        Ok(total)
    }

    fn get_announcement_filename(&self, voice: AnnouncementVoice) -> String {
        match voice {
            AnnouncementVoice::BohdanHdal => "announcement.ogg".to_string(),
            AnnouncementVoice::SoniaSotnyk => "announcement_sotnyk.ogg".to_string(),
            AnnouncementVoice::DaniaKhomutovskyi => "announcement_khomutovskyi.ogg".to_string(),
            AnnouncementVoice::RadioBg => "announcement_radio_bg.ogg".to_string(),
            AnnouncementVoice::AirAlert => "announcement_air_alert.ogg".to_string(),
        }
    }

    fn get_anthem_filename(&self, voice: AnthemVoice) -> String {
        match voice {
            AnthemVoice::Default => "anthem.ogg".to_string(),
            AnthemVoice::MykhailoKhoma => "anthem_khoma.ogg".to_string(),
            AnthemVoice::OleksandrPonomarov => "anthem_ponomarov.ogg".to_string(),
        }
    }

    fn get_ending_filename(&self, voice: AnnouncementVoice) -> Option<String> {
        match voice {
            AnnouncementVoice::BohdanHdal => Some("ending.ogg".to_string()),
            AnnouncementVoice::SoniaSotnyk => Some("ending_sotnyk.ogg".to_string()),
            AnnouncementVoice::DaniaKhomutovskyi => Some("ending_khomutovskyi.ogg".to_string()),
            AnnouncementVoice::RadioBg => Some("ending_radio_bg.ogg".to_string()),
            AnnouncementVoice::AirAlert => None,
        }
    }

    /// Resolves the absolute path to an audio resource using Tauri's path resolver.
    /// Works on all platforms: EXE, MSI, MSIX, AppImage, Snap.
    fn get_path(&self, filename: &str) -> Result<PathBuf> {
        let resource_path = format!("audio/{}", filename);

        // 1. Try Tauri's standard resource resolver
        let tauri_path = self
            .app_handle
            .path()
            .resolve(&resource_path, tauri::path::BaseDirectory::Resource)
            .ok();

        // 2. Try relative to the executable (common in Snap bin/)
        let exe_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("audio").join(filename)));

        // 3. Try hardcoded Snap layout path (as a last resort)
        let layout_path = Some(PathBuf::from(format!(
            "/usr/lib/minute-of-silence/audio/{}",
            filename
        )));

        let candidates = vec![tauri_path, exe_path, layout_path];

        for candidate in candidates.into_iter().flatten() {
            if candidate.exists() {
                log::info!("Found audio at: {:?}", candidate);
                return Ok(candidate);
            }
            log::debug!("Audio not found at: {:?}", candidate);
        }

        log::error!(
            "Audio resource {} not found in any candidate path",
            filename
        );
        Err(AppError::Audio(format!(
            "Audio file not found: {}",
            filename
        )))
    }
}
