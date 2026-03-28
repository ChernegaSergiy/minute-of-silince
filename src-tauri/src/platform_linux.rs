//! Linux platform integrations.
//!
//! On Linux we use the MPRIS D-Bus interface to pause/resume media players.

use std::collections::HashSet;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref PAUSED_PLAYERS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

pub mod volume {
    use crate::error::{AppError, Result};
    use std::process::Command;

    pub fn get_volume() -> Result<u8> {
        let output = Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
            .map_err(|e| AppError::Io(e))?;

        if !output.status.success() {
            return get_volume_amixer();
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let volume_str = stdout.trim();
        
        if volume_str.contains("MUTE") {
            return Ok(0);
        }

        let parts: Vec<&str> = volume_str.split_whitespace().collect();
        if parts.is_empty() {
            return Err(AppError::Platform("Failed to parse volume".into()));
        }

        let vol_str = parts[0].trim_end_matches('.');
        let vol_str = vol_str.trim_start_matches("0.");
        let vol_str = vol_str.trim_start_matches("Volume: ");
        
        let vol: f64 = vol_str.parse().unwrap_or(50.0);
        Ok(vol as u8)
    }

    fn get_volume_amixer() -> Result<u8> {
        let output = Command::new("amixer")
            .args(["get", "Master"])
            .output()
            .map_err(|e| AppError::Io(e))?;

        if !output.status.success() {
            return Err(AppError::Platform("Failed to get volume".into()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines() {
            if line.contains("%[") || line.contains("Playback") {
                if let Some(pos) = line.find('[') {
                    let rest = &line[pos..];
                    if let Some(end) = rest.find('%') {
                        let vol_str = &rest[1..end];
                        if let Ok(vol) = vol_str.parse::<u8>() {
                            return Ok(vol);
                        }
                    }
                }
            }
        }

        Err(AppError::Platform("Could not parse volume".into()))
    }

    pub fn set_volume(level: u8) -> Result<()> {
        let clamped = level.min(100);
        
        let result = Command::new("wpctl")
            .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{}%", clamped)])
            .status();

        if result.is_ok() && result.unwrap().success() {
            return Ok(());
        }

        set_volume_amixer(clamped)
    }

    fn set_volume_amixer(level: u8) -> Result<()> {
        let output = Command::new("amixer")
            .args(["set", "Master", &format!("{}%", level)])
            .output()
            .map_err(|e| AppError::Io(e))?;

        if !output.status.success() {
            return Err(AppError::Platform("Failed to set volume".into()));
        }

        Ok(())
    }
}

pub mod media {
    use super::*;
    use crate::error::{AppError, Result};
    use std::process::Command;

    pub fn pause_all() -> Result<()> {
        pause_all_via_mpris().or_else(|_| pause_all_via_xdotool())
    }

    pub fn resume_all() -> Result<()> {
        resume_all_via_mpris().or_else(|_| resume_all_via_xdotool())
    }

    fn get_mpris_players() -> Result<Vec<String>> {
        let output = Command::new("dbus-send")
            .args([
                "--print-reply",
                "--dest=org.freedesktop.DBus",
                "/org/freedesktop/DBus",
                "org.freedesktop.DBus.ListNames",
            ])
            .output()
            .map_err(|e| AppError::Io(e))?;

        if !output.status.success() {
            return Err(AppError::Platform("Failed to list D-Bus names".into()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let players: Vec<String> = stdout
            .lines()
            .filter(|line| line.contains("org.mpris.MediaPlayer2"))
            .filter_map(|line| {
                line.split('"')
                    .nth(1)
                    .filter(|s| !s.contains("org.freedesktop.DBus"))
                    .map(String::from)
            })
            .collect();

        Ok(players)
    }

    fn get_playback_status(player: &str) -> Option<String> {
        let output = Command::new("dbus-send")
            .args([
                "--print-reply",
                &format!("--dest={}", player),
                "/org/mpris/MediaPlayer2",
                "org.freedesktop.DBus.Properties.Get",
                "string:org.mpris.MediaPlayer2.Player",
                "string:PlaybackStatus",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.lines()
            .find(|line| line.contains("Playing") || line.contains("Paused"))
            .map(|line| {
                if line.contains("Playing") {
                    "Playing".to_string()
                } else {
                    "Paused".to_string()
                }
            })
    }

    fn pause_all_via_mpris() -> Result<()> {
        let players = get_mpris_players()?;

        if players.is_empty() {
            log::info!("No MPRIS media players found");
            return Err(AppError::Platform("No MPRIS players found".into()));
        }

        let mut paused = PAUSED_PLAYERS.lock().unwrap();
        paused.clear();

        for player in &players {
            if let Some(status) = get_playback_status(player) {
                if status == "Playing" {
                    log::info!("Pausing MPRIS player: {}", player);
                    let _ = Command::new("dbus-send")
                        .args([
                            "--print-reply",
                            &format!("--dest={}", player),
                            "/org/mpris/MediaPlayer2",
                            "org.mpris.MediaPlayer2.Player.Pause",
                        ])
                        .output();
                    paused.insert(player.clone());
                } else {
                    log::debug!("Skipping {} (status: {})", player, status);
                }
            }
        }

        Ok(())
    }

    fn resume_all_via_mpris() -> Result<()> {
        let paused = PAUSED_PLAYERS.lock().unwrap();
        let players: Vec<String> = paused.iter().cloned().collect();

        if players.is_empty() {
            log::info!("No previously paused players to resume");
            return Ok(());
        }

        for player in &players {
            log::info!("Resuming MPRIS player: {}", player);
            let _ = Command::new("dbus-send")
                .args([
                    "--print-reply",
                    &format!("--dest={}", player),
                    "/org/mpris/MediaPlayer2",
                    "org.mpris.MediaPlayer2.Player.Play",
                ])
                .output();
        }

        Ok(())
    }

    fn pause_all_via_xdotool() -> Result<()> {
        let status = Command::new("xdotool")
            .args(["key", "--window", "0", "XF86AudioPlay"])
            .status();

        match status {
            Ok(s) if s.success() => Ok(()),
            Ok(s) => Err(AppError::Platform(format!(
                "xdotool exited with status {s}"
            ))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                log::warn!("xdotool not found; skipping media key");
                Ok(())
            }
            Err(e) => Err(AppError::Io(e)),
        }
    }

    fn resume_all_via_xdotool() -> Result<()> {
        pause_all_via_xdotool()
    }
}
