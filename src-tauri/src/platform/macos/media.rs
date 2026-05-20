//! Pause / resume media players on macOS via NSWorkspace and osascript.

use crate::error::{AppError, Result};
use std::process::Command;

pub async fn pause_all() -> Result<Vec<String>> {
    let output = Command::new("osascript")
        .args([
            "-e",
            r#"set runningApps to {}
set appList to application registry of (system info)
repeat with theApp in appList
    try
        set theId to bundle identifier of theApp
        set end of runningApps to theId
    end try
end repeat
return runningApps"#,
        ])
        .output()
        .map_err(|e| AppError::Platform(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let bundle_ids: Vec<String> = stdout
        .trim()
        .split(", ")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    let mut paused = Vec::new();

    for bundle_id in bundle_ids {
        if let Ok(name) = get_app_name(&bundle_id) {
            if try_pause(&name) {
                paused.push(bundle_id);
                log::info!("Paused macOS player: {}", name);
            }
        }
    }

    Ok(paused)
}

fn get_app_name(bundle_id: &str) -> Result<String> {
    let output = Command::new("osascript")
        .args([
            "-e",
            &format!(
                r#"name of application id "{}""#,
                bundle_id.replace("\"", "\\\"")
            ),
        ])
        .output()
        .map_err(|e| AppError::Platform(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        return Err(AppError::Platform("App not found".to_string()));
    }
    Ok(stdout.trim().to_string())
}

fn try_pause(name: &str) -> bool {
    let script = format!(
        r#"tell application "{}"
            if player state is playing then
                pause
                return "paused"
            end if
            return "not_playing"
        end tell"#,
        name.replace("\"", "\\\"")
    );

    match Command::new("osascript").args(["-e", &script]).output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim() == "paused",
        Err(_) => false,
    }
}

pub async fn resume_specific(players: Vec<String>) -> Result<()> {
    for bundle_id in players {
        if let Ok(name) = get_app_name(&bundle_id) {
            let script = format!(
                r#"tell application "{}" play end tell"#,
                name.replace("\"", "\\\"")
            );

            if let Err(e) = Command::new("osascript").args(["-e", &script]).output() {
                log::warn!("Failed to resume {}: {}", name, e);
            } else {
                log::info!("Resumed macOS player: {}", name);
            }
        }
    }

    Ok(())
}
