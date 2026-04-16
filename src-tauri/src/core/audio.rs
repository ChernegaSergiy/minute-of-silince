//! Backend audio playback engine.

use rodio::{Decoder, DeviceSinkBuilder, Player, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

use crate::core::settings::AudioPreset;
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
        let path = self.get_path(filename)?;
        let source = Decoder::new(BufReader::new(File::open(&path)?))
            .map_err(|e| AppError::Audio(format!("Failed to decode audio file: {}", e)))?;
        let duration = source.total_duration().unwrap_or(Duration::ZERO);
        Ok(duration)
    }

    pub fn play_preset(&self, preset: AudioPreset, volume: u8) -> Result<()> {
        let start_counter = self.stop_counter.load(Ordering::SeqCst);

        let sink = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| AppError::Audio(format!("Failed to open audio stream: {e}")))?;

        let mixer = sink.mixer();
        let player = Player::connect_new(mixer);

        let volume_float = volume as f32 / 100.0;
        player.set_volume(volume_float);

        match preset {
            AudioPreset::VoiceMetronome => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let announcement = self.get_path("announcement.ogg")?;
                let metronome = self.get_path("metronome.ogg")?;

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                    player.append(source);
                }
                if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                    player.append(source);
                }
            }
            AudioPreset::MetronomeOnly => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let metronome = self.get_path("metronome.ogg")?;
                if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                    player.append(source);
                }
            }
            AudioPreset::VoiceSilenceBell => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let announcement = self.get_path("announcement.ogg")?;
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
            AudioPreset::VoiceSilence => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let announcement = self.get_path("announcement.ogg")?;
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
            AudioPreset::VoiceMetronomeAnthem => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let announcement = self.get_path("announcement.ogg")?;
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
            AudioPreset::MetronomeAnthem => {
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
            AudioPreset::BellSilenceBell => {
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
            AudioPreset::BellMetronomeBell => {
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
            AudioPreset::Silence => {
                // No audio, just wait for 60 seconds
                if self.sleep_interruptible(Duration::from_secs(60), start_counter) {
                    return Ok(());
                }
            }
        }

        self.wait_player_interruptible(&player, start_counter);
        Ok(())
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

        log::info!("Audio resource path: {:?}", path);
        Ok(path)
    }
}
