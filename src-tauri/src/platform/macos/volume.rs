//! macOS volume control via Swift helper using native CoreAudio.

use crate::error::{AppError, Result};

unsafe extern "C" {
    fn macos_get_volume() -> u8;
    fn macos_set_volume(level: u8) -> bool;
    fn macos_is_muted() -> bool;
    fn macos_set_mute(mute: bool) -> bool;
}

pub fn get_volume() -> Result<u8> {
    Ok(unsafe { macos_get_volume() })
}

pub fn set_volume(level: u8) -> Result<()> {
    let success = unsafe { macos_set_volume(level) };
    if success {
        Ok(())
    } else {
        Err(AppError::Platform("Failed to set macOS volume".to_string()))
    }
}

pub fn is_muted() -> Result<bool> {
    Ok(unsafe { macos_is_muted() })
}

pub fn set_mute(mute: bool) -> Result<()> {
    let success = unsafe { macos_set_mute(mute) };
    if success {
        Ok(())
    } else {
        Err(AppError::Platform(
            "Failed to toggle macOS mute state".to_string(),
        ))
    }
}
