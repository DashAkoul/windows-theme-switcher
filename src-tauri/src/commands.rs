//! Tauri commands exposed to the React frontend.
//!
//! Each `#[tauri::command]` is a thin wrapper around an `*_inner` function
//! that takes `&AppHandle` so the same logic can be called from tray event
//! handlers (which have no `State` parameter).

use std::path::PathBuf;
use std::time::Instant;

use tauri::{AppHandle, Emitter, Manager};

use crate::config::{AppConfig, ThemeProfile, DARK_PROFILE_ID, LIGHT_PROFILE_ID};
use crate::error::{AppError, AppResult, ErrorCode};
use crate::theme::{self, ThemeState};
use crate::AppState;

/// Full application state returned to the frontend on startup.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStateResponse {
    pub config: AppConfig,
    pub current: ThemeState,
    pub os_build: u32,
    pub is_windows11: bool,
    pub os_display: String,
    pub exe_path: String,
    pub log_path: String,
    /// Route the UI should open with (set when the tray asks for a view).
    pub pending_route: Option<String>,
}

fn os_display() -> String {
    let build = crate::registry::read_windows_build();
    let display = crate::registry::read_display_version().unwrap_or_default();
    if display.is_empty() {
        format!("Windows (build {build})")
    } else {
        format!("Windows 11 {display} (build {build})")
            .replace("Windows 11", if build >= 22000 { "Windows 11" } else { "Windows 10" })
    }
}

// ---------------------------------------------------------------------------
// Inner implementations (callable from tray handlers)
// ---------------------------------------------------------------------------

pub fn toggle_theme_inner(app: &AppHandle) -> AppResult<ThemeState> {
    let current = theme::read_theme_state()?;
    let target_id = if current.is_light() {
        DARK_PROFILE_ID
    } else {
        LIGHT_PROFILE_ID
    };

    let app_state = app.state::<AppState>();
    let profile = {
        let config = app_state.config.lock().unwrap_or_else(|e| e.into_inner());
        config
            .profile(target_id)
            .cloned()
            .ok_or_else(|| AppError::new(ErrorCode::ProfileNotFound, format!("profile '{target_id}' not found")))?
    };

    let started = Instant::now();
    let new_state = theme::apply_profile(&profile).inspect_err(|e| {
        crate::logger::error(format!("apply_profile({target_id}) failed: {e}"));
    })?;
    crate::logger::info(format!(
        "applied profile '{target_id}' in {}ms",
        started.elapsed().as_millis()
    ));

    // Persist last active profile (best effort - never blocks the switch).
    {
        let mut config = app_state.config.lock().unwrap_or_else(|e| e.into_inner());
        config.last_active_profile = Some(target_id.to_string());
        let _ = config.save(&app_state.config_path);
    }

    crate::tray::refresh(app, &new_state);
    let _ = app.emit("theme-applied", &new_state);
    Ok(new_state)
}

pub fn apply_profile_inner(app: &AppHandle, id: &str) -> AppResult<ThemeState> {
    let app_state = app.state::<AppState>();
    let profile = {
        let config = app_state.config.lock().unwrap_or_else(|e| e.into_inner());
        config
            .profile(id)
            .cloned()
            .ok_or_else(|| AppError::new(ErrorCode::ProfileNotFound, format!("profile '{id}' not found")))?
    };

    let new_state = theme::apply_profile(&profile)?;

    {
        let mut config = app_state.config.lock().unwrap_or_else(|e| e.into_inner());
        config.last_active_profile = Some(id.to_string());
        let _ = config.save(&app_state.config_path);
    }

    crate::tray::refresh(app, &new_state);
    let _ = app.emit("theme-applied", &new_state);
    Ok(new_state)
}

pub fn toggle_autostart_inner(app: &AppHandle) -> AppResult<bool> {
    let app_state = app.state::<AppState>();
    let exe = std::env::current_exe()
        .map_err(|e| AppError::new(ErrorCode::Autostart, e.to_string()))?;
    let exe_path = exe.to_string_lossy().to_string();

    let new_value = {
        let mut config = app_state.config.lock().unwrap_or_else(|e| e.into_inner());
        config.settings.start_with_windows = !config.settings.start_with_windows;
        config.settings.start_with_windows
    };

    crate::autostart::set_enabled(&exe_path, new_value)?;
    let _ = app_state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .save(&app_state.config_path);

    crate::logger::info(format!("autostart set to {new_value}"));
    Ok(new_value)
}

