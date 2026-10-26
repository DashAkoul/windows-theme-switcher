//! Tauri commands exposed to the panel UI.
//!
//! Each `#[tauri::command]` is a thin wrapper around an `*_inner` function
//! that takes `&AppHandle` so the same logic can be called from tray event
//! handlers (which have no `State` parameter).

use std::time::Instant;

use tauri::{AppHandle, Emitter, Manager};

use crate::config::{AppConfig, DARK_PROFILE_ID, LIGHT_PROFILE_ID};
use crate::error::{AppError, AppResult, ErrorCode};
use crate::theme::{self, ThemeState};
use crate::AppState;

/// State returned to the panel on startup.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStateResponse {
    pub config: AppConfig,
    pub current: ThemeState,
    pub is_windows11: bool,
    pub version: String,
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
    apply_profile_inner(app, target_id)
}

pub fn apply_profile_inner(app: &AppHandle, id: &str) -> AppResult<ThemeState> {
    let app_state = app.state::<AppState>();
    let profile = {
        let config = app_state.config.lock().unwrap_or_else(|e| e.into_inner());
        config.profile(id).cloned().ok_or_else(|| {
            AppError::new(
                ErrorCode::ProfileNotFound,
                format!("profile '{id}' not found"),
            )
        })?
    };

    let started = Instant::now();
    let new_state = theme::apply_profile(&profile).inspect_err(|e| {
        crate::logger::error(format!("apply_profile({id}) failed: {e}"));
    })?;
    crate::logger::info(format!(
        "applied profile '{id}' in {}ms",
        started.elapsed().as_millis()
    ));

    {
        let mut config = app_state.config.lock().unwrap_or_else(|e| e.into_inner());
        config.last_active_profile = Some(id.to_string());
        let _ = config.save(&app_state.config_path);
    }

    // Remember this state so the echo of our own WM_SETTINGCHANGE broadcast
    // does not trigger a second tray rebuild.
    crate::monitor::note_applied_state(&new_state);
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

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Get the config + current Windows theme state.
#[tauri::command]
pub fn get_app_state(app: AppHandle) -> AppResult<AppStateResponse> {
    let app_state = app.state::<AppState>();
    let config = app_state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();

    // A failed registry read must never brick the panel: fall back to a
    // neutral state and keep going.
    let current = theme::read_theme_state().unwrap_or_else(|e| {
        crate::logger::warn(format!("read_theme_state failed, using fallback: {e}"));
        ThemeState {
            app_mode_light: true,
            system_mode_light: true,
            transparency: true,
            accent_color: None,
            wallpaper: None,
        }
    });

    Ok(AppStateResponse {
        config,
        current,
        is_windows11: crate::registry::read_windows_build() >= 22000,
        version: env!("CARGO_PKG_VERSION").to_string(),
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

/// Persist settings (only "start with Windows" and "check updates" exist).
#[tauri::command]
pub fn save_settings(app: AppHandle, settings: crate::config::Settings) -> AppResult<AppConfig> {
    let app_state = app.state::<AppState>();
    let previous = app_state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();

    let mut config = previous.clone();
    config.settings = settings;
    config.save(&app_state.config_path)?;

    // Sync autostart with the (possibly changed) setting.
    if let Ok(exe) = std::env::current_exe() {
        let exe_path = exe.to_string_lossy().to_string();
        crate::autostart::set_enabled(&exe_path, config.settings.start_with_windows)?;
    }

    *app_state.config.lock().unwrap_or_else(|e| e.into_inner()) = config.clone();

    if previous.settings.start_with_windows != config.settings.start_with_windows {
        let current = theme::read_theme_state().unwrap_or_else(|_| ThemeState {
            app_mode_light: true,
            system_mode_light: true,
            transparency: true,
            accent_color: None,
            wallpaper: None,
        });
        crate::tray::refresh(&app, &current);
    }

    Ok(config)
}

/// Open the Windows personalization settings page (colors).
#[tauri::command]
pub fn open_windows_settings() -> AppResult<()> {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::w;

    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            w!("ms-settings:personalization-colors"),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    let code = result.0 as isize;
    if code <= 32 {
        return Err(AppError::new(
            ErrorCode::WindowsApi,
            format!("ShellExecuteW failed with code {code}"),
        ));
    }
    Ok(())
}

/// Hide the panel (the webview is released after a short idle period).
#[tauri::command]
pub fn hide_panel(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    // Hiding from the ✕ button may not produce a focus event, so make sure
    // the idle timer that releases the webview is armed here too.
    crate::schedule_idle_destroy(app);
}
