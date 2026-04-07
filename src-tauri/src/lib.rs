//! Main library entry point for the Minute of Silence application.
//!
//! Orchestrates the initialization of the Tauri application,
//! including plugin registration, tray setup, and starting the scheduler.

mod commands;
mod core;
mod error;
mod state;
mod tray;

use tauri::Manager;

// Initialize i18n
rust_i18n::i18n!("locales");

pub use core::settings::{AudioPreset, Settings};

pub use error::{AppError, Result};
pub use state::AppState;

#[cfg(target_os = "windows")]
mod platform_windows;

#[cfg(target_os = "linux")]
mod platform_linux;

/// Application entry point — called from `main.rs`.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            if std::env::var("SNAP").is_ok() {
                Some(vec!["minute-of-silence", "--hidden"])
            } else {
                Some(vec!["--hidden"])
            },
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app.get_webview_window("main").map(|w| {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            });
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            // Set the backend locale to match the system locale
            let locale = sys_locale::get_locale().unwrap_or_else(|| "uk".to_string());
            let lang = locale
                .split(['-', '_'])
                .next()
                .unwrap_or("uk");
            rust_i18n::set_locale(lang);
            log::info!("Backend locale set to: {}, source: {}", lang, locale);

            let settings = Settings::load_or_default();
            app.manage(AppState::new_with_settings(
                app.handle().clone(),
                settings.clone(),
            ));

            // Synchronise autostart state with the plugin (skip on Snap/Flatpak where handled by package manager).
            #[cfg(not(test))]
            {
                let is_snap = std::env::var("SNAP").is_ok();
                if !is_snap {
                    use tauri_plugin_autostart::ManagerExt;
                    let autostart_manager = app.autolaunch();
                    if settings.autostart_enabled {
                        let _ = autostart_manager.enable();
                    } else {
                        let _ = autostart_manager.disable();
                    }
                }
            }

            // Build the system-tray icon.
            tray::build_tray(app)?;

            // If started with --hidden (e.g. from autostart), hide the main window immediately.
            if std::env::args().any(|arg| arg == "--hidden") {
                if let Some(window) = app.get_webview_window("main") {
                    window.hide()?;
                }
            }

            // Hide from taskbar; the app lives in the tray only.
            #[cfg(target_os = "windows")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.set_skip_taskbar(true)?;
                }
            }

            // Spawn the scheduler loop.
            let app_handle = app.handle().clone();
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
