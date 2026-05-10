//! System-tray icon setup and context-menu event handling.

use crate::platform::is_dark_mode;
use rust_i18n::t;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Emitter, Manager,
};

use crate::AppState;

/// Build and register the system-tray icon for `app`.
pub fn build_tray(app: &App) -> tauri::Result<()> {
    let open_i = MenuItem::with_id(app, "open", t!("open"), true, None::<&str>)?;
    let skip_i = MenuItem::with_id(app, "skip_next", t!("skip_next"), true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", t!("quit"), true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&open_i, &skip_i, &sep, &quit_i])?;

    let icon = if is_dark_mode() {
        tauri::include_image!("icons/tray-icon-32-light.png")
    } else {
        tauri::include_image!("icons/tray-icon-32-dark.png")
    };

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .menu(&menu)
        .tooltip(t!("tray_tooltip"))
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "skip_next" => {
                let state = app.state::<AppState>();
                let tomorrow = (chrono::Local::now() + chrono::Duration::days(1)).date_naive();
                state.lock().skip_date = Some(tomorrow);
                log::info!("Tray: next ceremony skipped ({tomorrow})");

                // Notify the frontend that the status has changed
                let _ = app.emit("status-updated", ());
            }
            "quit" => {
                log::info!("Quit requested via tray");
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // Left-click toggles the main window.
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
