//! Linux platform integrations.
//!
//! On Linux we send `XF86AudioPlay` via `xdotool` (if available) or use the
//! MPRIS D-Bus interface to pause/resume media players.

pub mod media {
    use crate::error::{AppError, Result};

    /// Pause media players via MPRIS D-Bus or xdotool fallback.
    pub fn pause_all() -> Result<()> {
        // TODO: implement D-Bus MPRIS `org.mpris.MediaPlayer2.Player.Pause`.
        // For now, fall back to xdotool if present.
        try_xdotool_media_key()
    }

    pub fn resume_all() -> Result<()> {
        try_xdotool_media_key()
    }

    fn try_xdotool_media_key() -> Result<()> {
        let status = std::process::Command::new("xdotool")
            .args(["key", "XF86AudioPlay"])
            .status();

        match status {
            Ok(s) if s.success() => Ok(()),
            Ok(s) => Err(AppError::Platform(format!(
                "xdotool exited with status {s}"
            ))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                log::warn!("xdotool not found; skipping media key");
                Ok(()) // Non-fatal: just proceed without pausing.
            }
            Err(e) => Err(AppError::Io(e)),
        }
    }
}
