use std::process::Command;

#[allow(dead_code)]
pub fn create_autostart_task(exe_path: &str) -> Result<(), String> {
    let task_name = "MinuteOfSilence";
    log::info!("Creating Windows Task Scheduler task: {}", exe_path);

    // First remove existing task if any
    let _ = Command::new("schtasks")
        .args(["/Delete", "/TN", task_name, "/F"])
        .output();

    // Create new task
    let output = Command::new("schtasks")
        .args([
            "/Create", "/TN", task_name, "/TR", exe_path, "/SC", "ONLOGON", "/RL", "LIMITED", "/F",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        log::info!("Successfully created autostart task");
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        log::error!("Failed to create autostart task: {}", error);
        Err(error)
    }
}

#[allow(dead_code)]
pub fn remove_autostart_task() -> Result<(), String> {
    let task_name = "MinuteOfSilence";

    let output = Command::new("schtasks")
        .args(["/Delete", "/TN", task_name, "/F"])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[allow(dead_code)]
pub fn is_autostart_task_exists() -> bool {
    let task_name = "MinuteOfSilence";

    Command::new("schtasks")
        .args(["/Query", "/TN", task_name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
