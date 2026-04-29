//! Main library entry point for the Minute of Silence application.

mod commands;
mod core;
mod error;
mod state;
mod tray;

use tauri::Manager;
rust_i18n::i18n!("locales");

pub use core::settings::{AudioPreset, Settings};
pub use error::{AppError, Result};
pub use state::AppState;

#[cfg(target_os = "windows")]
mod is_msix;
#[cfg(target_os = "linux")]
mod platform_linux;
#[cfg(target_os = "windows")]
mod platform_windows;
#[cfg(target_os = "windows")]
mod platform_windows_notifications;

/// Application entry point — called from `main.rs`.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w
                    .unminimize()
                    .and_then(|_| w.show())
                    .and_then(|_| w.set_focus());
            }
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            let handle = app.handle();

            // --- 1. Localization ---
            let locale = sys_locale::get_locale().unwrap_or_else(|| "uk".to_string());
            let lang = locale.split(['-', '_']).next().unwrap_or("uk");
            rust_i18n::set_locale(lang);
            log::info!("Backend locale set to: {}, source: {}", lang, locale);

            // --- 2. State Management ---
            let settings = Settings::load_or_default();
            app.manage(AppState::new_with_settings(
                handle.clone(),
                settings.clone(),
            ));

            // --- 3. Autostart & Snap Logic ---
            let is_hidden = std::env::args().any(|arg| arg == "--hidden");

            #[cfg(not(test))]
            {
                let is_snap = std::env::var("SNAP").is_ok();

                #[cfg(target_os = "windows")]
                let is_msix = crate::is_msix::is_msix_package();

                #[cfg(not(target_os = "windows"))]
                let is_msix = false;

                if is_msix {
                    log::info!(
                        "Running as MSIX package — autostart is managed by the StartupTask \
                         manifest extension (autostart_enabled = {}).",
                        settings.autostart_enabled
                    );
                } else if is_snap {
                    #[cfg(target_os = "linux")]
                    update_snap_autostart(settings.autostart_enabled);

                    if is_hidden && !settings.autostart_enabled {
                        log::info!("Autostart is disabled in settings. Exiting Snap instance launched with --hidden.");
                        std::process::exit(0);
                    }
                } else {
                    use tauri_plugin_autostart::ManagerExt;
                    let autostart_manager = app.autolaunch();
                    if settings.autostart_enabled {
                        let _ = autostart_manager.enable();
                    } else {
                        let _ = autostart_manager.disable();
                    }
                }
            }

            // --- 4. UI Initialization ---
            tray::build_tray(app)?;

            if let Some(window) = app.get_webview_window("main") {
                if is_hidden {
                    window.hide()?;
                }
                #[cfg(target_os = "windows")]
                window.set_skip_taskbar(true)?;
            }

            // --- 5. Core Services ---
            let app_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                core::scheduler::run(app_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::get_status,
            commands::sync_ntp_now,
            commands::skip_next,
            commands::unskip_next,
            commands::trigger_ceremony_now,
            commands::finish_ceremony_now,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Minute of Silence");
}

#[cfg(target_os = "linux")]
pub(crate) fn update_snap_autostart(enabled: bool) {
    let home = std::env::var("SNAP_REAL_HOME")
        .or_else(|_| std::env::var("USER").map(|u| format!("/home/{}", u)))
        .unwrap_or_default();

    if home.is_empty() {
        log::warn!("Could not determine real HOME directory for Snap autostart.");
        return;
    }

    let autostart_dir = std::path::PathBuf::from(home).join(".config/autostart");
    let desktop_file_path = autostart_dir.join("minute-of-silence.desktop");

    if enabled {
        if !autostart_dir.exists() {
            let _ = std::fs::create_dir_all(&autostart_dir);
        }

        let content = "[Desktop Entry]\n\
                       Name=Хвилина мовчання\n\
                       Exec=minute-of-silence --hidden\n\
                       Icon=minute-of-silence\n\
                       Terminal=false\n\
                       Type=Application\n\
                       Categories=Utility;Clock;\n\
                       X-GNOME-Autostart-enabled=true\n";

        if let Err(e) = std::fs::write(&desktop_file_path, content) {
            log::error!("Failed to write Snap autostart file: {}", e);
        } else {
            log::info!("Snap autostart file updated at: {:?}", desktop_file_path);
        }
    } else if desktop_file_path.exists() {
        let _ = std::fs::remove_file(&desktop_file_path);
        log::info!("Snap autostart file removed.");
    }
}
