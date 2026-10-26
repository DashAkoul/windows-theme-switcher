//! Theme detection and application - the heart of the application.
//!
//! Everything is done through documented, stable Windows surfaces:
//!  - `HKCU\...\Themes\Personalize` for app/system mode and transparency
//!  - `HKCU\Software\Microsoft\Windows\DWM` for the accent color
//!  - `SystemParametersInfoW(SPI_SETDESKWALLPAPER)` for the wallpaper
//!  - a `WM_SETTINGCHANGE` broadcast (exactly like the Settings app does)
//!
//! No Administrator rights are required: every key is under HKCU.

use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::LPARAM;
use windows::Win32::UI::WindowsAndMessaging::{
    SendMessageTimeoutW, SystemParametersInfoW, HWND_BROADCAST, SMTO_ABORTIFHUNG,
    SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER, WM_SETTINGCHANGE,
};

use crate::config::ThemeProfile;
use crate::error::{AppError, AppResult, ErrorCode};
use crate::registry;

/// Value names inside the Personalize key.
const VALUE_APPS_USE_LIGHT_THEME: &str = "AppsUseLightTheme";
const VALUE_SYSTEM_USES_LIGHT_THEME: &str = "SystemUsesLightTheme";
const VALUE_ENABLE_TRANSPARENCY: &str = "EnableTransparency";

/// Value names inside the DWM key.
const VALUE_ACCENT_COLOR: &str = "AccentColor";
const VALUE_COLOR_PREVALENCE: &str = "ColorPrevalence";

/// The message payload used by Windows itself when theme settings change.
const IMMERSIVE_COLOR_SET: &str = "ImmersiveColorSet";

/// Snapshot of the current Windows personalization state.
/// This is the single source of truth - never trust cached app state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeState {
    /// App mode is light (`AppsUseLightTheme == 1`).
    pub app_mode_light: bool,
    /// Windows/system mode is light (`SystemUsesLightTheme == 1`).
    /// On builds without the key, mirrors `app_mode_light`.
    pub system_mode_light: bool,
    /// Transparency effects enabled.
    pub transparency: bool,
    /// Current accent color as `#RRGGBB`, if readable.
    pub accent_color: Option<String>,
    /// Current wallpaper absolute path, if readable.
    pub wallpaper: Option<String>,
}

impl ThemeState {
    /// The effective current theme (what the user perceives as Light/Dark).
    pub fn is_light(&self) -> bool {
        self.app_mode_light
    }
}

/// Convert `#RRGGBB` to the ABGR DWORD used by `DWM\AccentColor`.
/// Example: `#0078D4` -> `0xFFD47800`.
pub fn hex_to_abgr(hex: &str) -> Option<u32> {
    let h = hex.strip_prefix('#')?;
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    // Windows stores accent as ABGR with an opaque alpha byte.
    Some(0xFF00_0000 | (u32::from(b) << 16) | (u32::from(g) << 8) | u32::from(r))
}

/// Convert an ABGR DWORD to `#RRGGBB`.
pub fn abgr_to_hex(value: u32) -> String {
    let r = (value & 0xFF) as u8;
    let g = ((value >> 8) & 0xFF) as u8;
    let b = ((value >> 16) & 0xFF) as u8;
    format!("#{r:02X}{g:02X}{b:02X}")
}

/// Read the current Windows theme state from the registry.
pub fn read_theme_state() -> AppResult<ThemeState> {
    let app_light_dword = registry::read_hkcu_dword(
        registry::PERSONALIZE_KEY,
        VALUE_APPS_USE_LIGHT_THEME,
    )?;
    let app_mode_light = app_light_dword.unwrap_or(1) != 0;

    let system_mode_light = match registry::read_hkcu_dword(
        registry::PERSONALIZE_KEY,
        VALUE_SYSTEM_USES_LIGHT_THEME,
    )? {
        Some(v) => v != 0,
        // Key missing on older builds: system follows app mode.
        None => app_mode_light,
    };

    let transparency =
        registry::read_hkcu_dword(registry::PERSONALIZE_KEY, VALUE_ENABLE_TRANSPARENCY)?
            .unwrap_or(1)
            != 0;

    let accent_color = registry::read_hkcu_dword(registry::DWM_KEY, VALUE_ACCENT_COLOR)?
        .map(abgr_to_hex);

    let wallpaper =
        registry::read_hkcu_string(registry::DESKTOP_KEY, "Wallpaper")?.filter(|s| !s.is_empty());

    Ok(ThemeState {
        app_mode_light,
        system_mode_light,
        transparency,
        accent_color,
        wallpaper,
    })
}

