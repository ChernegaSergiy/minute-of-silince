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
    use windows::core::{GUID, HRESULT, PCWSTR};
    use windows::Win32::Media::Audio::{
        eCommunications, eConsole, eMultimedia, eRender, ERole, IMMDeviceEnumerator,
        MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
    };
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};

    // Undocumented IPolicyConfig interface GUID
    const IPOLICYCONFIG_GUID: GUID = GUID::from_u128(0x870af99c_171d_4f15_af0d_e63df40c2bc9);

    #[repr(C)]
    struct IPolicyConfigVtbl {
        pub base: [usize; 10], // Skip first 10 methods (QueryInterface, AddRef, Release, etc.)
        pub set_default_endpoint:
            unsafe extern "system" fn(this: *mut usize, device_id: PCWSTR, role: ERole) -> HRESULT,
    }

    #[repr(C)]
    struct IPolicyConfig {
        pub vtbl: *const IPolicyConfigVtbl,
    }
    /// Force the system to use built-in speakers if available.
    /// Returns the ID of the previously default device so it can be restored.
    pub fn force_speakers() -> Result<Option<String>> {
        log::info!("Attempting to force audio to speakers via IPolicyConfig...");
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER)
                    .map_err(|e| AppError::Platform(format!("Enumerator failed: {e}")))?;

            // 1. Get current default device ID to return it for later restoration
            let previous_default = enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .ok()
                .and_then(|d| d.GetId().ok())
                .and_then(|id| id.to_string().ok());

            // 2. Find speakers
            let collection = enumerator
                .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
                .map_err(|e| AppError::Platform(format!("Enum endpoints failed: {e}")))?;

            let count = collection
                .GetCount()
                .map_err(|e| AppError::Platform(format!("GetCount failed: {e}")))?;
            let mut speaker_id: Option<String> = None;

            for i in 0..count {
                let device = collection
                    .Item(i)
                    .map_err(|e| AppError::Platform(format!("Item failed: {e}")))?;
                let id = device
                    .GetId()
                    .map_err(|e| AppError::Platform(format!("GetId failed: {e}")))?;
                let id_str = id
                    .to_string()
                    .map_err(|e| AppError::Platform(e.to_string()))?;

                if id_str.to_lowercase().contains("speaker")
                    || id_str.to_lowercase().contains("internal")
                {
                    speaker_id = Some(id_str);
                    break;
                }
            }

            // 3. Switch to speakers if found and different from current
            if let Some(id) = speaker_id {
                if Some(id.clone()) == previous_default {
                    log::info!("Speakers are already the default device");
                    return Ok(None); // No need to restore if nothing changed
                }

                log::info!("Found speakers: {}. Activating...", id);
                set_default_device_api(&id)?;
                log::info!("Audio output successfully redirected to speakers");
                Ok(previous_default)
            } else {
                log::warn!("No speakers found among active audio endpoints");
                Ok(None)
            }
        }
    }

    /// Restore audio output to a specific device by its ID.
    pub fn restore_output(device_id: &str) -> Result<()> {
        log::info!("Restoring audio output to previous device: {}", device_id);
        set_default_device_api(device_id)
    }

    /// Internal helper to call IPolicyConfig
    fn set_default_device_api(id: &str) -> Result<()> {
        unsafe {
            // We use IUnknown as a generic COM interface to satisfy trait bounds of CoCreateInstance
            let unknown: windows::core::IUnknown =
                CoCreateInstance(&IPOLICYCONFIG_GUID, None, CLSCTX_INPROC_SERVER).map_err(|e| {
                    AppError::Platform(format!("IPolicyConfig creation failed: {e}"))
                })?;

            // Cast the raw pointer to our manual IPolicyConfig structure
            let policy_config = unknown.as_raw() as *mut IPolicyConfig;

            let id_u16: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
            let pcwstr = PCWSTR(id_u16.as_ptr());

            ((*(*policy_config).vtbl).set_default_endpoint)(
                policy_config as *mut usize,
                pcwstr,
                eConsole,
            );
            ((*(*policy_config).vtbl).set_default_endpoint)(
                policy_config as *mut usize,
                pcwstr,
                eMultimedia,
            );
            ((*(*policy_config).vtbl).set_default_endpoint)(
                policy_config as *mut usize,
                pcwstr,
                eCommunications,
            );
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
