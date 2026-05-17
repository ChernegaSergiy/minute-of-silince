//! Main library entry point for the Minute of Silence application.

mod app;
mod core;
mod error;
mod platform;
mod state;

use tauri::Manager;
rust_i18n::i18n!("locales");

pub use core::settings::{AudioPreset, Settings};
pub use error::{AppError, Result};
pub use state::AppState;

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
                let settings = app.state::<AppState>().lock().settings.clone();

                let is_snap = std::env::var("SNAP").is_ok();
                let is_flatpak = std::env::var("FLATPAK_ID").is_ok();

                if (is_snap || is_flatpak) && is_hidden && !settings.autostart_enabled {
                    log::info!(
                        "Autostart is disabled in settings. \
                         Exiting {} instance launched with --hidden.",
                        if is_snap { "Snap" } else { "Flatpak" }
                    );
                    std::process::exit(0);
                }
            }

            // --- 4. UI Initialization ---
            app::tray::build_tray(app)?;

            #[cfg(target_os = "windows")]
            crate::platform::windows::theme::start_theme_watcher(handle.clone());

            #[cfg(target_os = "linux")]
            crate::platform::linux::theme::start_theme_watcher(handle.clone());

            if let Some(window) = app.get_webview_window("main") {
                let main_window = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = main_window.hide();
                    }
                });

                if is_hidden {
                    window.hide()?;
                }
                #[cfg(target_os = "windows")]
                {
                    window.set_skip_taskbar(true)?;
                    crate::platform::windows::power::register_power_hook(&window);
                }
            }

            #[cfg(not(test))]
            {
                let app_handle = handle.clone();
                tauri::async_runtime::spawn_blocking(move || {
                    if let Err(e) = app::commands::sync_autostart_from_system(app_handle.state::<AppState>())
                    {
                        log::warn!("Failed to sync autostart from system: {}", e);
                    }
                });
            }

            // --- 5. Core Services ---
            let app_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                core::scheduler::run(app_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app::commands::get_settings,
            app::commands::sync_autostart_from_system,
            app::commands::save_settings,
            app::commands::get_status,
            app::commands::sync_ntp_now,
            app::commands::skip_next,
            app::commands::unskip_next,
            app::commands::trigger_ceremony_now,
            app::commands::finish_ceremony_now,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Minute of Silence");
}
