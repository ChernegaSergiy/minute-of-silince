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
                let is_snap = std::env::var("SNAP").is_ok();
                let is_flatpak = std::env::var("FLATPAK_ID").is_ok();
                let is_msix = crate::platform::is_msix();

                if is_msix {
                    log::info!(
                        "Running as MSIX package — autostart is managed by the StartupTask \
                         manifest extension (autostart_enabled = {}).",
                        settings.autostart_enabled
                    );
                } else if is_snap || is_flatpak {
                    // Snap/Flatpak autostart: manage the .desktop file manually
                    // to ensure correct Exec commands and paths.
                    #[cfg(target_os = "linux")]
                    crate::platform::linux::autostart::manage(settings.autostart_enabled);

                    if is_hidden && !settings.autostart_enabled {
                        log::info!(
                            "Autostart is disabled in settings. \
                             Exiting {} instance launched with --hidden.",
                            if is_snap { "Snap" } else { "Flatpak" }
                        );
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
            app::tray::build_tray(app)?;

            if let Some(window) = app.get_webview_window("main") {
                if is_hidden {
                    window.hide()?;
                }
                #[cfg(target_os = "windows")]
                {
                    window.set_skip_taskbar(true)?;
                    crate::platform::windows::power::register_power_hook(&window);
                }
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
            app::commands::save_settings,
            app::commands::get_status,
            app::commands::sync_ntp_now,
            app::commands::skip_next,
            app::commands::unskip_next,
            app::commands::trigger_ceremony_now,
            app::commands::finish_ceremony_now,
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