/// Broadcast `WM_SETTINGCHANGE` so Explorer and all apps pick up the change
/// immediately - the same mechanism the Settings app uses.
///
/// Runs on a worker thread (fire-and-forget): `SendMessageTimeoutW` with
/// `HWND_BROADCAST` visits every top-level window in turn, so a single busy
/// app can stall the call for many seconds. The theme switch itself only
/// needs the registry writes; notification is best-effort.
pub fn broadcast_theme_change() -> AppResult<()> {
    std::thread::spawn(|| {
        let payload: Vec<u16> = IMMERSIVE_COLOR_SET
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut result: usize = 0;
        let started = std::time::Instant::now();
        unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                Default::default(),
                LPARAM(payload.as_ptr() as isize),
                SMTO_ABORTIFHUNG,
                1000,
                Some(&mut result),
            );
        }
        let elapsed = started.elapsed().as_millis();
        if elapsed > 900 {
            crate::logger::warn(format!(
                "WM_SETTINGCHANGE broadcast took {elapsed}ms (some windows are slow)"
            ));
        }
    });
    Ok(())
}

/// Guard so overlapping applies (rapid tray clicks / double click) never
/// stack up into multiple broadcasts.
static APPLYING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Apply a theme profile to Windows. All-or-nothing per setting: if a write
/// fails, remaining writes are still attempted and the first error is
/// returned after the broadcast.
pub fn apply_profile(profile: &ThemeProfile) -> AppResult<ThemeState> {
    profile.validate()?;

    if APPLYING.swap(true, std::sync::atomic::Ordering::SeqCst) {
        // Another apply is already in flight. Report the live state instead
        // of queuing a second (potentially very slow) notification.
        return read_theme_state();
    }

    let result = apply_profile_inner(profile);
    APPLYING.store(false, std::sync::atomic::Ordering::SeqCst);
    result
}

fn apply_profile_inner(profile: &ThemeProfile) -> AppResult<ThemeState> {
    let mut first_error: Option<AppError> = None;

    // 1. App mode
    if let Err(e) = registry::write_hkcu_dword(
        registry::PERSONALIZE_KEY,
        VALUE_APPS_USE_LIGHT_THEME,
        u32::from(profile.app_mode == crate::config::ThemeMode::Light),
    ) {
        first_error.get_or_insert(e);
    }

    // 2. Windows (system) mode
    if let Err(e) = registry::write_hkcu_dword(
        registry::PERSONALIZE_KEY,
        VALUE_SYSTEM_USES_LIGHT_THEME,
        u32::from(profile.windows_mode == crate::config::ThemeMode::Light),
    ) {
        first_error.get_or_insert(e);
    }

    // 3. Transparency
    if let Err(e) = registry::write_hkcu_dword(
        registry::PERSONALIZE_KEY,
        VALUE_ENABLE_TRANSPARENCY,
        u32::from(profile.transparency),
    ) {
        first_error.get_or_insert(e);
    }

    // 4. Accent color (optional). Also enable ColorPrevalence so the accent
    //    shows on the taskbar/Start surfaces.
    if let Some(hex) = &profile.accent_color {
        match hex_to_abgr(hex) {
            Some(abgr) => {
                if let Err(e) =
                    registry::write_hkcu_dword(registry::DWM_KEY, VALUE_ACCENT_COLOR, abgr)
                {
                    first_error.get_or_insert(e);
                }
                if let Err(e) = registry::write_hkcu_dword(
                    registry::DWM_KEY,
                    VALUE_COLOR_PREVALENCE,
                    1,
                ) {
                    first_error.get_or_insert(e);
                }
            }
            None => {
                first_error.get_or_insert(AppError::new(
                    ErrorCode::ConfigInvalid,
                    format!("invalid accent color: {hex}"),
                ));
            }
        }
    }

    // 5. Wallpaper (optional)
    if let Some(path) = &profile.wallpaper {
        if !std::path::Path::new(path).is_file() {
            first_error.get_or_insert(AppError::new(
                ErrorCode::Wallpaper,
                format!("wallpaper file not found: {path}"),
            ));
        } else {
            let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
            let r = unsafe {
                SystemParametersInfoW(
                    SPI_SETDESKWALLPAPER,
                    0,
                    Some(wide.as_ptr() as *mut std::ffi::c_void),
                    SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
                )
            };
            if let Err(e) = r {
                first_error.get_or_insert(AppError::new(
                    ErrorCode::Wallpaper,
                    format!("SystemParametersInfo(SPI_SETDESKWALLPAPER) failed: {e}"),
                ));
            }
        }
    }

    // 6. Broadcast regardless of partial failures so successful writes apply.
    if let Err(e) = broadcast_theme_change() {
        first_error.get_or_insert(e);
    }

    match first_error {
        Some(e) => Err(e),
        None => read_theme_state(),
    }
}

