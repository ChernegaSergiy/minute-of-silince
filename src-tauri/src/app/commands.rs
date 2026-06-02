//! Tauri IPC commands exposed to the frontend via `invoke()`.

use std::{
    collections::VecDeque,
    fs,
    io::{BufRead, BufReader},
};

use tauri::{AppHandle, Manager, State};

use crate::{
    Result,
    app::next_skip_date,
    state::{AppState, StatusSnapshot},
};

// Settings

/// Sync the persisted autostart setting with the actual platform state.
#[tauri::command]
pub fn sync_autostart_from_system(state: State<'_, AppState>) -> Result<()> {
    crate::platform::sync_autostart_from_system(state)
}

// Status

/// Return a lightweight runtime status snapshot.
#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> StatusSnapshot {
    state.get_snapshot()
}

// Skip / unskip

/// Skip the ceremony for the next calendar day.
#[tauri::command]
pub fn skip_next(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    let skip_date = next_skip_date(chrono::Local::now());
    let mut inner = state.lock();
    inner.settings.skip_date = Some(skip_date);
    inner.settings.save_to_store(&app)?;
    log::info!("Next ceremony skipped (date: {skip_date})");
    Ok(())
}

/// Remove the skip flag for the next calendar day.
#[tauri::command]
pub fn unskip_next(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    let mut inner = state.lock();
    inner.settings.skip_date = None;
    inner.settings.save_to_store(&app)?;
    log::info!("Skip for next ceremony removed");
    Ok(())
}

// Manual trigger

/// Force immediate NTP synchronization.
#[tauri::command]
pub async fn sync_ntp_now(state: State<'_, AppState>) -> Result<StatusSnapshot> {
    log::info!("Manual NTP sync requested");
    let _ = state.ntp_service.sync().await;
    Ok(state.get_snapshot())
}

/// Immediately trigger the ceremony (for testing / demonstration purposes).
#[tauri::command]
pub async fn trigger_ceremony_now(app: AppHandle) -> Result<()> {
    log::info!("Manual ceremony trigger requested");
    crate::core::scheduler::trigger_now(app).await;
    Ok(())
}

/// Finish the ceremony early (called by frontend when audio playback completes).
#[tauri::command]
pub async fn finish_ceremony_now(app: AppHandle) -> Result<()> {
    log::info!("Ceremony finish requested by audio engine");
    let platform = crate::platform::get_platform();
    crate::core::CeremonyManager::finish_ceremony(app, platform).await;
    Ok(())
}

/// Return debug info and a tail of the latest application log file.
#[tauri::command]
pub fn get_log_contents(app: AppHandle) -> Result<String> {
    const MAX_LOG_TAIL_LINES: usize = 200;

    let log_dir = app.path().app_log_dir()?;
    let package = app.package_info();
    let log_path = log_dir.join(format!("{}.log", package.name));

    let mut lines = vec![
        format!("app: {}", package.name),
        format!("version: {}", package.version),
        format!("os: {}", std::env::consts::OS),
        format!("arch: {}", std::env::consts::ARCH),
        format!("log_dir: {}", log_dir.display()),
    ];

    if log_path.is_file() {
        let file = fs::File::open(&log_path)?;
        let reader = BufReader::new(file);
        let mut tail = VecDeque::with_capacity(MAX_LOG_TAIL_LINES);

        for line in reader.lines() {
            let line = line?;
            if tail.len() == MAX_LOG_TAIL_LINES {
                tail.pop_front();
            }
            tail.push_back(line);
        }

        lines.push(format!("log_file: {}", log_path.display()));
        lines.push(format!("lines_copied: {}", tail.len()));
        lines.push(String::from("---"));
        lines.extend(tail);
    } else {
        lines.push(String::from("log_file: not found"));
        lines.push(String::from("lines_copied: 0"));
        lines.push(String::from("---"));
    }

    Ok(lines.join("\n"))
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub date: Option<String>,
    pub body: Option<String>,
}

/// Check for updates and store the result in AppState.
#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<UpdateInfo>> {
    if !crate::platform::should_check_for_updates() {
        return Ok(None);
    }

    // Check if there is already a pending update stored in state
    {
        let inner = state.lock();
        if let Some(ref update) = inner.pending_update {
            return Ok(Some(UpdateInfo {
                version: update.version.clone(),
                current_version: app.package_info().version.to_string(),
                date: update.date.clone(),
                body: update.body.clone(),
            }));
        }
    }

    use tauri_plugin_updater::UpdaterExt;
    log::info!("Checking for updates...");
    let updater = app
        .updater_builder()
        .build()
        .map_err(|e| crate::AppError::Update(e.to_string()))?;
    match updater.check().await {
        Ok(Some(update)) => {
            log::info!("Update found: {:?}", update.version);
            let info = UpdateInfo {
                version: update.version.clone(),
                current_version: app.package_info().version.to_string(),
                date: update.date.clone(),
                body: update.body.clone(),
            };

            let mut inner = state.lock();
            inner.pending_update = Some(update);
            Ok(Some(info))
        }
        Ok(None) => {
            log::info!("No updates available.");
            Ok(None)
        }
        Err(e) => {
            log::error!("Failed to check for updates: {}", e);
            Err(crate::AppError::Update(e.to_string()))
        }
    }
}

/// Download and install the pending update.
#[tauri::command]
pub async fn install_update(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    log::info!("Installing update...");
    let update = {
        let mut inner = state.lock();
        inner.pending_update.take()
    };

    if let Some(update) = update {
        use tauri::Emitter;
        use tauri_plugin_process::ProcessExt;

        #[derive(serde::Serialize, Clone)]
        struct ProgressPayload {
            progress: f64,
            status: String,
        }

        let app_clone = app.clone();
        let mut downloaded = 0;

        let res = update
            .download_and_install(
                move |chunk_length, total_length| {
                    downloaded += chunk_length;
                    if let Some(total) = total_length {
                        let progress = (downloaded as f64 / total as f64) * 100.0;
                        let _ = app_clone.emit(
                            "update-progress",
                            ProgressPayload {
                                progress,
                                status: "downloading".to_string(),
                            },
                        );
                    }
                },
                move || {
                    let _ = app.emit(
                        "update-progress",
                        ProgressPayload {
                            progress: 100.0,
                            status: "installing".to_string(),
                        },
                    );
                },
            )
            .await;

        match res {
            Ok(_) => {
                log::info!("Update installed. Restarting...");
                app_clone.restart();
                Ok(())
            }
            Err(e) => {
                let err_str = e.to_string();
                log::error!("Failed to download and install update: {}", err_str);
                Err(crate::AppError::Update(err_str))
            }
        }
    } else {
        Err(crate::AppError::Update(
            "No pending update found".to_string(),
        ))
    }
}
