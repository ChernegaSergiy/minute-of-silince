//! Pause / resume system-wide media playback on Windows.

use log::{error, info};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackInfo,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};

use crate::error::{AppError, Result};

pub async fn pause_all() -> Result<Vec<String>> {
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?
        .await
        .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?;

    let sessions = manager
        .GetSessions()
        .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?;

    let count = sessions
        .Size()
        .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?;

    let mut paused_ids = Vec::new();
    info!("Found {} media sessions", count);

    for i in 0..count {
        if let Ok(session) = sessions.GetAt(i) {
            let app_id = session.SourceAppUserModelId().unwrap_or_default();
            let app_id_str = app_id.to_string();

            let playback_info = match session.GetPlaybackInfo() {
                Ok(info) => info,
                Err(e) => {
                    error!("Failed to get playback info for session {}: {:?}", i, e);
                    continue;
                }
            };

            let status = match playback_info.PlaybackStatus() {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to get playback status for session {}: {:?}", i, e);
                    continue;
                }
            };

            if status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing {
                info!("Pausing session {}: AppId={}", i, app_id_str);
                if session.TryPauseAsync().is_ok() {
                    paused_ids.push(app_id_str);
                }
            }
        }
    }

    Ok(paused_ids)
}

pub async fn resume_specific(players: Vec<String>) -> Result<()> {
    if players.is_empty() {
        return Ok(());
    }

    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?
        .await
        .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?;

    let sessions = manager
        .GetSessions()
        .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?;

    let count = sessions
        .Size()
        .map_err(|e: windows::core::Error| AppError::Platform(e.to_string()))?;

    for i in 0..count {
        if let Ok(session) = sessions.GetAt(i) {
            let app_id = session
                .SourceAppUserModelId()
                .unwrap_or_default()
                .to_string();

            if players.contains(&app_id) {
                let playback_info = match session.GetPlaybackInfo() {
                    Ok(info) => info,
                    Err(_) => continue,
                };

                let status = match playback_info.PlaybackStatus() {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                if status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused {
                    info!("Resuming session {}: AppId={}", i, app_id);
                    let _ = session.TryPlayAsync();
                }
            }
        }
    }

    Ok(())
}
