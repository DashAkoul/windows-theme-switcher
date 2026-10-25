//! Windows Theme Switcher - library crate.
//!
//! Module map:
//! - [`config`]   - profiles, settings, atomic JSON persistence
//! - [`registry`] - HKCU/HKLM registry helpers (per-user, no admin)
//! - [`theme`]    - theme detection & application (registry + broadcast)
//! - [`monitor`]  - WM_SETTINGCHANGE listener + single-instance signaling
//! - [`tray`]     - system tray icon and context menu
//! - [`commands`] - Tauri commands for the React frontend
//! - [`autostart`]- "Start with Windows" via the Run key
//! - [`logger`]   - append-only file logger

mod autostart;
mod commands;
mod config;
mod error;
mod logger;
mod monitor;
mod registry;
mod theme;
mod tray;

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use tauri::{AppHandle, Manager, WebviewWindow};

pub use error::{AppError, ErrorCode};

/// Global app handle, initialized in `setup()`. Used by the native message
/// pump thread (which has no other way to reach Tauri).
pub static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

/// Shared application state managed by Tauri.
pub struct AppState {
    /// The loaded configuration (source of truth for the UI).
    pub config: Mutex<config::AppConfig>,
    /// Absolute path of `config.json`.
    pub config_path: PathBuf,
    /// The tray icon (kept alive for the process lifetime).
    pub tray: Mutex<Option<tauri::tray::TrayIcon>>,
    /// Route the UI should open with when it next mounts.
    pub pending_route: Mutex<Option<String>>,
}

/// Entry point called by `main.rs`.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::toggle_theme,
            commands::apply_profile,
            commands::save_config,
            commands::set_autostart,
            commands::detect_current_profile,
            commands::open_windows_settings,
            commands::open_url,
            commands::get_diagnostics,
            commands::clear_pending_route,
            commands::reset_config,
            commands::config_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Global handle for native threads.
    let handle = app.handle().clone();
    let _ = APP_HANDLE.set(handle.clone());

    // Paths & logger.
    let config_dir = app.path().app_config_dir()?;
    let config_path = config::AppConfig::config_path(&config_dir);
    logger::init(config_dir.join("app.log"));
    logger::info("=== Windows Theme Switcher starting ===");

    // Single instance: if another copy is already running, wake it and exit.
    if let monitor::InstanceCheck::AlreadyRunning = monitor::ensure_single_instance() {
        logger::info("another instance is already running; exiting");
        std::process::exit(0);
    }

    // Load configuration.
    let config = config::AppConfig::load(&config_path).map_err(|e| {
        logger::error(format!("failed to load config: {e}"));
        e.to_string()
    })?;

    // Sync autostart registry state with the user's setting.
    if let Ok(exe) = std::env::current_exe() {
        let exe_path = exe.to_string_lossy().to_string();
        if autostart::is_enabled() != config.settings.start_with_windows {
            if let Err(e) = autostart::set_enabled(&exe_path, config.settings.start_with_windows) {
                logger::warn(format!("failed to sync autostart: {e}"));
            }
        }
    }

    // Manage shared state.
    app.manage(AppState {
        config: Mutex::new(config.clone()),
        config_path: config_path.clone(),
        tray: Mutex::new(None),
        pending_route: Mutex::new(None),
    });

    // Tray icon + context menu.
    tray::build(&handle).map_err(|e| {
        logger::error(format!("failed to build tray icon: {e}"));
        e.to_string()
    })?;

    // Native message pump (external theme changes + instance wake-up).
    monitor::start_monitor_thread();

    // Main window: effects, visibility and close-to-tray behavior.
    if let Some(window) = app.get_webview_window("main") {
        apply_window_effects(&window, &config.settings);
        setup_window_behavior(window.clone());

        if !config.settings.onboarding_completed {
            let _ = window.show();
            let _ = window.set_focus();
            logger::info("showing onboarding window (first run)");
        } else if !config.settings.launch_minimized {
            let _ = window.show();
        } else {
            logger::info("starting minimized to tray");
        }
    }

    logger::info(format!(
        "startup complete (build {}, config: {})",
        registry::read_windows_build(),
        config_path.display()
    ));
    Ok(())
}

/// Apply Mica (Windows 11) or Acrylic (Windows 10) to the main window when
/// transparency is enabled in settings. Failures are non-fatal.
fn apply_window_effects(window: &WebviewWindow, settings: &config::Settings) {
    if !settings.transparency {
        return;
    }
    let dark = match settings.ui_theme {
        config::UiTheme::Dark => true,
        config::UiTheme::Light => false,
        config::UiTheme::System => !crate::theme::read_theme_state()
            .map(|s| s.app_mode_light)
            .unwrap_or(false),
    };

    let build = registry::read_windows_build();
    if build >= 22000 {
        if let Err(e) = window_vibrancy::apply_mica(window, Some(dark)) {
            logger::warn(format!("apply_mica failed: {e}"));
        }
    } else if let Err(e) = window_vibrancy::apply_acrylic(window, Some((0x20, 0x20, 0x20, 0xFF))) {
        logger::warn(format!("apply_acrylic failed: {e}"));
    }
}

/// Close button minimizes to tray instead of quitting; the tray "Exit"
/// menu item is the only way to quit.
fn setup_window_behavior(window: WebviewWindow) {
    let window_for_events = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = window_for_events.hide();
            logger::info("main window hidden to tray");
        }
    });
}
