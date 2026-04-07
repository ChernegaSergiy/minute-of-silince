//! System-tray icon setup and context-menu event handling.

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Emitter, Manager,
};

use crate::AppState;

/// Build and register the system-tray icon for `app`.
pub fn build_tray(app: &App) -> tauri::Result<()> {
    let open_i = MenuItem::with_id(app, "open", "Відкрити", true, None::<&str>)?;
    let skip_i = MenuItem::with_id(app, "skip_next", "Пропустити завтра", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "Вийти", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&open_i, &skip_i, &sep, &quit_i])?;

    let icon = app.default_window_icon().cloned().unwrap_or_else(|| {
        log::warn!("Default window icon not found, tray may have no icon");
        // Fallback or handle appropriately - for now we use a placeholder or let it fail gracefully
        tauri::image::Image::new(&[], 0, 0)
    });

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .menu(&menu)
        .tooltip("Хвилина мовчання")
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
