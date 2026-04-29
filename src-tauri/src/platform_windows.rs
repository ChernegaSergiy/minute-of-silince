//! Windows-specific platform integrations.
//!
//! Exposes three sub-modules:
//! * `media`      — pause / resume system-wide media playback.
//! * `volume`     — control system (endpoint) volume via `IAudioEndpointVolume`.
//! * `app_volume` — per-application volume control via WASAPI `ISimpleAudioVolume`.

pub mod volume {
    use crate::error::{AppError, Result};
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    use windows::Win32::Media::Audio::{
        eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
    };
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};

    pub fn get_volume() -> Result<u8> {
        unsafe {
            let endpoint = get_endpoint()?;
            let volume = endpoint
                .GetMasterVolumeLevelScalar()
                .map_err(|e| AppError::Platform(e.to_string()))?;
            Ok((volume * 100.0) as u8)
        }
    }

    pub fn set_volume(level: u8) -> Result<()> {
        unsafe {
            let endpoint = get_endpoint()?;
            let clamped = (level as f32 / 100.0).clamp(0.0, 1.0);
            endpoint
                .SetMasterVolumeLevelScalar(clamped, std::ptr::null())
                .map_err(|e| AppError::Platform(e.to_string()))?;

            Ok(())
        }
    }

    pub fn is_muted() -> Result<bool> {
        unsafe {
            let endpoint = get_endpoint()?;
            let muted = endpoint
                .GetMute()
                .map_err(|e| AppError::Platform(e.to_string()))?;
            Ok(muted.as_bool())
        }
    }

    pub fn set_mute(mute: bool) -> Result<()> {
        unsafe {
            let endpoint = get_endpoint()?;
            endpoint
                .SetMute(mute, std::ptr::null())
                .map_err(|e| AppError::Platform(e.to_string()))?;
            Ok(())
        }
    }

    fn get_endpoint() -> Result<IAudioEndpointVolume> {
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER)
                    .map_err(|e| AppError::Platform(e.to_string()))?;

            let device = enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|e| AppError::Platform(e.to_string()))?;

            let endpoint: IAudioEndpointVolume = device
                .Activate(CLSCTX_INPROC_SERVER, None)
                .map_err(|e| AppError::Platform(e.to_string()))?;

            Ok(endpoint)
        }
    }
}

/// Per-application volume control via WASAPI `ISimpleAudioVolume`.
///
/// Unlike [`volume`] which changes the system-wide endpoint volume (affecting
/// every application on the system), this module controls only the volume for
/// **our process's audio session**.  The result is visible in the Windows
/// Volume Mixer as a per-app level and leaves the system master volume and
/// other applications completely untouched.
pub mod app_volume {
    use crate::error::{AppError, Result};
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::Media::Audio::{
        eConsole, eRender, IAudioSessionManager, IMMDeviceEnumerator, ISimpleAudioVolume,
        MMDeviceEnumerator,
    };
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};

    /// Obtain the `ISimpleAudioVolume` interface for the current process's
    /// default per-process audio session.
    fn get_simple_audio_volume() -> Result<ISimpleAudioVolume> {
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER)
                    .map_err(|e| AppError::Platform(e.to_string()))?;

            let device = enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|e| AppError::Platform(e.to_string()))?;

            // Activate IAudioSessionManager (not IAudioSessionManager2) —
            // GetSimpleAudioVolume lives on the v1 interface.
            let session_manager: IAudioSessionManager = device
                .Activate(CLSCTX_INPROC_SERVER, None)
                .map_err(|e| AppError::Platform(e.to_string()))?;

            // NULL GUID → default audio session for this process.
            // BOOL(0)   → per-process session (not cross-process).
            let volume: ISimpleAudioVolume = session_manager
                .GetSimpleAudioVolume(std::ptr::null(), BOOL(0))
                .map_err(|e| AppError::Platform(e.to_string()))?;

            Ok(volume)
        }
    }

    /// Return the current per-application session volume as a percentage (0–100).
    pub fn get_volume() -> Result<u8> {
        unsafe {
            let volume = get_simple_audio_volume()?;
            let level = volume
                .GetMasterVolume()
                .map_err(|e| AppError::Platform(e.to_string()))?;
            Ok((level * 100.0) as u8)
        }
    }

    /// Set the per-application session volume (`level` in the range 0–100).
    ///
    /// This adjusts only our app's contribution to the audio mix and has no
    /// effect on the system master volume or any other application.
    pub fn set_volume(level: u8) -> Result<()> {
        unsafe {
            let volume = get_simple_audio_volume()?;
            let clamped = (level as f32 / 100.0).clamp(0.0, 1.0);
            volume
                .SetMasterVolume(clamped, std::ptr::null())
                .map_err(|e| AppError::Platform(e.to_string()))?;
            Ok(())
        }
    }

    /// Return `true` if the current process's audio session is muted.
    pub fn is_muted() -> Result<bool> {
        unsafe {
            let volume = get_simple_audio_volume()?;
            let muted = volume
                .GetMute()
                .map_err(|e| AppError::Platform(e.to_string()))?;
            Ok(muted.as_bool())
        }
    }

    /// Mute or unmute the current process's audio session.
    pub fn set_mute(mute: bool) -> Result<()> {
        unsafe {
            let volume = get_simple_audio_volume()?;
            volume
                .SetMute(BOOL::from(mute), std::ptr::null())
                .map_err(|e| AppError::Platform(e.to_string()))?;
            Ok(())
        }
    }
}

pub mod media {
    use log::{error, info};
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackInfo,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    };

    use crate::error::{AppError, Result};

    pub async fn pause_all() -> Result<()> {
        let manager: GlobalSystemMediaTransportControlsSessionManager =
            GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?
                .await
                .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?;

        let sessions = manager
            .GetSessions()
            .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?;

        let count = sessions
            .Size()
            .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?;

        info!("Found {} media sessions", count);

        for i in 0..count {
            if let Ok(session) = sessions.GetAt(i) {
                let app_id: windows::core::HSTRING =
                    session.SourceAppUserModelId().unwrap_or_default();
                info!("Session {}: AppId={}", i, app_id);

                let playback_info: GlobalSystemMediaTransportControlsSessionPlaybackInfo =
                    match session.GetPlaybackInfo() {
                        Ok(info) => info,
                        Err(e) => {
                            error!("Failed to get playback info for session {}: {:?}", i, e);
                            continue;
                        }
                    };

                let status = match playback_info.PlaybackStatus() {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Failed to get playback status for session {}: {:?}", i, e);
                        continue;
                    }
                };

                if status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing {
                    info!("Pausing session {}...", i);
                    let _ = session.TryPauseAsync();
                }
            }
        }

        Ok(())
    }
}

pub mod power {
    //! Listen for `WM_POWERBROADCAST` events so the scheduler can detect
    //! whether the PC woke from sleep after 09:00.

    use windows::Win32::UI::WindowsAndMessaging::{PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND};

    #[allow(dead_code)]
    pub fn is_resume_event(wparam: usize) -> bool {
        wparam == PBT_APMRESUMESUSPEND as usize || wparam == PBT_APMRESUMEAUTOMATIC as usize
    }
}
