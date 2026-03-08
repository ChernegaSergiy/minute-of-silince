use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

/// Unified application error type.
///
/// All variants implement `serde::Serialize` so they can be forwarded to the
/// frontend via Tauri's `invoke` error channel.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialisation error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("NTP sync failed: {0}")]
    Ntp(String),

    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Settings error: {0}")]
    Settings(String),
}

// Required so Tauri commands can return `Result<T, AppError>`.
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
