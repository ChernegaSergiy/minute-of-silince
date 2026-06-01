// Prevents an additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Fix window matching/icons on pure Wayland when running inside Snap
    #[cfg(target_os = "linux")]
    if std::env::var_os("SNAP").is_some() {
        glib::set_prgname(Some("minute-of-silence_minute-of-silence"));
    }

    minute_of_silence_lib::run();
}
