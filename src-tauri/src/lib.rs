mod commands;
mod core;
mod error;
mod state;
mod tray;

#[cfg(target_os = "windows")]
use tauri::Manager;

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
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .manage(AppState::new())
        .setup(|app| {
            // Build the system-tray icon.
            tray::build_tray(app)?;

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
