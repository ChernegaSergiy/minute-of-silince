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
            Name=Хвилина мовчання\n\
            Exec=minute-of-silence --hidden\n\
            Hidden=false\n\
            NoDisplay=false\n\
            X-GNOME-Autostart-enabled=true\n";
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
            Name=Хвилина мовчання\n\
            Exec=flatpak run {} --hidden\n\
            Hidden=false\n\
            NoDisplay=false\n\
            X-GNOME-Autostart-enabled=true\n\
            X-Flatpak={}\n",
            flatpak_id, flatpak_id
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
