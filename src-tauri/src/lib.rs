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
    // Native singletons + signal window are set up BEFORE any Tauri
    // initialization: a second launch must exit (almost) immediately,
    // without creating a webview, and must wake the running instance.
    let first_instance = monitor::ensure_single_instance();
    monitor::start_monitor_thread();
    if let monitor::InstanceCheck::AlreadyRunning = first_instance {
        std::process::exit(0);
    }

    let app = tauri::Builder::default()
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
            commands::log_debug,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app, event| {
        // Closing the window destroys it (frees the whole WebView2 memory
        // footprint) - the app keeps running in the tray until "Exit".
        if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
            if code.is_none() {
                api.prevent_exit();
            }
        }
    });
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

    // Main window is created LAZILY (see `ensure_main_window`): a tray-only
    // start must not pay the ~400 MB WebView2 footprint. It is only created
    // when the user actually needs the UI.
    let need_window = !config.settings.onboarding_completed || !config.settings.launch_minimized;
    if need_window {
        if let Some(window) = ensure_main_window(&handle) {
            let _ = window.show();
            if !config.settings.onboarding_completed {
                let _ = window.set_focus();
                logger::info("showing onboarding window (first run)");
            }
        }
    } else {
        logger::info("starting minimized to tray (no webview)");
    }

    logger::info(format!(
        "startup complete (build {}, config: {})",
        registry::read_windows_build(),
        config_path.display()
    ));
    Ok(())
}

/// Create the main window on demand (or return the existing one).
///
/// The window is intentionally **not** declared in `tauri.conf.json`: a
/// tray-only start would otherwise spin up WebView2 (~400 MB across its
/// helper processes) even though no UI is needed. Closing the window
/// destroys it, which releases that memory again; the app keeps running in
/// the tray and recreates the window when the user asks for it.
pub fn ensure_main_window(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(window) = app.get_webview_window("main") {
        return Some(window);
    }

    let built = tauri::WebviewWindowBuilder::new(
        app,
        "main",
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("Windows Theme Switcher")
    .inner_size(940.0, 680.0)
    .min_inner_size(760.0, 560.0)
    .decorations(false)
    .center()
    .visible(false)
    .shadow(true)
    .build();

    match built {
        Ok(window) => {
            window.on_window_event(|event| {
                if let tauri::WindowEvent::Destroyed = event {
                    logger::info("main window destroyed (webview memory released)");
                }
            });
            Some(window)
        }
        Err(e) => {
            logger::error(format!("failed to create main window: {e}"));
            None
        }
    }
}
