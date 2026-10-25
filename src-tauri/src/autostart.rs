//! "Start with Windows" support via the per-user Run registry key.
//!
//! Uses `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, which does
//! not require Administrator privileges (per-user autostart).

use crate::error::{AppError, AppResult};
use crate::registry;
use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
use winreg::RegKey;

/// Value name under the Run key.
pub const RUN_VALUE_NAME: &str = "Windows Theme Switcher";

/// Whether autostart is currently enabled (Run value exists).
pub fn is_enabled() -> bool {
    matches!(
        registry::read_hkcu_string(registry::RUN_KEY, RUN_VALUE_NAME),
        Ok(Some(_))
    )
}

/// Enable or disable autostart by writing/removing the Run value.
/// `exe_path` must be the absolute path to the current executable.
pub fn set_enabled(exe_path: &str, enabled: bool) -> AppResult<()> {
    if enabled {
        // Quote the path so spaces in the folder name are handled.
        let value = format!("\"{exe_path}\"");
        let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey(registry::RUN_KEY)
            .map_err(|e| AppError::new(crate::error::ErrorCode::Autostart, e.to_string()))?;
        key.set_value(RUN_VALUE_NAME, &value)
            .map_err(|e| AppError::new(crate::error::ErrorCode::Autostart, e.to_string()))?;
    } else {
        // Remove the value; ignore "not found" (already disabled).
        match RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags(registry::RUN_KEY, KEY_WRITE)
        {
            Ok(key) => match key.delete_value(RUN_VALUE_NAME) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => {
                    return Err(AppError::new(
                        crate::error::ErrorCode::Autostart,
                        e.to_string(),
                    ))
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(AppError::new(crate::error::ErrorCode::Autostart, e.to_string())),
        }
    }
    Ok(())
}

/// Format the Run value for a given exe path (exposed for tests).
#[allow(dead_code)]
pub fn run_value_for(exe_path: &str) -> String {
    format!("\"{exe_path}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_value_is_quoted() {
        assert_eq!(
            run_value_for(r"C:\Program Files\App\app.exe"),
            "\"C:\\Program Files\\App\\app.exe\""
        );
    }

    #[test]
    fn enable_disable_roundtrip() {
        // Uses the real Run key but restores the original state afterwards.
        // Some hardened environments (endpoint protection, sandboxed CI)
        // forbid writes to autorun locations; in that case we verify the
        // failure is reported cleanly instead of panicking.
        let exe = std::env::current_exe().unwrap();
        let exe_path = exe.to_string_lossy().to_string();

        let was_enabled = is_enabled();

        match set_enabled(&exe_path, true) {
            Ok(()) => {
                assert!(is_enabled());
                set_enabled(&exe_path, false).unwrap();
                assert!(!is_enabled());
                // restore original state
                if was_enabled {
                    let _ = set_enabled(&exe_path, true);
                }
            }
            Err(e) => {
                assert_eq!(e.code, crate::error::ErrorCode::Autostart);
                eprintln!("skipping Run key roundtrip (environment restriction): {e}");
            }
        }
    }
}
