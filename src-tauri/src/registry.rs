//! Registry access helpers for Windows personalization settings.
//!
//! All access is done under `HKEY_CURRENT_USER` (per-user), which never
//! requires Administrator privileges. We always access the 64-bit registry
//! view explicitly (`KEY_WOW64_64KEY`) so behavior is deterministic
//! regardless of how the process is launched.

use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY, KEY_WRITE};
use winreg::RegKey;

use crate::error::{AppError, AppResult};

/// Registry paths used by the application (all under HKCU except the
/// Windows build number which is read-only from HKLM).
pub const PERSONALIZE_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
pub const DWM_KEY: &str = r"Software\Microsoft\Windows\DWM";
pub const DESKTOP_KEY: &str = r"Control Panel\Desktop";
pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
pub const NT_CURRENT_VERSION_KEY: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion";

/// Read a REG_DWORD from HKCU. Returns `Ok(None)` when the value does not
/// exist (which is a legitimate state on older Windows builds).
pub fn read_hkcu_dword(subkey: &str, value: &str) -> AppResult<Option<u32>> {
    let key = match RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(subkey, KEY_READ | KEY_WOW64_64KEY)
    {
        Ok(k) => k,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(AppError::registry_read(subkey, value, e)),
    };
    match key.get_value::<u32, _>(value) {
        Ok(v) => Ok(Some(v)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(AppError::registry_read(subkey, value, e)),
    }
}

/// Write a REG_DWORD to HKCU, creating the key if needed.
pub fn write_hkcu_dword(subkey: &str, value: &str, data: u32) -> AppResult<()> {
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey_with_flags(subkey, KEY_WRITE | KEY_WOW64_64KEY)
        .map_err(|e| AppError::registry_write(subkey, value, e))?;
    key.set_value(value, &data)
        .map_err(|e| AppError::registry_write(subkey, value, e))
}

/// Read a REG_SZ string from HKCU. Returns `Ok(None)` when missing.
pub fn read_hkcu_string(subkey: &str, value: &str) -> AppResult<Option<String>> {
    let key = match RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(subkey, KEY_READ | KEY_WOW64_64KEY)
    {
        Ok(k) => k,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(AppError::registry_read(subkey, value, e)),
    };
    match key.get_value::<String, _>(value) {
        Ok(v) => Ok(Some(v)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(AppError::registry_read(subkey, value, e)),
    }
}

/// Read a REG_SZ string from HKLM (read-only system info).
pub fn read_hklm_string(subkey: &str, value: &str) -> AppResult<Option<String>> {
    let key = match RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(subkey, KEY_READ | KEY_WOW64_64KEY)
    {
        Ok(k) => k,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(AppError::registry_read(subkey, value, e)),
    };
    match key.get_value::<String, _>(value) {
        Ok(v) => Ok(Some(v)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(AppError::registry_read(subkey, value, e)),
    }
}

/// Read the Windows build number from HKLM. Returns 0 when unavailable.
pub fn read_windows_build() -> u32 {
    read_hklm_string(NT_CURRENT_VERSION_KEY, "CurrentBuildNumber")
        .ok()
        .flatten()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_windows_build_is_plausible() {
        // On any supported machine the build is at least 17763 (Win10 1809).
        let build = read_windows_build();
        assert!(build >= 17763, "unexpected build: {build}");
    }

    #[test]
    fn missing_value_returns_none() {
        let v = read_hkcu_dword(PERSONALIZE_KEY, "ThisValueDoesNotExist___");
        assert!(matches!(v, Ok(None)));
    }

    #[test]
    fn missing_key_returns_none() {
        let v = read_hkcu_string(r"Software\WTS_Nonexistent_Key_123", "x");
        assert!(matches!(v, Ok(None)));
    }

    #[test]
    fn write_and_read_back_roundtrip() {
        // Uses a scratch key under HKCU to avoid touching real settings.
        const SCRATCH: &str = r"Software\WindowsThemeSwitcher_Test";
        write_hkcu_dword(SCRATCH, "test_value", 42).unwrap();
        assert_eq!(read_hkcu_dword(SCRATCH, "test_value").unwrap(), Some(42));
        // cleanup
        let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey(SCRATCH);
    }
}
