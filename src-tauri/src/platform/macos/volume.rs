//! macOS volume control via osascript.

use crate::error::{AppError, Result};
use std::process::Command;

pub fn get_volume() -> Result<u8> {
    let output = Command::new("osascript")
        .args(["-e", "output volume of (get volume settings)"])
        .output()
        .map_err(|e| AppError::Platform(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .trim()
        .parse::<u8>()
        .map_err(|e| AppError::Platform(e.to_string()))
}

pub fn set_volume(level: u8) -> Result<()> {
    let clamped = level.min(100);
    Command::new("osascript")
        .args(["-e", &format!("set volume output volume {}", clamped)])
        .output()
        .map_err(|e| AppError::Platform(e.to_string()))?;
    Ok(())
}

pub fn is_muted() -> Result<bool> {
    let output = Command::new("osascript")
        .args(["-e", "output muted of (get volume settings)"])
        .output()
        .map_err(|e| AppError::Platform(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim() == "true")
}

pub fn set_mute(mute: bool) -> Result<()> {
    let val = if mute { "true" } else { "false" };
    Command::new("osascript")
        .args(["-e", &format!("set volume output muted {}", val)])
        .output()
        .map_err(|e| AppError::Platform(e.to_string()))?;
    Ok(())
}
