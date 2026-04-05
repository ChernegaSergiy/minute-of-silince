//! Windows-specific platform integrations.
//!
//! Exposes two sub-modules:
//! * `media`  — pause / resume system-wide media playback.
//! * `volume` — control system volume.

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

pub mod media {
    use log::{error, info};
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    };

    use crate::error::{AppError, Result};

    pub fn pause_all() -> Result<()> {
        let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .map_err(|e| AppError::Platform(e.to_string()))?
            .get()
            .map_err(|e| AppError::Platform(e.to_string()))?;

        let sessions = manager
            .GetSessions()
            .map_err(|e| AppError::Platform(e.to_string()))?;

        let count = sessions
            .Size()
            .map_err(|e| AppError::Platform(e.to_string()))?;

        info!("Found {} media sessions", count);

        for i in 0..count {
            if let Ok(session) = sessions.GetAt(i) {
                let app_id = session.SourceAppUserModelId().unwrap_or_default();
                info!("Session {}: AppId={}", i, app_id);

                let playback_info = match session.GetPlaybackInfo() {
                    Ok(info) => info,
                    Err(e) => {
                        error!("Failed to get playback info: {:?}", e);
                        continue;
                    }
                };

                let status = match playback_info.PlaybackStatus() {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Failed to get playback status: {:?}", e);
                        continue;
                    }
                };

                info!("  Status: {:?}", status);

                if status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing {
                    info!("  Pausing session {}...", i);
                    match session.TryPauseAsync() {
                        Ok(op) => match op.get() {
                            Ok(true) => info!("  Successfully paused"),
                            Ok(false) => error!("  Pause rejected by app"),
                            Err(e) => error!("  Pause failed: {:?}", e),
                        },
                        Err(e) => error!("  TryPauseAsync failed: {:?}", e),
                    }
                }
            } else {
                error!("Failed to get session at index {}", i);
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
