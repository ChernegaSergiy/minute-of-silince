//! Manage snap and flatpak autostart by writing/removing a .desktop file in
//! appropriate config directories.

pub fn manage(enable: bool) {
    if let Ok(snap_user_data) = std::env::var("SNAP_USER_DATA") {
        manage_snap(enable, &snap_user_data);
    } else if let Ok(flatpak_id) = std::env::var("FLATPAK_ID") {
        manage_flatpak(enable, &flatpak_id);
    } else {
        log::debug!("Not running as a snap or flatpak, skipping autostart management");
    }
}

fn manage_snap(enable: bool, snap_user_data: &str) {
    let autostart_dir = std::path::PathBuf::from(snap_user_data).join(".config/autostart");
    let desktop_path = autostart_dir.join("minute-of-silence.desktop");

    if enable {
        if let Err(e) = std::fs::create_dir_all(&autostart_dir) {
            log::warn!(
                "snap autostart: cannot create dir {:?}: {}",
                autostart_dir,
                e
            );
            return;
        }
        let content = "[Desktop Entry]\n\
            Type=Application\n\
            Name=Minute of Silence\n\
            Exec=minute-of-silence --hidden\n\
            Icon=minute-of-silence\n\
            Hidden=false\n\
            NoDisplay=false\n";
        match std::fs::write(&desktop_path, content) {
            Ok(_) => log::info!("snap autostart: enabled ({:?})", desktop_path),
            Err(e) => log::warn!("snap autostart: write failed: {}", e),
        }
    } else {
        remove_file(&desktop_path, "snap");
    }
}

fn manage_flatpak(enable: bool, flatpak_id: &str) {
    // In Flatpak, $HOME is mapped to the sandbox home.
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/runner".to_string());
    let autostart_dir = std::path::PathBuf::from(home).join(".config/autostart");
    let desktop_path = autostart_dir.join(format!("{}.desktop", flatpak_id));

    if enable {
        if let Err(e) = std::fs::create_dir_all(&autostart_dir) {
            log::warn!(
                "flatpak autostart: cannot create dir {:?}: {}",
                autostart_dir,
                e
            );
            return;
        }
        // For Flatpak, we use `flatpak run` command
        let content = format!(
            "[Desktop Entry]\n\
            Type=Application\n\
            Name=Minute of Silence\n\
            Exec=flatpak run {} --hidden\n\
            Hidden=false\n\
            NoDisplay=false\n",
            flatpak_id
        );
        match std::fs::write(&desktop_path, content) {
            Ok(_) => log::info!("flatpak autostart: enabled ({:?})", desktop_path),
            Err(e) => log::warn!("flatpak autostart: write failed: {}", e),
        }
    } else {
        remove_file(&desktop_path, "flatpak");
    }
}

fn remove_file(path: &std::path::Path, context: &str) {
    match std::fs::remove_file(path) {
        Ok(_) => log::info!("{} autostart: disabled ({:?})", context, path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {} // already absent
        Err(e) => log::warn!("{} autostart: remove failed: {}", context, e),
    }
}

/// Returns the current autostart state reported by the platform.
pub fn system_autostart_enabled() -> Option<bool> {
    if let Ok(snap_user_data) = std::env::var("SNAP_USER_DATA") {
        let desktop_path = std::path::PathBuf::from(snap_user_data)
            .join(".config/autostart/minute-of-silence.desktop");
        Some(desktop_path.exists())
    } else if let Ok(flatpak_id) = std::env::var("FLATPAK_ID") {
        let home = std::env::var("HOME").ok()?;
        let desktop_path = std::path::PathBuf::from(home)
            .join(".config/autostart")
            .join(format!("{}.desktop", flatpak_id));
        Some(desktop_path.exists())
    } else {
        None
    }
}

/// Apply the requested autostart state to the current platform.
pub fn apply_autostart_enabled(app: &tauri::AppHandle, enabled: bool) {
    let is_snap = std::env::var("SNAP").is_ok();
    let is_flatpak = std::env::var("FLATPAK_ID").is_ok();

    if is_snap || is_flatpak {
        manage(enabled);
    } else {
        use tauri_plugin_autostart::ManagerExt;
        let autostart_manager = app.autolaunch();
        if enabled {
            let _ = autostart_manager.enable();
        } else {
            let _ = autostart_manager.disable();
        }
    }
}
