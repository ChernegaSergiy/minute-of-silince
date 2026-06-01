//! macOS-specific autostart management.

pub fn system_autostart_enabled() -> Option<bool> {
    None
}

pub fn apply_autostart_enabled(app: &tauri::AppHandle, enabled: bool) {
    use tauri_plugin_autostart::ManagerExt;
    let autostart_manager = app.autolaunch();
    if enabled {
        let _ = autostart_manager.enable();
    } else {
        let _ = autostart_manager.disable();
    }
}
