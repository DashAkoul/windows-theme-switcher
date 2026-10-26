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
mod memory;
mod monitor;
mod registry;
mod theme;
mod tray;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use tauri::{AppHandle, Manager, WebviewWindow};

pub use error::{AppError, ErrorCode};

/// Milliseconds since the Unix epoch of the last `show_panel()` call.
///
/// Windows can deliver a `Focused(false)` event right after a window is
/// shown, before it actually receives focus (foreground-lock rules). Without
/// this guard the popup would hide itself instantly and a tray click would
/// look like it did nothing.
static LAST_SHOWN_MS: AtomicU64 = AtomicU64::new(0);

/// Grace period (ms) during which an early focus loss is ignored.
const FOCUS_GRACE_MS: u64 = 700;

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::toggle_theme,
            commands::apply_profile,
            commands::save_settings,
            commands::open_windows_settings,
            commands::hide_panel,
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
    });

    // Tray icon + context menu.
    tray::build(&handle).map_err(|e| {
        logger::error(format!("failed to build tray icon: {e}"));
        e.to_string()
    })?;

    // The panel is created LAZILY (see `ensure_panel`): a tray-only start
    // must not pay the ~400 MB WebView2 footprint. It is only created when
    // the user opens it, and destroyed again when it loses focus/idles.
    logger::info("starting in tray (panel is created on demand)");

    logger::info(format!(
        "startup complete (build {}, config: {})",
        registry::read_windows_build(),
        config_path.display()
    ));

    // Nothing else is going to run until the user clicks the tray icon, so
    // hand the resident pages back once the startup burst is over.
    memory::trim_soon(1500);
    Ok(())
}

/// Panel geometry (a compact popup, not a full application window).
const PANEL_W: f64 = 340.0;
/// Panel height: tall enough for the cards, the toggle, the link, a two-line
/// error message and the footer, without leaving a big empty gap.
const PANEL_H: f64 = 232.0;

/// Create the panel on demand (or return the existing one).
///
/// The window is intentionally **not** declared in `tauri.conf.json`: a
/// tray-only start would otherwise spin up WebView2 (~400 MB across its
/// helper processes) even though no UI is needed. The panel hides when it
/// loses focus and is destroyed after a short idle period, which releases
/// that memory again; the app keeps running in the tray.
pub fn ensure_panel(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(window) = app.get_webview_window("main") {
        return Some(window);
    }

    let built = tauri::WebviewWindowBuilder::new(
        app,
        "main",
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("Windows Theme Switcher")
    .inner_size(PANEL_W, PANEL_H)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(false)
    .shadow(true)
    .build();

    match built {
        Ok(window) => {
            // Popup behaviour: hide as soon as the panel loses focus, and
            // release the webview entirely after a short idle period.
            let app_for_events = app.clone();
            let window_for_events = window.clone();
            window.on_window_event(move |event| match event {
                tauri::WindowEvent::Focused(false) => {
                    // Ignore the spurious focus loss that can arrive while the
                    // window is still becoming foreground right after `show()`.
                    let since_shown = now_ms().saturating_sub(LAST_SHOWN_MS.load(Ordering::Relaxed));
                    if since_shown < FOCUS_GRACE_MS {
                        logger::info(format!(
                            "ignoring early focus loss ({since_shown}ms after show)"
                        ));
                        return;
                    }
                    let _ = window_for_events.hide();
                    logger::info("panel hidden (focus lost)");
                    schedule_idle_destroy(app_for_events.clone());
                    // Hidden but not destroyed yet: page out what the webview
                    // touched so the process does not sit at ~45 MB for the
                    // two minutes before the idle destroy fires.
                    memory::trim_soon(500);
                }
                tauri::WindowEvent::Destroyed => {
                    logger::info("panel destroyed (webview memory released)");
                    // The WebView2 runtime pages are still mapped in this
                    // process; page them out so the tray drops back to the
                    // kilobyte range instead of ~45 MB.
                    memory::trim_soon(300);
                }
                _ => {}
            });
            Some(window)
        }
        Err(e) => {
            logger::error(format!("failed to create panel: {e}"));
            None
        }
    }
}

/// How long a hidden panel stays alive before its webview is released.
const IDLE_DESTROY_SECS: u64 = 120;

/// True while an idle-destroy timer is pending, so repeated hide events do
/// not spawn one thread each.
static DESTROY_SCHEDULED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Destroy the hidden panel after a while so the WebView2 memory (which is
/// large compared to the rest of the app) does not stay resident forever.
pub(crate) fn schedule_idle_destroy(app: AppHandle) {
    if DESTROY_SCHEDULED.swap(true, Ordering::SeqCst) {
        return; // a timer is already pending
    }
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(IDLE_DESTROY_SECS));
        DESTROY_SCHEDULED.store(false, Ordering::SeqCst);
        let app_for_main = app.clone();
        let _ = app.run_on_main_thread(move || {
            if let Some(window) = app_for_main.get_webview_window("main") {
                if !window.is_visible().unwrap_or(false) {
                    let _ = window.close();
                    logger::info("panel closed after idle (memory released)");
                }
            }
        });
    });
}

/// Show the panel as a tray popup: positioned above the taskbar in the
/// bottom-right corner, focused, and closed again when it loses focus.
pub fn show_panel(app: &AppHandle) {
    let Some(window) = ensure_panel(app) else {
        return;
    };

    // Bottom-right of the work area, a few pixels above the taskbar.
    if let Ok(Some(monitor)) = window.current_monitor() {
        let scale = monitor.scale_factor();
        let area = monitor.work_area();
        let x = area.position.x as f64 + area.size.width as f64 - PANEL_W * scale - 12.0 * scale;
        let y = area.position.y as f64 + area.size.height as f64 - PANEL_H * scale - 12.0 * scale;
        let _ = window.set_position(tauri::LogicalPosition::new(x / scale, y / scale));
    }

    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
    LAST_SHOWN_MS.store(now_ms(), Ordering::Relaxed);
    logger::info("panel shown");
}
