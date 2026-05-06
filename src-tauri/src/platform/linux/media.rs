//! Pause media players on Linux using D-Bus (MPRIS).

use crate::error::{AppError, Result};
use zbus::proxy;

#[proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_service = "org.mpris.MediaPlayer2.spotify",
    default_path = "/org/mpris/MediaPlayer2"
)]
trait MediaPlayer2Player {
    fn pause(&self) -> zbus::Result<()>;
    fn play(&self) -> zbus::Result<()>;
    #[zbus(property)]
    fn playback_status(&self) -> zbus::Result<String>;
}

pub async fn pause_all() -> Result<Vec<String>> {
    let result = tokio::task::spawn_blocking(|| {
        let conn =
            zbus::blocking::Connection::session().map_err(|e| AppError::Platform(e.to_string()))?;
        let dbus = zbus::blocking::fdo::DBusProxy::new(&conn)
            .map_err(|e| AppError::Platform(e.to_string()))?;
        let names = dbus
            .list_names()
            .map_err(|e| AppError::Platform(e.to_string()))?;

        let mut paused_names = Vec::new();
        for name in names {
            if name.starts_with("org.mpris.MediaPlayer2.") {
                let player = MediaPlayer2PlayerProxyBlocking::builder(&conn)
                    .destination(name.as_str())
                    .map_err(|e| AppError::Platform(e.to_string()))?
                    .build()
                    .map_err(|e| AppError::Platform(e.to_string()))?;

                if let Ok(status) = player.playback_status() {
                    if status == "Playing" {
                        if player.pause().is_ok() {
                            paused_names.push(name.clone());
                        }
                    }
                }
            }
        }
        Ok(paused_names)
    })
    .await
    .map_err(|e| AppError::Platform(e.to_string()))?;

    result
}

pub async fn resume_specific(players: Vec<String>) -> Result<()> {
    if players.is_empty() {
        return Ok(());
    }

    let result = tokio::task::spawn_blocking(move || {
        let conn =
            zbus::blocking::Connection::session().map_err(|e| AppError::Platform(e.to_string()))?;

        for name in players {
            let player = MediaPlayer2PlayerProxyBlocking::builder(&conn)
                .destination(name.as_str())
                .map_err(|e| AppError::Platform(e.to_string()))?
                .build()
                .map_err(|e| AppError::Platform(e.to_string()))?;

            if let Ok(status) = player.playback_status() {
                if status == "Paused" {
                    let _ = player.play();
                }
            }
        }
        Ok(())
    })
    .await
    .map_err(|e| AppError::Platform(e.to_string()))?;

    result
}
