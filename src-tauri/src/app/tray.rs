//! System-tray icon setup and context-menu event handling.

use crate::app::next_skip_date;
use crate::platform::is_dark_mode;
use rust_i18n::t;
use tauri::{
    App, Emitter, Manager, Wry,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::AppState;

/// Structure that explicitly owns the menu and all its elements to keep them from being dropped on Linux.
#[derive(Clone)]
pub struct TrayMenuState {
    pub menu: Menu<Wry>,
    pub open_item: MenuItem<Wry>,
    pub skip_item: MenuItem<Wry>,
    pub quit_item: MenuItem<Wry>,
}

impl std::fmt::Debug for TrayMenuState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrayMenuState")
            .field("menu", &"<Tauri Menu>")
            .field("open_item", &"<Tauri MenuItem>")
            .field("skip_item", &"<Tauri MenuItem>")
            .field("quit_item", &"<Tauri MenuItem>")
            .finish()
    }
}

/// Build and register the system-tray icon for `app`.
pub fn build_tray(app: &App) -> tauri::Result<()> {
    let open_i = MenuItem::with_id(app, "open", t!("open"), true, None::<&str>)?;
    let skip_i = MenuItem::with_id(app, "skip_next", t!("skip_next"), true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", t!("quit"), true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&open_i, &skip_i, &sep, &quit_i])?;

    // Store the tray menu in AppState to keep strong references to it and its items
    let state = app.state::<AppState>();
    let _ = state.tray_menu.set(TrayMenuState {
        menu: menu.clone(),
        open_item: open_i,
        skip_item: skip_i,
        quit_item: quit_i,
    });

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
                let skip_date = next_skip_date(chrono::Local::now());
                {
                    let mut inner = state.lock();
                    inner.settings.skip_date = Some(skip_date);
                    let _ = inner.settings.save_to_store(app);
                }
                log::info!("Tray: next ceremony skipped ({skip_date})");

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
