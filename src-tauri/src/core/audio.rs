//! Backend audio playback engine.

use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::core::settings::AudioPreset;
use crate::error::{AppError, Result};

/// Audio engine for ceremony playback.
/// This acts as a class responsible for all audio operations.
#[derive(Debug)]
pub struct AudioEngine {
    stop_counter: AtomicU64,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
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

    fn wait_sink_interruptible(&self, sink: &Sink, start_counter: u64) -> bool {
        while !sink.empty() {
            if self.is_stopped(start_counter) {
                sink.stop();
                return true;
            }
            thread::sleep(Duration::from_millis(50));
        }
        false
    }

    pub fn play_preset(&self, preset: AudioPreset, volume: u8) -> Result<()> {
        let start_counter = self.stop_counter.load(Ordering::SeqCst);

        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| AppError::Audio(format!("Failed to open audio stream: {e}")))?;

        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| AppError::Audio(format!("Failed to create audio sink: {e}")))?;

        let volume_float = volume as f32 / 100.0;
        sink.set_volume(volume_float);

        match preset {
            AudioPreset::VoiceMetronome => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let announcement = self.get_path("announcement.ogg");
                let metronome = self.get_path("metronome.ogg");

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                    sink.append(source);
                }
                if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                    sink.append(source);
                }
            }
            AudioPreset::MetronomeOnly => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let metronome = self.get_path("metronome.ogg");
                if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                    sink.append(source);
                }
            }
            AudioPreset::VoiceSilenceBell => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let announcement = self.get_path("announcement.ogg");
                let bell = self.get_path("bell.ogg");

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                    sink.append(source);
                }

                if self.wait_sink_interruptible(&sink, start_counter) {
                    return Ok(());
                }
                if self.sleep_interruptible(Duration::from_secs(60), start_counter) {
                    return Ok(());
                }

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell)?)) {
                    sink.append(source);
                }
            }
            AudioPreset::VoiceSilence => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let announcement = self.get_path("announcement.ogg");
                if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                    sink.append(source);
                }
                if self.wait_sink_interruptible(&sink, start_counter) {
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
                let announcement = self.get_path("announcement.ogg");
                let metronome = self.get_path("metronome.ogg");
                let anthem = self.get_path("anthem.ogg");

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                    sink.append(source);
                }

                if self.wait_sink_interruptible(&sink, start_counter) {
                    return Ok(());
                }
                if self.sleep_interruptible(Duration::from_secs(1), start_counter) {
                    return Ok(());
                }

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                    sink.append(source);
                }

                if self.sleep_interruptible(Duration::from_secs(30), start_counter) {
                    return Ok(());
                }

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&anthem)?)) {
                    sink.append(source);
                }
            }
            AudioPreset::MetronomeAnthem => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let metronome = self.get_path("metronome.ogg");
                let anthem = self.get_path("anthem.ogg");

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                    sink.append(source);
                }

                if self.sleep_interruptible(Duration::from_secs(30), start_counter) {
                    return Ok(());
                }

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&anthem)?)) {
                    sink.append(source);
                }
            }
            AudioPreset::BellSilenceBell => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let bell = self.get_path("bell.ogg");

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell)?)) {
                    sink.append(source);
                }

                if self.wait_sink_interruptible(&sink, start_counter) {
                    return Ok(());
                }
                if self.sleep_interruptible(Duration::from_secs(60), start_counter) {
                    return Ok(());
                }

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell)?)) {
                    sink.append(source);
                }
            }
            AudioPreset::BellMetronomeBell => {
                if self.is_stopped(start_counter) {
                    return Ok(());
                }
                let bell = self.get_path("bell.ogg");
                let metronome = self.get_path("metronome.ogg");

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell)?)) {
                    sink.append(source);
                }

                if self.wait_sink_interruptible(&sink, start_counter) {
                    return Ok(());
                }
                if self.sleep_interruptible(Duration::from_secs(1), start_counter) {
                    return Ok(());
                }

                if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                    sink.append(source);
                }

                if self.sleep_interruptible(Duration::from_secs(58), start_counter) {
                    return Ok(());
                }

                let bell2 = self.get_path("bell.ogg");
                if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell2)?)) {
                    sink.append(source);
                }
            }
        }

        self.wait_sink_interruptible(&sink, start_counter);
        Ok(())
    }

    fn get_path(&self, filename: &str) -> PathBuf {
        PathBuf::from("audio/").join(filename)
    }
}
