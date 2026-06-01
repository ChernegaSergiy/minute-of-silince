//! macOS-specific theme detection.

pub fn detect_system_theme() -> bool {
    use std::process::Command;
    let output = Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.trim().to_lowercase().contains("dark")
    } else {
        false
    }
}

pub fn is_dark_mode() -> bool {
    detect_system_theme()
}
