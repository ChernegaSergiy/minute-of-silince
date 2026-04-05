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
    //! Pause and resume other media players using the Windows multimedia API.
    //!
    //! Strategy: Check if any audio is actually playing before pausing. Only send
    //! the media toggle key if something was playing — this prevents accidentally
    //! unpausing media that was already paused, and avoids interfering if the user
    //! manually unpaused during the ceremony.

    use std::sync::Mutex;
    use windows::Win32::Media::Audio::{
        eConsole, eRender, AudioSessionStateActive, IAudioSessionControl, IAudioSessionManager2,
        IMMDeviceEnumerator, MMDeviceEnumerator,
    };
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, VK_MEDIA_PLAY_PAUSE,
    };

    use crate::error::{AppError, Result};

    lazy_static::lazy_static! {
        static ref WAS_PLAYING: Mutex<bool> = Mutex::new(false);
    }

    pub fn pause_all() -> Result<()> {
        let is_playing = is_anything_playing()?;
        *WAS_PLAYING.lock().unwrap() = is_playing;

        if is_playing {
            send_media_key()?;
        }
        Ok(())
    }

    pub fn resume_all() -> Result<()> {
        let was_playing = *WAS_PLAYING.lock().unwrap();
        if was_playing {
            send_media_key()?;
        }
        *WAS_PLAYING.lock().unwrap() = false;
        Ok(())
    }

    fn is_anything_playing() -> Result<bool> {
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER)
                    .map_err(|e| AppError::Platform(e.to_string()))?;

            let device = enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|e| AppError::Platform(e.to_string()))?;

            let session_manager: IAudioSessionManager2 = device
                .Activate(CLSCTX_INPROC_SERVER, None)
                .map_err(|e| AppError::Platform(e.to_string()))?;

            let session_enumerator = session_manager
                .GetSessionEnumerator()
                .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?;

            let count = session_enumerator
                .GetCount()
                .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?;

            for i in 0..count {
                let session: IAudioSessionControl = session_enumerator
                    .GetSession(i)
                    .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?;

                let state = session
                    .GetState()
                    .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?;

                if state == AudioSessionStateActive {
                    return Ok(true);
                }
            }

            Ok(false)
        }
    }

    fn send_media_key() -> Result<()> {
        let key_down = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: windows::Win32::UI::Input::KeyboardAndMouse::KEYBDINPUT {
                    wVk: VK_MEDIA_PLAY_PAUSE,
                    wScan: 0,
                    dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let key_up = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: windows::Win32::UI::Input::KeyboardAndMouse::KEYBDINPUT {
                    wVk: VK_MEDIA_PLAY_PAUSE,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let inputs = [key_down, key_up];
        let result = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };

        if result == 0 {
            Err(AppError::Platform(
                "SendInput failed: no inputs were sent".into(),
            ))
        } else {
            Ok(())
        }
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
