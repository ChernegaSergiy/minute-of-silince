use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use crate::core::settings::AudioPreset;
use crate::error::{AppError, Result};

static AUDIO_PLAYING: AtomicBool = AtomicBool::new(false);

pub fn is_playing() -> bool {
    AUDIO_PLAYING.load(Ordering::SeqCst)
}

pub fn stop() {
    AUDIO_PLAYING.store(false, Ordering::SeqCst);
}

fn get_audio_path(filename: &str) -> PathBuf {
    PathBuf::from("audio/").join(filename)
}

pub fn play_preset(preset: AudioPreset, volume: u8) -> Result<()> {
    AUDIO_PLAYING.store(true, Ordering::SeqCst);
    
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| AppError::Audio(format!("Failed to open audio stream: {e}")))?;
    
    let sink = Sink::try_new(&stream_handle)
        .map_err(|e| AppError::Audio(format!("Failed to create audio sink: {e}")))?;
    
    let volume_float = volume as f32 / 100.0;
    sink.set_volume(volume_float);
    
    match preset {
        AudioPreset::VoiceMetronome => {
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
            let metronome = get_audio_path("metronome.ogg");
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                sink.append(source);
            }
        }
        AudioPreset::VoiceSilenceBell => {
            let announcement = get_audio_path("announcement.ogg");
            let bell = get_audio_path("bell.ogg");
            
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                sink.append(source);
            }
            
            drop(sink);
            thread::sleep(std::time::Duration::from_secs(60));
            
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
            let announcement = get_audio_path("announcement.ogg");
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                sink.append(source);
            }
        }
        AudioPreset::VoiceMetronomeAnthem => {
            let announcement = get_audio_path("announcement.ogg");
            let metronome = get_audio_path("metronome.ogg");
            let anthem = get_audio_path("anthem.ogg");
            
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&announcement)?)) {
                sink.append(source);
            }
            
            drop(sink);
            thread::sleep(std::time::Duration::from_secs(1));
            
            let (_stream2, stream_handle2) = OutputStream::try_default()
                .map_err(|e| AppError::Audio(format!("Failed to open audio stream: {e}")))?;
            let sink2 = Sink::try_new(&stream_handle2)
                .map_err(|e| AppError::Audio(format!("Failed to create audio sink: {e}")))?;
            sink2.set_volume(volume_float);
            
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                sink2.append(source);
            }
            
            thread::sleep(std::time::Duration::from_secs(30));
            
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&anthem)?)) {
                sink2.append(source);
            }
            sink2.sleep_until_end();
            return Ok(());
        }
        AudioPreset::MetronomeAnthem => {
            let metronome = get_audio_path("metronome.ogg");
            let anthem = get_audio_path("anthem.ogg");
            
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                sink.append(source);
            }
            
            thread::sleep(std::time::Duration::from_secs(30));
            
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&anthem)?)) {
                sink.append(source);
            }
        }
        AudioPreset::BellSilenceBell => {
            let bell = get_audio_path("bell.ogg");
            
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell)?)) {
                sink.append(source);
            }
            
            drop(sink);
            thread::sleep(std::time::Duration::from_secs(60));
            
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
            let bell = get_audio_path("bell.ogg");
            let metronome = get_audio_path("metronome.ogg");
            
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell)?)) {
                sink.append(source);
            }
            
            drop(sink);
            thread::sleep(std::time::Duration::from_secs(1));
            
            let (_stream2, stream_handle2) = OutputStream::try_default()
                .map_err(|e| AppError::Audio(format!("Failed to open audio stream: {e}")))?;
            let sink2 = Sink::try_new(&stream_handle2)
                .map_err(|e| AppError::Audio(format!("Failed to create audio sink: {e}")))?;
            sink2.set_volume(volume_float);
            
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&metronome)?)) {
                sink2.append(source);
            }
            
            thread::sleep(std::time::Duration::from_secs(58));
            
            let bell2 = get_audio_path("bell.ogg");
            if let Ok(source) = Decoder::new(BufReader::new(File::open(&bell2)?)) {
                sink2.append(source);
            }
            sink2.sleep_until_end();
            return Ok(());
        }
    }
    
    sink.sleep_until_end();
    AUDIO_PLAYING.store(false, Ordering::SeqCst);
    Ok(())
}
