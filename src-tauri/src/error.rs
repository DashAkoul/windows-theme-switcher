//! Central error type shared by every Tauri command and internal module.
//!
//! Errors are serialized to the frontend as `{ code, message }` so the UI can
//! show actionable dialogs (retry / open Windows Settings / copy diagnostics)
//! instead of crashing.

use serde::Serialize;

/// Stable, machine-readable error codes. The frontend matches on these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    RegistryRead,
    RegistryWrite,
    Wallpaper,
    ConfigRead,
    ConfigWrite,
    ConfigInvalid,
    Autostart,
    ProfileNotFound,
    Io,
    WindowsApi,
}

/// Application-wide error type.
#[derive(Debug)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn registry_read(key: &str, value: &str, err: impl std::fmt::Display) -> Self {
        Self::new(
            ErrorCode::RegistryRead,
            format!("Failed to read registry value '{key}\\{value}': {err}"),
        )
    }

    pub fn registry_write(key: &str, value: &str, err: impl std::fmt::Display) -> Self {
        Self::new(
            ErrorCode::RegistryWrite,
            format!("Failed to write registry value '{key}\\{value}': {err}"),
        )
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

/// Tauri requires command errors to be serializable. We expose a stable
/// shape instead of leaking internal error details.
impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", &self.code)?;
        state.serialize_field("message", &self.message)?;
        state.end()
    }
}

// ---------------------------------------------------------------------------
// Conversions from common error sources
// ---------------------------------------------------------------------------

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::new(ErrorCode::Io, err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::new(ErrorCode::ConfigRead, format!("Invalid configuration JSON: {err}"))
    }
}

/// Convenience alias used across the codebase.
pub type AppResult<T> = Result<T, AppError>;
