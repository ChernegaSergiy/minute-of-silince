// Prevents an additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    {
        // Force X11 backend for GTK on Linux to ensure system tray menu text is rendered correctly under Wayland
        if std::env::var("GDK_BACKEND").is_err() {
            unsafe {
                std::env::set_var("GDK_BACKEND", "x11");
            }
        }
    }

    minute_of_silence_lib::run();
}
