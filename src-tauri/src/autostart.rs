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
///
/// Kept for the tests and for callers that only care about existence; the
/// app itself uses [`sync`], which also compares the stored path.
#[cfg_attr(not(test), allow(dead_code))]
pub fn is_enabled() -> bool {
    current_command().is_some()
}

/// The command line currently stored in the Run key, if any.
pub fn current_command() -> Option<String> {
    registry::read_hkcu_string(registry::RUN_KEY, RUN_VALUE_NAME)
        .ok()
        .flatten()
}

/// Bring the Run key in line with `enabled` for `exe_path`.
///
/// Unlike [`is_enabled`] this also compares the *path*, so an update that
/// moves the executable (or a user switching between the installed and a
/// portable build) rewrites the stale entry instead of leaving autostart
/// pointing at a binary that no longer exists.
///
/// Returns `Ok(true)` when the registry was changed.
pub fn sync(exe_path: &str, enabled: bool) -> AppResult<bool> {
    let expected = format!("\"{exe_path}\"");
    let current = current_command();

    if enabled {
        if current.as_deref() == Some(expected.as_str()) {
            return Ok(false);
        }
        set_enabled(exe_path, true)?;
        Ok(true)
    } else {
        if current.is_none() {
            return Ok(false);
        }
        set_enabled(exe_path, false)?;
        Ok(true)
    }
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

    /// Both tests below write the *real* Run key, and Rust runs tests in
    /// parallel threads by default - without this lock they clobber each
    /// other's value and fail intermittently.
    static RUN_KEY_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn lock_run_key() -> std::sync::MutexGuard<'static, ()> {
        RUN_KEY_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn run_value_is_quoted() {
        assert_eq!(
            run_value_for(r"C:\Program Files\App\app.exe"),
            "\"C:\\Program Files\\App\\app.exe\""
        );
    }

    #[test]
    fn sync_rewrites_a_stale_path() {
        let _guard = lock_run_key();
        // Uses the real Run key but always restores the previous value.
        let exe = std::env::current_exe().unwrap();
        let exe_path = exe.to_string_lossy().to_string();
        let original = current_command();

        // A stale entry for a path that does not exist.
        match set_enabled(r"C:\definitely\not\here\old.exe", true) {
            Ok(()) => {
                assert_eq!(
                    current_command().as_deref(),
                    Some("\"C:\\definitely\\not\\here\\old.exe\"")
                );
                // sync must notice the mismatch and rewrite it.
                let changed = sync(&exe_path, true).unwrap();
                assert!(changed);
                assert_eq!(current_command().as_deref(), Some(format!("\"{exe_path}\"").as_str()));
                // A second sync is a no-op.
                assert!(!sync(&exe_path, true).unwrap());
            }
            Err(e) => eprintln!("skipping sync test (environment restriction): {e}"),
        }

        // restore
        match original {
            Some(cmd) => {
                let _ = set_enabled(cmd.trim_matches('"'), true);
            }
            None => {
                let _ = set_enabled(&exe_path, false);
            }
        }
    }

    #[test]
    fn enable_disable_roundtrip() {
        let _guard = lock_run_key();
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
