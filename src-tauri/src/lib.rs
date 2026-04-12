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

#[cfg(target_os = "linux")]
mod platform_linux;
#[cfg(target_os = "windows")]
mod platform_windows;

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
            let is_snap = std::env::var("SNAP").is_ok();

            // Sync the actual OS autostart file with settings.
            #[cfg(not(test))]
            if !is_snap {
                use tauri_plugin_autostart::ManagerExt;
                let autostart_manager = app.autolaunch();
                if settings.autostart_enabled {
                    let _ = autostart_manager.enable();
                } else {
                    let _ = autostart_manager.disable();
                }
            } else if is_hidden && !settings.autostart_enabled {
                // If started automatically by Snap but user disabled autostart in settings, exit.
                log::info!("Autostart is disabled in settings. Exiting Snap instance launched with --hidden.");
                std::process::exit(0);
            }

            // --- 4. UI Initialization ---
            tray::build_tray(app)?;

            if let Some(window) = app.get_webview_window("main") {
                if is_hidden {
                    window.hide()?;
                }
                // Hide from taskbar; the app lives in the tray only.
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
            // Close button minimises to tray instead of quitting.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Minute of Silence");
}
