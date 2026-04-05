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
    use windows::Win32::Foundation::PROPERTYKEY;
    use windows::Win32::Media::Audio::{
        eCommunications, eConsole, eMultimedia, eRender, ERole, IMMDeviceEnumerator,
        MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
    };
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER, STGM_READ};

    // EndpointFormFactor constants (mmdeviceapi.h)
    // RemoteNetworkDevice      = 0
    // Speakers                 = 1  <-- що нам потрібно
    // LineLevel                = 2
    // Headphones               = 3  <-- виключаємо
    // Microphone               = 4
    // Headset                  = 5
    // Handset                  = 6
    // UnknownDigitalPassthrough= 7
    // SPDIF                    = 8  <-- виключаємо (цифровий)
    // DigitalAudioDisplayDevice= 9  <-- виключаємо (HDMI/DisplayPort)
    // UnknownFormFactor        = 10
    const FORM_FACTOR_SPEAKERS: u32 = 1;
    const FORM_FACTOR_HEADPHONES: u32 = 3;
    const FORM_FACTOR_SPDIF: u32 = 8;
    const FORM_FACTOR_DIGITAL_DISPLAY: u32 = 9; // HDMI / DisplayPort

    // Undocumented IPolicyConfig interface GUID
    const IPOLICYCONFIG_GUID: GUID = GUID::from_u128(0x870af99c_171d_4f15_af0d_e63df40c2bc9);

    #[repr(C)]
    struct IPolicyConfigVtbl {
        pub base: [usize; 10], // QueryInterface, AddRef, Release + reserved methods
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

            // 2. Enumerate active render endpoints and find speakers
            let collection = enumerator
                .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
                .map_err(|e| AppError::Platform(format!("Enum endpoints failed: {e}")))?;

            let count = collection
                .GetCount()
                .map_err(|e| AppError::Platform(format!("GetCount failed: {e}")))?;

            let mut speaker_id: Option<String> = None;

            // PKEY_AudioEndpoint_FormFactor: {1DA5D803-D492-4EDD-8C23-E0C0FFEE7F0E}, pid=0
            // Value type is VT_UINT (u32), read via .ulVal — NOT .uiVal (u16)
            let form_factor_pkey = PROPERTYKEY {
                fmtid: windows::core::GUID::from_u128(0x1da5d803_d492_4edd_8c23_e0c0ffee7f0e),
                pid: 0,
            };

            for i in 0..count {
                let device = collection
                    .Item(i)
                    .map_err(|e| AppError::Platform(format!("Item failed: {e}")))?;

                let id_str = device
                    .GetId()
                    .map_err(|e| AppError::Platform(format!("GetId failed: {e}")))?
                    .to_string()
                    .map_err(|e| AppError::Platform(e.to_string()))?;

                // Read FormFactor as VT_UINT (u32) using .ulVal
                let form_factor: u32 = device
                    .OpenPropertyStore(STGM_READ)
                    .ok()
                    .and_then(|props| props.GetValue(&form_factor_pkey).ok())
                    .map(|v| v.Anonymous.Anonymous.Anonymous.ulVal)
                    .unwrap_or(u32::MAX);

                log::info!("Device ID: {} | FormFactor: {}", id_str, form_factor);

                // Accept only Speakers (1).
                // Reject headphones (3), SPDIF (8), and HDMI/DisplayPort (9).
                let is_speakers = form_factor == FORM_FACTOR_SPEAKERS;
                let is_excluded = form_factor == FORM_FACTOR_HEADPHONES
                    || form_factor == FORM_FACTOR_SPDIF
                    || form_factor == FORM_FACTOR_DIGITAL_DISPLAY;

                if is_speakers && !is_excluded {
                    speaker_id = Some(id_str);
                    log::info!("Speaker found (FormFactor={})", form_factor);
                    break;
                }
            }

            // 3. Switch to speakers if found and different from current default
            if let Some(id) = speaker_id {
                if Some(id.clone()) == previous_default {
                    log::info!("Speakers are already the default device");
                    return Ok(None); // Nothing changed — nothing to restore
                }

                log::info!("Switching default audio output to: {}", id);
                set_default_device_api(&id)?;
                log::info!("Audio output successfully redirected to speakers");
                Ok(previous_default)
            } else {
                log::warn!("No suitable speaker endpoint found among active audio devices");
                Ok(None)
            }
        }
    }

    /// Restore audio output to a specific device by its ID.
    pub fn restore_output(device_id: &str) -> Result<()> {
        log::info!("Restoring audio output to previous device: {}", device_id);
        set_default_device_api(device_id)
    }

    /// Internal helper to call IPolicyConfig::SetDefaultEndpoint for all three roles.
    fn set_default_device_api(id: &str) -> Result<()> {
        unsafe {
            let unknown: windows::core::IUnknown =
                CoCreateInstance(&IPOLICYCONFIG_GUID, None, CLSCTX_INPROC_SERVER).map_err(|e| {
                    AppError::Platform(format!("IPolicyConfig creation failed: {e}"))
                })?;

            let policy_config = unknown.as_raw() as *mut IPolicyConfig;

            let id_u16: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
            let pcwstr = PCWSTR(id_u16.as_ptr());

            let _ = ((*(*policy_config).vtbl).set_default_endpoint)(
                policy_config as *mut usize,
                pcwstr,
                eConsole,
            );
            let _ = ((*(*policy_config).vtbl).set_default_endpoint)(
                policy_config as *mut usize,
                pcwstr,
                eMultimedia,
            );
            let _ = ((*(*policy_config).vtbl).set_default_endpoint)(
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
    //! Strategy: Send `VK_MEDIA_PLAY_PAUSE` via `SendInput` — works for most apps
    //! (Spotify, browser video, VLC, etc.) without needing per-process control.

    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, VK_MEDIA_PLAY_PAUSE,
    };

    use crate::error::{AppError, Result};

    pub fn pause_all() -> Result<()> {
        send_media_key()
    }

    pub fn resume_all() -> Result<()> {
        send_media_key()
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

/// Temporary diagnostic command — paste into src-tauri/src/commands.rs
/// and add to invoke_handler in lib.rs as `commands::list_audio_devices`
/// Remove after diagnosing.

#[tauri::command]
pub fn list_audio_devices() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::PROPERTYKEY;
        use windows::Win32::Media::Audio::{
            eRender, IMMDeviceEnumerator, MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
        };
        use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER, STGM_READ};

        let form_factor_pkey = PROPERTYKEY {
            fmtid: windows::core::GUID::from_u128(0x1da5d803_d492_4edd_8c23_e0c0ffee7f0e),
            pid: 0,
        };

        // PKEY_Device_FriendlyName: {A45C254E-DF1C-4EFD-8020-67D146A850E0}, pid=14
        let friendly_name_pkey = PROPERTYKEY {
            fmtid: windows::core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
            pid: 14,
        };

        unsafe {
            let Ok(enumerator) = CoCreateInstance::<_, IMMDeviceEnumerator>(
                &MMDeviceEnumerator,
                None,
                CLSCTX_INPROC_SERVER,
            ) else {
                return vec!["ERROR: CoCreateInstance failed".to_string()];
            };

            let Ok(collection) = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            else {
                return vec!["ERROR: EnumAudioEndpoints failed".to_string()];
            };

            let Ok(count) = collection.GetCount() else {
                return vec!["ERROR: GetCount failed".to_string()];
            };

            let mut results = Vec::new();

            for i in 0..count {
                let Ok(device) = collection.Item(i) else {
                    continue;
                };

                let id = device
                    .GetId()
                    .ok()
                    .and_then(|id| id.to_string().ok())
                    .unwrap_or_else(|| "<no id>".to_string());

                let props = device.OpenPropertyStore(STGM_READ).ok();

                let form_factor: u32 = props
                    .as_ref()
                    .and_then(|p| p.GetValue(&form_factor_pkey).ok())
                    .map(|v| v.Anonymous.Anonymous.Anonymous.ulVal)
                    .unwrap_or(999);

                let friendly_name: String = props
                    .as_ref()
                    .and_then(|p| p.GetValue(&friendly_name_pkey).ok())
                    .and_then(|v| {
                        // VT_LPWSTR = 31
                        let vt = v.Anonymous.Anonymous.vt.0;
                        if vt == 31 {
                            let ptr = v.Anonymous.Anonymous.Anonymous.pwszVal;
                            if !ptr.is_null() {
                                return ptr.to_string().ok();
                            }
                        }
                        None
                    })
                    .unwrap_or_else(|| "<no name>".to_string());

                results.push(format!(
                    "FormFactor={} | Name={} | ID={}",
                    form_factor, friendly_name, id
                ));
            }

            if results.is_empty() {
                results.push("No active render devices found".to_string());
            }

            results
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        vec!["Not supported on this platform".to_string()]
    }
}
