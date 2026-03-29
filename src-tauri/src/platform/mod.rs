#[cfg(target_os = "windows")]
pub use crate::platform_windows::{media, power, volume};

#[cfg(target_os = "linux")]
pub use crate::platform_linux::{media, volume};

#[cfg(target_os = "linux")]
pub mod power {
    pub fn is_resume_event(_wparam: usize) -> bool {
        false
    }
}
