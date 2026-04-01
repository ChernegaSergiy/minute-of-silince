//! Linux platform integrations using native ALSA and D-Bus (MPRIS) APIs.

use crate::error::{AppError, Result};
use alsa::mixer::{Mixer, Selem, SelemId};
use std::collections::HashSet;
use std::sync::Mutex;
use zbus::proxy;

lazy_static::lazy_static! {
    static ref PAUSED_PLAYERS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

pub mod volume {
    use super::*;

    pub fn get_volume() -> Result<u8> {
        let mixer = open_mixer()?;
        let selem = find_master_selem(&mixer)?;

        let (min, max) = selem.get_playback_volume_range();
        let volume = selem
            .get_playback_volume(alsa::mixer::SelemChannelId::FrontLeft)
            .map_err(|e| AppError::Platform(e.to_string()))?;

        if max == min {
            return Ok(0);
        }

        let percent = ((volume - min) as f64 / (max - min) as f64 * 100.0) as u8;
        Ok(percent)
    }

    pub fn set_volume(level: u8) -> Result<()> {
        let mixer = open_mixer()?;
        let selem = find_master_selem(&mixer)?;

        let (min, max) = selem.get_playback_volume_range();
        let val = (min as f64 + (max - min) as f64 * (level as f64 / 100.0)) as i64;

        selem
            .set_playback_volume_all(val)
            .map_err(|e| AppError::Platform(e.to_string()))?;

        Ok(())
    }

    pub fn is_muted() -> Result<bool> {
        let mixer = open_mixer()?;
        let selem = find_master_selem(&mixer)?;

        let switch = selem
            .get_playback_switch(alsa::mixer::SelemChannelId::FrontLeft)
            .map_err(|e| AppError::Platform(e.to_string()))?;

        Ok(switch == 0)
    }

    pub fn set_mute(mute: bool) -> Result<()> {
        let mixer = open_mixer()?;
        let selem = find_master_selem(&mixer)?;

        let switch = if mute { 0 } else { 1 };
        selem
            .set_playback_switch_all(switch)
            .map_err(|e| AppError::Platform(e.to_string()))?;

        Ok(())
    }

    fn open_mixer() -> Result<Mixer> {
        Mixer::new("default", false).map_err(|e| AppError::Platform(e.to_string()))
    }

    fn find_master_selem(mixer: &Mixer) -> Result<Selem<'_>> {
        let sid = SelemId::new("Master", 0);
        mixer
            .find_selem(&sid)
            .ok_or_else(|| AppError::Platform("Could not find 'Master' mixer element".into()))
    }
}

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

pub mod media {
    use super::*;
    use zbus::blocking::Connection;

    pub fn pause_all() -> Result<()> {
        let conn = Connection::session().map_err(|e| AppError::Platform(e.to_string()))?;
        let dbus = zbus::blocking::fdo::DBusProxy::new(&conn)
            .map_err(|e| AppError::Platform(e.to_string()))?;
        let names = dbus
            .list_names()
            .map_err(|e| AppError::Platform(e.to_string()))?;

        let mut paused = PAUSED_PLAYERS.lock().unwrap();
        paused.clear();

        for name in names {
            if name.starts_with("org.mpris.MediaPlayer2.") {
                let player = MediaPlayer2PlayerProxyBlocking::builder(&conn)
                    .destination(name.as_str())
                    .map_err(|e| AppError::Platform(e.to_string()))?
                    .build()
                    .map_err(|e| AppError::Platform(e.to_string()))?;

                if let Ok(status) = player.playback_status() {
                    if status == "Playing" && player.pause().is_ok() {
                        paused.insert(name.to_string());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn resume_all() -> Result<()> {
        let conn = Connection::session().map_err(|e| AppError::Platform(e.to_string()))?;
        let mut paused = PAUSED_PLAYERS.lock().unwrap();

        for name in paused.iter() {
            let player = MediaPlayer2PlayerProxyBlocking::builder(&conn)
                .destination(name.as_str())
                .map_err(|e| AppError::Platform(e.to_string()))?
                .build()
                .map_err(|e| AppError::Platform(e.to_string()))?;

            let _ = player.play();
        }

        paused.clear();
        Ok(())
    }
}
