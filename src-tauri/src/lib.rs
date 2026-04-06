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
            Some(vec!["--hidden"]),
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
            app.manage(AppState::new(app.handle().clone()));

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
