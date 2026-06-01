//! macOS-specific theme detection via Swift helper.

unsafe extern "C" {
    fn macos_detect_system_theme() -> bool;
}

pub fn detect_system_theme() -> bool {
    unsafe { macos_detect_system_theme() }
}

pub fn is_dark_mode() -> bool {
    detect_system_theme()
}