pub fn show_window_with_route(app: &AppHandle, route: &str) {
    let app_state = app.state::<AppState>();
    if let Ok(mut pending) = app_state.pending_route.lock() {
        *pending = Some(route.to_string());
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
    let _ = app.emit("tray://navigate", route);
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Get the full app state (config + current Windows theme + OS info).
#[tauri::command]
pub fn get_app_state(app: AppHandle) -> AppResult<AppStateResponse> {
    let app_state = app.state::<AppState>();
    let config = app_state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let pending_route = app_state
        .pending_route
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let current = theme::read_theme_state()?;
    let build = crate::registry::read_windows_build();

    Ok(AppStateResponse {
        config,
        current,
        os_build: build,
        is_windows11: build >= 22000,
        os_display: os_display(),
        exe_path: std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        log_path: crate::logger::log_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        pending_route,
    })
}

/// Toggle between the Light and Dark profiles.
#[tauri::command]
pub fn toggle_theme(app: AppHandle) -> AppResult<ThemeState> {
    toggle_theme_inner(&app)
}

/// Apply a specific profile by id.
#[tauri::command]
pub fn apply_profile(app: AppHandle, id: String) -> AppResult<ThemeState> {
    apply_profile_inner(&app, &id)
}

/// Save the full configuration (called from the settings UI).
#[tauri::command]
pub fn save_config(app: AppHandle, config: AppConfig) -> AppResult<AppConfig> {
    config.validate()?;
    let app_state = app.state::<AppState>();
    config.save(&app_state.config_path)?;

    // Sync autostart with the (possibly changed) setting.
    if let Ok(exe) = std::env::current_exe() {
        let exe_path = exe.to_string_lossy().to_string();
        crate::autostart::set_enabled(&exe_path, config.settings.start_with_windows)?;
    }

    *app_state.config.lock().unwrap_or_else(|e| e.into_inner()) = config.clone();
    let current = theme::read_theme_state().unwrap_or_else(|_| fallback_state());
    crate::tray::refresh(&app, &current);
    Ok(config)
}

/// Neutral fallback state used only if the registry cannot be read.
fn fallback_state() -> ThemeState {
    ThemeState {
        app_mode_light: true,
        system_mode_light: true,
        transparency: true,
        accent_color: None,
        wallpaper: None,
    }
}

/// Enable/disable "Start with Windows".
#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> AppResult<bool> {
    let app_state = app.state::<AppState>();
    let exe = std::env::current_exe()
        .map_err(|e| AppError::new(ErrorCode::Autostart, e.to_string()))?;
    crate::autostart::set_enabled(&exe.to_string_lossy(), enabled)?;
    {
        let mut config = app_state.config.lock().unwrap_or_else(|e| e.into_inner());
        config.settings.start_with_windows = enabled;
        let _ = config.save(&app_state.config_path);
    }
    Ok(enabled)
}

/// Build a profile from the current Windows state ("Detect current").
#[tauri::command]
pub fn detect_current_profile() -> AppResult<ThemeProfile> {
    theme::profile_from_current_state("Detected")
}

/// Open the Windows personalization settings page (colors).
#[tauri::command]
pub fn open_windows_settings() -> AppResult<()> {
    open_url("ms-settings:personalization-colors".to_string())
}

/// Open an arbitrary URL with the default handler (used for the GitHub link).
#[tauri::command]
pub fn open_url(url: String) -> AppResult<()> {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::w;

    let wide: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            windows::core::PCWSTR(wide.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    // ShellExecuteW returns an HINSTANCE; values <= 32 indicate failure.
    let code = result.0 as isize;
    if code <= 32 {
        return Err(AppError::new(
            ErrorCode::WindowsApi,
            format!("ShellExecuteW failed with code {code}"),
        ));
    }
    Ok(())
}

/// Collect diagnostic information as JSON (safe to copy/share).
#[tauri::command]
pub fn get_diagnostics(app: AppHandle) -> String {
    let app_state = app.state::<AppState>();
    let config = app_state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let current = theme::read_theme_state().ok();
    let build = crate::registry::read_windows_build();

    let diagnostics = serde_json::json!({
        "app": {
            "name": "Windows Theme Switcher",
            "version": env!("CARGO_PKG_VERSION"),
            "exe": std::env::current_exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
        },
        "os": {
            "build": build,
            "windows11": build >= 22000,
            "displayVersion": crate::registry::read_display_version(),
        },
        "currentTheme": current,
        "registry": {
            "appsUseLightTheme": crate::registry::read_hkcu_dword(crate::registry::PERSONALIZE_KEY, "AppsUseLightTheme").ok(),
            "systemUsesLightTheme": crate::registry::read_hkcu_dword(crate::registry::PERSONALIZE_KEY, "SystemUsesLightTheme").ok(),
            "enableTransparency": crate::registry::read_hkcu_dword(crate::registry::PERSONALIZE_KEY, "EnableTransparency").ok(),
            "accentColor": crate::registry::read_hkcu_dword(crate::registry::DWM_KEY, "AccentColor").ok(),
            "colorPrevalence": crate::registry::read_hkcu_dword(crate::registry::DWM_KEY, "ColorPrevalence").ok(),
            "autostart": crate::autostart::is_enabled(),
        },
        "configPath": app_state.config_path.to_string_lossy(),
        "config": config,
        "logTail": crate::logger::tail(50),
    });

    serde_json::to_string_pretty(&diagnostics).unwrap_or_else(|_| "{}".to_string())
}

/// Clear the pending route after the UI consumed it.
#[tauri::command]
pub fn clear_pending_route(app: AppHandle) -> AppResult<()> {
    let app_state = app.state::<AppState>();
    if let Ok(mut pending) = app_state.pending_route.lock() {
        *pending = None;
    }
    Ok(())
}

/// Reset configuration to defaults (keeps the current Windows theme).
#[tauri::command]
pub fn reset_config(app: AppHandle) -> AppResult<AppConfig> {
    let app_state = app.state::<AppState>();
    let config = AppConfig::default();
    config.save(&app_state.config_path)?;

    // Sync autostart to the reset value.
    if let Ok(exe) = std::env::current_exe() {
        let _ = crate::autostart::set_enabled(
            &exe.to_string_lossy(),
            config.settings.start_with_windows,
        );
    }

    *app_state.config.lock().unwrap_or_else(|e| e.into_inner()) = config.clone();
    crate::tray::refresh(&app, &theme::read_theme_state().unwrap_or_else(|_| fallback_state()));
    Ok(config)
}

/// Return the config file path (shown in Advanced settings).
#[tauri::command]
pub fn config_path(app: AppHandle) -> PathBuf {
    app.state::<AppState>().config_path.clone()
}

/// Write a message from the frontend into the app log (used by the UI
/// self-test to report layout/CSS diagnostics).
#[tauri::command]
pub fn log_debug(message: String) {
    crate::logger::info(format!("[ui] {message}"));
}
