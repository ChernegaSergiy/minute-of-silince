//! Windows-specific platform integrations.
//!
//! Exposes three sub-modules:
//! * `media`  — pause / resume system-wide media playback.
//! * `volume` — control system volume.
//! * `power`  — register for power-broadcast events (sleep / wake).

pub mod volume {
    use crate::error::{AppError, Result};
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    use windows::Win32::Media::Audio::{MMDeviceEnumerator, IMMDeviceEnumerator};
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};

    pub fn get_volume() -> Result<u8> {
        unsafe {
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(
                &MMDeviceEnumerator,
                None,
                CLSCTX_INPROC_SERVER,
            )
            .map_err(AppError::Platform)?;

            let device = enumerator
                .GetDefaultAudioEndpoint(windows::Win32::Media::Audio::Render, windows::Win32::Media::Audio::Communications)
                .map_err(AppError::Platform)?;

            let endpoint: IAudioEndpointVolume = device
                .Activate(CLSCTX_INPROC_SERVER, None)
                .map_err(AppError::Platform)?;

            let volume = endpoint
                .GetMasterVolumeLevelScalar()
                .map_err(AppError::Platform)?;
            Ok((volume * 100.0) as u8)
        }
    }

    pub fn set_volume(level: u8) -> Result<()> {
        unsafe {
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(
                &MMDeviceEnumerator,
                None,
                CLSCTX_INPROC_SERVER,
            )
            .map_err(AppError::Platform)?;

            let device = enumerator
                .GetDefaultAudioEndpoint(windows::Win32::Media::Audio::Render, windows::Win32::Media::Audio::Communications)
                .map_err(AppError::Platform)?;

            let endpoint: IAudioEndpointVolume = device
                .Activate(CLSCTX_INPROC_SERVER, None)
                .map_err(AppError::Platform)?;

            let clamped = (level as f32 / 100.0).min(1.0).max(0.0);
            endpoint
                .SetMasterVolumeLevelScalar(clamped, std::ptr::null())
                .map_err(AppError::Platform)?;

            Ok(())
        }
    }
}

pub mod media {
    //! Pause and resume other media players using the Windows multimedia API.
    //!
    //! Strategy (in order of preference):
    //! 1. Send `VK_MEDIA_PLAY_PAUSE` via `SendInput` — works for most apps.
    //! 2. Mute individual audio sessions via `IAudioSessionControl` — used as
    //!    a complement when step 1 is insufficient.

    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, VK_MEDIA_PLAY_PAUSE,
    };

    use crate::error::{AppError, Result};

    /// Send a synthetic `MEDIA_PLAY_PAUSE` key press to the system.
    ///
    /// This pauses Spotify, browser video, VLC, etc. without needing
    /// per-process control.
    pub fn pause_all() -> Result<()> {
        send_media_key()
    }

    /// Send a second `MEDIA_PLAY_PAUSE` to resume playback.
    pub fn resume_all() -> Result<()> {
        send_media_key()
    }

    fn send_media_key() -> Result<()> {
        // Key-down event.
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

        // Key-up event.
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
    //!
    //! NOTE: Full message-loop integration is wired via the Tauri window
    //! `WndProc` callback; this module exposes the handler logic only.

    use windows::Win32::UI::WindowsAndMessaging::{PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND};

    /// Returns `true` when `wparam` signals a resume-from-sleep event.
    pub fn is_resume_event(wparam: usize) -> bool {
        wparam == PBT_APMRESUMESUSPEND as usize || wparam == PBT_APMRESUMEAUTOMATIC as usize
    }
}