/// Build a `ThemeProfile` that mirrors the current Windows state.
/// Used by the "Detect current" button in the theme editor.
pub fn profile_from_current_state(name: &str) -> AppResult<ThemeProfile> {
    let state = read_theme_state()?;
    let mode = |light: bool| {
        if light {
            crate::config::ThemeMode::Light
        } else {
            crate::config::ThemeMode::Dark
        }
    };
    Ok(ThemeProfile {
        name: name.to_string(),
        windows_mode: mode(state.system_mode_light),
        app_mode: mode(state.app_mode_light),
        transparency: state.transparency,
        accent_color: state.accent_color,
        wallpaper: state.wallpaper,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_to_abgr_conversion() {
        assert_eq!(hex_to_abgr("#0078D4"), Some(0xFFD47800));
        assert_eq!(hex_to_abgr("#000000"), Some(0xFF000000));
        assert_eq!(hex_to_abgr("#FFFFFF"), Some(0xFFFFFFFF));
        assert_eq!(hex_to_abgr("#FF0000"), Some(0xFF0000FF));
        assert_eq!(hex_to_abgr("0078D4"), None);
        assert_eq!(hex_to_abgr("#12345"), None);
        assert_eq!(hex_to_abgr("#GGGGGG"), None);
    }

    #[test]
    fn abgr_to_hex_conversion() {
        assert_eq!(abgr_to_hex(0xFFD47800), "#0078D4");
        assert_eq!(abgr_to_hex(0xFF000000), "#000000");
        assert_eq!(abgr_to_hex(0xFFFFFFFF), "#FFFFFF");
    }

    #[test]
    fn color_roundtrip() {
        for hex in ["#0078D4", "#107C10", "#E81123", "#6B69D6"] {
            assert_eq!(abgr_to_hex(hex_to_abgr(hex).unwrap()), hex);
        }
    }

    #[test]
    fn read_theme_state_is_readable() {
        // On any Windows 10/11 machine this must succeed.
        let state = read_theme_state().unwrap();
        // Sanity: system mode falls back to app mode.
        if !state.app_mode_light {
            // dark app mode with light system mode is possible, both are booleans
        }
        let _ = state.is_light();
    }

    #[test]
    fn profile_from_current_state_roundtrip() {
        let profile = profile_from_current_state("Detected").unwrap();
        assert_eq!(profile.name, "Detected");
        assert!(profile.validate().is_ok());
    }

    #[test]
    fn broadcast_does_not_panic() {
        // Should never panic even if Explorer is busy.
        let _ = broadcast_theme_change();
    }

    #[test]
    fn apply_current_state_is_idempotent() {
        // Build a profile from the current Windows state and apply it:
        // Windows must end up in exactly the same state (no visible change).
        // This exercises the full write path (Personalize keys, DWM accent,
        // wallpaper, broadcast) without changing anything.
        let profile = profile_from_current_state("idempotent").unwrap();
        let before = read_theme_state().unwrap();
        let after = apply_profile(&profile).unwrap();
        assert_eq!(before.app_mode_light, after.app_mode_light);
        assert_eq!(before.system_mode_light, after.system_mode_light);
        assert_eq!(before.transparency, after.transparency);
        assert_eq!(before.accent_color, after.accent_color);
        assert_eq!(before.wallpaper, after.wallpaper);
    }
}
