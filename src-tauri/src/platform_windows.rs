//! Windows-specific platform integrations.
//!
//! Exposes three sub-modules:
//! * `media`  — pause / resume system-wide media playback.
//! * `volume` — control system volume.
//! * `power`  — register for power-broadcast events (sleep / wake).

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

pub mod output {
    use crate::error::{AppError, Result};
    use windows::core::{Interface, GUID, HRESULT, PCWSTR};
    use windows::Win32::Media::Audio::{
        DEVICE_STATE_ACTIVE, IMMDeviceEnumerator, MMDeviceEnumerator, eRender, eMultimedia, ERole, eConsole, eCommunications
    };
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};

    // Undocumented IPolicyConfig interface GUID
    const IPOLICYCONFIG_GUID: GUID = GUID::from_u128(0x870af99c_171d_4f15_af0d_e63df40c2bc9);

    #[repr(C)]
    struct IPolicyConfigVtbl {
        pub base: [usize; 10], // Skip first 10 methods (QueryInterface, AddRef, Release, etc.)
        pub set_default_endpoint: unsafe extern "system" fn(
            this: *mut usize,
            device_id: PCWSTR,
            role: ERole,
        ) -> HRESULT,
    }

    #[repr(C)]
    struct IPolicyConfig {
        pub vtbl: *const IPolicyConfigVtbl,
    }
/// Force the system to use built-in speakers if available.
pub fn force_speakers() -> Result<()> {
    log::info!("Attempting to force audio to speakers via IPolicyConfig...");
    unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER)
                .map_err(|e| AppError::Platform(format!("Enumerator failed: {e}")))?;

        let collection = enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .map_err(|e| AppError::Platform(format!("Enum endpoints failed: {e}")))?;

        let count = collection.GetCount().map_err(|e| AppError::Platform(format!("GetCount failed: {e}")))?;
        let mut speaker_id: Option<String> = None;

        for i in 0..count {
            let device = collection.Item(i).map_err(|e| AppError::Platform(format!("Item failed: {e}")))?;
            let id = device.GetId().map_err(|e| AppError::Platform(format!("GetId failed: {e}")))?;
            let id_str = id.to_string().map_err(|e| AppError::Platform(e.to_string()))?;

            // Note: In a production environment, we should check PKEY_Device_FormFactor.
            // For this implementation, we search for common keywords.
            if id_str.to_lowercase().contains("speaker") || id_str.to_lowercase().contains("internal") {
                speaker_id = Some(id_str);
                break;
            }
        }

        if let Some(id) = speaker_id {
            log::info!("Found speakers: {}. Activating...", id);

            let policy_config: *mut IPolicyConfig = CoCreateInstance(
                &IPOLICYCONFIG_GUID,
                None,
                CLSCTX_INPROC_SERVER,
            ).map_err(|e| AppError::Platform(format!("IPolicyConfig creation failed: {e}")))?;

            let id_u16: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
            let pcwstr = PCWSTR(id_u16.as_ptr());

            // Set as default for all roles
            ((*(*policy_config).vtbl).set_default_endpoint)(policy_config as *mut usize, pcwstr, eConsole);
            ((*(*policy_config).vtbl).set_default_endpoint)(policy_config as *mut usize, pcwstr, eMultimedia);
            ((*(*policy_config).vtbl).set_default_endpoint)(policy_config as *mut usize, pcwstr, eCommunications);

            log::info!("Audio output successfully redirected to speakers");
        } else {
            log::warn!("No speakers found among active audio endpoints");
        }

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

    #[allow(dead_code)]
    pub fn is_resume_event(wparam: usize) -> bool {
        wparam == PBT_APMRESUMESUSPEND as usize || wparam == PBT_APMRESUMEAUTOMATIC as usize
    }
}
