//! Windows volume control via Windows Core Audio APIs.

use crate::error::{AppError, Result};
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{IMMDeviceEnumerator, MMDeviceEnumerator, eConsole, eRender};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};

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
