use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use crate::core::settings::AudioPreset;
use crate::error::{AppError, Result};

static STOP_SIGNAL: AtomicBool = AtomicBool::new(false);

fn get_audio_path(filename: &str) -> PathBuf {
    PathBuf::from("audio/").join(filename)
}

pub fn stop() {
    STOP_SIGNAL.store(true, Ordering::SeqCst);
}

fn check_stop() -> bool {
    if STOP_SIGNAL.load(Ordering::SeqCst) {
        STOP_SIGNAL.store(false, Ordering::SeqCst);
        true
    } else {
        false
    }
}

pub fn play_preset(preset: AudioPreset, volume: u8) -> Result<()> {
    STOP_SIGNAL.store(false, Ordering::SeqCst);

    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| AppError::Audio(format!("Failed to open audio stream: {e}")))?;

    let sink = Sink::try_new(&stream_handle)
        .map_err(|e| AppError::Audio(format!("Failed to create audio sink: {e}")))?;

    let volume_float = volume as f32 / 100.0;
    sink.set_volume(volume_float);

    match preset {
        AudioPreset::VoiceMetronome => {
            if check_stop() {
                return Ok(());
            }
            let announcement = get_audio_path("announcement.ogg");
            let metronome = get_audio_path("metronome.ogg");

            if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                sink.append(source);
            }
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                sink.append(source);
            }
        }
        AudioPreset::MetronomeOnly => {
            if check_stop() {
                return Ok(());
            }
            let metronome = get_audio_path("metronome.ogg");
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                sink.append(source);
            }
        }
        AudioPreset::VoiceSilenceBell => {
            if check_stop() {
                return Ok(());
            }
            let announcement = get_audio_path("announcement.ogg");
            let bell = get_audio_path("bell.ogg");

            if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                sink.append(source);
            }

            drop(sink);
            if check_stop() {
                return Ok(());
            }
            thread::sleep(std::time::Duration::from_secs(60));
            if check_stop() {
                return Ok(());
            }

            let (_stream2, stream_handle2) = OutputStream::try_default()
                .map_err(|e| AppError::Audio(format!("Failed to open audio stream: {e}")))?;
            let sink2 = Sink::try_new(&stream_handle2)
                .map_err(|e| AppError::Audio(format!("Failed to create audio sink: {e}")))?;
            sink2.set_volume(volume_float);

            if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell)?)) {
                sink2.append(source);
            }
            sink2.sleep_until_end();
            return Ok(());
        }
        AudioPreset::VoiceSilence => {
            if check_stop() {
                return Ok(());
            }
            let announcement = get_audio_path("announcement.ogg");
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                sink.append(source);
            }
        }
        AudioPreset::VoiceMetronomeAnthem => {
            if check_stop() {
                return Ok(());
            }
            let announcement = get_audio_path("announcement.ogg");
            let metronome = get_audio_path("metronome.ogg");
            let anthem = get_audio_path("anthem.ogg");

            if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                sink.append(source);
            }

            drop(sink);
            if check_stop() {
                return Ok(());
            }
            thread::sleep(std::time::Duration::from_secs(1));
            if check_stop() {
                return Ok(());
            }

            let (_stream2, stream_handle2) = OutputStream::try_default()
                .map_err(|e| AppError::Audio(format!("Failed to open audio stream: {e}")))?;
            let sink2 = Sink::try_new(&stream_handle2)
                .map_err(|e| AppError::Audio(format!("Failed to create audio sink: {e}")))?;
            sink2.set_volume(volume_float);

            if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                sink2.append(source);
            }

            if check_stop() {
                return Ok(());
            }
            thread::sleep(std::time::Duration::from_secs(30));
            if check_stop() {
                return Ok(());
            }

            if let Ok(source) = Decoder::new(BufReader::new(File::open(&anthem)?)) {
                sink2.append(source);
            }
            sink2.sleep_until_end();
            return Ok(());
        }
        AudioPreset::MetronomeAnthem => {
            if check_stop() {
                return Ok(());
            }
            let metronome = get_audio_path("metronome.ogg");
            let anthem = get_audio_path("anthem.ogg");

            if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                sink.append(source);
            }

            if check_stop() {
                return Ok(());
            }
            thread::sleep(std::time::Duration::from_secs(30));
            if check_stop() {
                return Ok(());
            }

            if let Ok(source) = Decoder::new(BufReader::new(File::open(&anthem)?)) {
                sink.append(source);
            }
        }
        AudioPreset::BellSilenceBell => {
            if check_stop() {
                return Ok(());
            }
            let bell = get_audio_path("bell.ogg");

            if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell)?)) {
                sink.append(source);
            }

            drop(sink);
            if check_stop() {
                return Ok(());
            }
            thread::sleep(std::time::Duration::from_secs(60));
            if check_stop() {
                return Ok(());
            }

            let (_stream2, stream_handle2) = OutputStream::try_default()
                .map_err(|e| AppError::Audio(format!("Failed to open audio stream: {e}")))?;
            let sink2 = Sink::try_new(&stream_handle2)
                .map_err(|e| AppError::Audio(format!("Failed to create audio sink: {e}")))?;
            sink2.set_volume(volume_float);

            if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell)?)) {
                sink2.append(source);
            }
            sink2.sleep_until_end();
            return Ok(());
        }
        AudioPreset::BellMetronomeBell => {
            if check_stop() {
                return Ok(());
            }
            let bell = get_audio_path("bell.ogg");
            let metronome = get_audio_path("metronome.ogg");

            if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell)?)) {
                sink.append(source);
            }

            drop(sink);
            if check_stop() {
                return Ok(());
            }
            thread::sleep(std::time::Duration::from_secs(1));
            if check_stop() {
                return Ok(());
            }

            let (_stream2, stream_handle2) = OutputStream::try_default()
                .map_err(|e| AppError::Audio(format!("Failed to open audio stream: {e}")))?;
            let sink2 = Sink::try_new(&stream_handle2)
                .map_err(|e| AppError::Audio(format!("Failed to create audio sink: {e}")))?;
            sink2.set_volume(volume_float);

            if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                sink2.append(source);
            }

            if check_stop() {
                return Ok(());
            }
            thread::sleep(std::time::Duration::from_secs(58));
            if check_stop() {
                return Ok(());
            }

            let bell2 = get_audio_path("bell.ogg");
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell2)?)) {
                sink2.append(source);
            }
            sink2.sleep_until_end();
            return Ok(());
        }
    }

    sink.sleep_until_end();
    Ok(())
}
