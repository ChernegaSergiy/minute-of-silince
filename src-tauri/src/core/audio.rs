//! Backend audio playback engine.

use rodio::{Decoder, DeviceSinkBuilder, Player};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

use crate::core::settings::{AnnouncementVoice, AudioPreset};
use crate::error::{AppError, Result};

/// Audio engine for ceremony playback.
/// This acts as a class responsible for all audio operations.
#[derive(Debug)]
pub struct AudioEngine {
    app_handle: AppHandle,
    stop_counter: AtomicU64,
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
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let path = self.get_path(filename)?;
        let file = File::open(&path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        hint.with_extension("ogg");

        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| AppError::Audio(format!("Failed to probe audio file: {}", e)))?;

        let format = probed.format;

        if let Some(track) = format.default_track() {
            if let Some(n_frames) = track.codec_params.n_frames {
                if let Some(sample_rate) = track.codec_params.sample_rate {
                    let duration_secs = n_frames as f64 / sample_rate as f64;
                    return Ok(Duration::from_secs_f64(duration_secs));
                }
            }
            if let Some(time_base) = track.codec_params.time_base {
                if let Some(n_frames) = track.codec_params.n_frames {
                    let duration_secs =
                        n_frames as f64 * time_base.numer as f64 / time_base.denom as f64;
                    return Ok(Duration::from_secs_f64(duration_secs));
                }
            }
        }

        log::warn!(
            "Could not determine duration of {}, using default 2s",
            filename
        );
        Ok(Duration::from_secs(2))
    }

    pub fn play_preset(
        &self,
        preset: AudioPreset,
        volume: u8,
        voice: AnnouncementVoice,
    ) -> Result<()> {
        let start_counter = self.stop_counter.load(Ordering::SeqCst);

        let sink = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| AppError::Audio(format!("Failed to open audio stream: {e}")))?;

        let mixer = sink.mixer();
        let player = Player::connect_new(mixer);

        let volume_float = volume as f32 / 100.0;
        player.set_volume(volume_float);

        match (preset, voice) {
            (AudioPreset::VoiceMetronome, _) => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let announcement_file = self.get_announcement_filename(voice);
                let announcement = self.get_path(&announcement_file)?;
                let metronome = self.get_path("metronome.ogg")?;

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                    player.append(source);
                }
                if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                    player.append(source);
                }
            }
            (AudioPreset::MetronomeOnly, _) => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let metronome = self.get_path("metronome.ogg")?;
                if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                    player.append(source);
                }
            }
            (AudioPreset::VoiceSilenceBell, _) => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let announcement_file = self.get_announcement_filename(voice);
                let announcement = self.get_path(&announcement_file)?;
                let bell = self.get_path("bell.ogg")?;

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                    player.append(source);
                }

                if self.wait_player_interruptible(&player, start_counter) {
                    return Ok(());
                }
                if self.sleep_interruptible(Duration::from_secs(60), start_counter) {
                    return Ok(());
                }

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell)?)) {
                    player.append(source);
                }
            }
            (AudioPreset::VoiceSilence, _) => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let announcement_file = self.get_announcement_filename(voice);
                let announcement = self.get_path(&announcement_file)?;
                if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                    player.append(source);
                }
                if self.wait_player_interruptible(&player, start_counter) {
                    return Ok(());
                }
                if self.sleep_interruptible(Duration::from_secs(60), start_counter) {
                    return Ok(());
                }
                return Ok(());
            }
            (AudioPreset::VoiceMetronomeAnthem, _) => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let announcement_file = self.get_announcement_filename(voice);
                let announcement = self.get_path(&announcement_file)?;
                let metronome = self.get_path("metronome.ogg")?;
                let anthem = self.get_path("anthem.ogg")?;

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                    player.append(source);
                }

                if self.wait_player_interruptible(&player, start_counter) {
                    return Ok(());
                }
                if self.sleep_interruptible(Duration::from_secs(1), start_counter) {
                    return Ok(());
                }

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                    player.append(source);
                }

                if self.sleep_interruptible(Duration::from_secs(30), start_counter) {
                    return Ok(());
                }

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&anthem)?)) {
                    player.append(source);
                }
            }
            (AudioPreset::MetronomeAnthem, _) => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let metronome = self.get_path("metronome.ogg")?;
                let anthem = self.get_path("anthem.ogg")?;

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                    player.append(source);
                }

                if self.sleep_interruptible(Duration::from_secs(30), start_counter) {
                    return Ok(());
                }

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&anthem)?)) {
                    player.append(source);
                }
            }
            (AudioPreset::BellSilenceBell, _) => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let bell = self.get_path("bell.ogg")?;

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell)?)) {
                    player.append(source);
                }

                if self.wait_player_interruptible(&player, start_counter) {
                    return Ok(());
                }
                if self.sleep_interruptible(Duration::from_secs(60), start_counter) {
                    return Ok(());
                }

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell)?)) {
                    player.append(source);
                }
            }
            (AudioPreset::BellMetronomeBell, _) => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let bell = self.get_path("bell.ogg")?;
                let metronome = self.get_path("metronome.ogg")?;

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell)?)) {
                    player.append(source);
                }

                if self.wait_player_interruptible(&player, start_counter) {
                    return Ok(());
                }
                if self.sleep_interruptible(Duration::from_secs(1), start_counter) {
                    return Ok(());
                }

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                    player.append(source);
                }

                if self.sleep_interruptible(Duration::from_secs(58), start_counter) {
                    return Ok(());
                }

                let bell2 = self.get_path("bell.ogg")?;
                if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell2)?)) {
                    player.append(source);
                }
            }
            (AudioPreset::Silence, _) => {
                // No audio, just wait for 60 seconds
                if self.sleep_interruptible(Duration::from_secs(60), start_counter) {
                    return Ok(());
                }
            }
        }

        self.wait_player_interruptible(&player, start_counter);
        Ok(())
    }

    fn get_announcement_filename(&self, voice: AnnouncementVoice) -> String {
        match voice {
            AnnouncementVoice::BohdanHdal => "announcement.ogg".to_string(),
            AnnouncementVoice::SoniaSotnyk => "announcement_sotnyk.ogg".to_string(),
            AnnouncementVoice::DaniaKhomutovskyi => "announcement_khomutovskyi.ogg".to_string(),
            AnnouncementVoice::AirAlert => "announcement_air_alert.ogg".to_string(),
        }
    }

    /// Resolves the absolute path to an audio resource using Tauri's path resolver.
    /// Works on all platforms: EXE, MSI, MSIX, AppImage, Snap.
    fn get_path(&self, filename: &str) -> Result<PathBuf> {
        let resource_path = format!("audio/{}", filename);

        let path = self
            .app_handle
            .path()
            .resolve(&resource_path, tauri::path::BaseDirectory::Resource)
            .map_err(|e| {
                AppError::Audio(format!(
                    "Failed to resolve audio path '{}': {}",
                    filename, e
                ))
            })?;

        log::debug!("Audio resource path: {:?}", path);
        Ok(path)
    }
}
