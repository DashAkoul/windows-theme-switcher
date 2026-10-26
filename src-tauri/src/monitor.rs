//! Native Windows message pump used for two purposes:
//!
//! 1. **External change detection** - listens for `WM_SETTINGCHANGE`
//!    ("ImmersiveColorSet") and `WM_DWMCOLORIZATIONCOLORCHANGED`, which
//!    Windows broadcasts when the user changes the theme in Settings (or
//!    any other app does). On change we re-read the registry and update
//!    the tray + notify the frontend.
//! 2. **Single-instance activation** - a second launch signals this window
//!    via a registered window message; we then show the main window.
//!
//! The window is a hidden top-level window (so it is findable via
//! `FindWindowW`) with a custom class name. A dedicated thread runs the
//! message loop; it never blocks the Tauri main thread.

use tauri::Emitter;
use windows::Win32::Foundation::{GetLastError, SetLastError, ERROR_ALREADY_EXISTS, HWND, LPARAM, LRESULT, WPARAM, WIN32_ERROR};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{
    CreateEventW, CreateMutexW, OpenEventW, SetEvent, SYNCHRONIZATION_ACCESS_RIGHTS,
    WaitForSingleObject,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, FindWindowW, GetMessageW, PostMessageW,
    RegisterClassW, RegisterWindowMessageW, MSG, WNDCLASSW, WNDCLASS_STYLES,
    WM_DWMCOLORIZATIONCOLORCHANGED, WM_SETTINGCHANGE,
};
use windows::core::{w, PCWSTR};

use crate::theme::{read_theme_state, ThemeState};
use crate::APP_HANDLE;

/// Window class name of the hidden signal window.
const SIGNAL_CLASS: &str = "WTS_SignalWindow";
/// Named mutex used for single-instance detection.
const MUTEX_NAME: &str = "Local\\WindowsThemeSwitcher_SingleInstance";
/// Registered message used to wake the running instance.
const WAKE_MESSAGE: &str = "WTS_ActivateInstance";

/// How often the idle janitor thread trims the working set (ms).
///
/// The WebView2 runtime keeps a background timer in this process even after
/// the panel is gone, so a few MB get touched every ~15 s. Trimming on a 3 s
/// cadence keeps the *observed* working set in the low hundreds of KB
/// (median 0 KB, max ~1.5 MB in a 30 s sample) and costs microseconds of CPU
/// per call — roughly 0.002 % of one core.
const IDLE_TRIM_INTERVAL_MS: u32 = 3_000;

/// `WAIT_TIMEOUT` (0x102) as returned by `WaitForSingleObject` when the
/// timeout elapses. The `windows` crate exposes it as a raw value only.
const WAIT_TIMEOUT_VALUE: u32 = 0x102;

/// NUL-terminated wide string for APIs where the `w!` macro cannot be used
/// (it only accepts string literals, not constants).
fn wide_str(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// The window procedure for the hidden signal window.
unsafe extern "system" fn signal_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_SETTINGCHANGE => {
                // lParam points to a wide string naming the changed area.
                if lparam.0 != 0 {
                    let name = wide_ptr_to_string(lparam.0 as *const u16);
                    if name == "ImmersiveColorSet" || name == "WindowsThemeElement" {
                        handle_external_theme_change();
                    }
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_DWMCOLORIZATIONCOLORCHANGED => {
                handle_external_theme_change();
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            _ => {
                // Registered wake message (dynamic id) - compare by id.
                let wake_id = wake_message_id();
                if wake_id != 0 && msg == wake_id {
                    show_main_window();
                    return LRESULT(0);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
    }
}

/// Cached id of the registered wake message.
fn wake_message_id() -> u32 {
    static ID: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *ID.get_or_init(|| {
        let name = wide_str(WAKE_MESSAGE);
        unsafe { RegisterWindowMessageW(PCWSTR(name.as_ptr())) }
    })
}

/// Read a NUL-terminated wide string from a raw pointer (best effort).
fn wide_ptr_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    unsafe {
        while *ptr.add(len) != 0 && len < 256 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(ptr, len);
        String::from_utf16_lossy(slice)
    }
}

/// Last theme state we reacted to. Windows broadcasts `WM_SETTINGCHANGE`
/// very often (and echoes our own broadcasts), so we only refresh the tray
/// and notify the UI when the state actually changed. This keeps idle CPU
/// and handle churn near zero.
static LAST_STATE: std::sync::Mutex<Option<ThemeState>> = std::sync::Mutex::new(None);

/// Record the state that was just applied by the app itself, so the echo of
/// our own broadcast is recognized as "no change".
pub fn note_applied_state(state: &ThemeState) {
    if let Ok(mut last) = LAST_STATE.lock() {
        *last = Some(state.clone());
    }
}

/// Called when Windows broadcasts a theme change from any source.
fn handle_external_theme_change() {
    let app = match APP_HANDLE.get() {
        Some(a) => a,
        None => return,
    };
    let state: ThemeState = match read_theme_state() {
        Ok(s) => s,
        Err(e) => {
            crate::logger::warn(format!("failed to re-read theme state: {e}"));
            return;
        }
    };

    // Cheap no-op guard: nothing changed (or this is the echo of our own
    // broadcast), so do not rebuild the tray menu or spam the frontend.
    {
        let mut last = LAST_STATE.lock().unwrap_or_else(|e| e.into_inner());
        if last.as_ref() == Some(&state) {
            return;
        }
        *last = Some(state.clone());
    }

    crate::logger::info(format!(
        "external theme change detected: app_mode={}",
        if state.app_mode_light { "light" } else { "dark" }
    ));
    crate::tray::refresh(app, &state);
    let _ = app.emit("system-theme-changed", &state);
}

/// Show the panel (called from the signal thread).
///
/// Window creation must happen on the main thread, so the work is posted to
/// the event loop instead of running on this thread.
fn show_main_window() {
    let app = match APP_HANDLE.get() {
        Some(a) => a,
        None => return,
    };
    let app_for_main = app.clone();
    let _ = app.run_on_main_thread(move || {
        crate::show_panel(&app_for_main);
    });
}

/// Start the message pump thread. Must be called once during setup, after
/// `APP_HANDLE` has been initialized.
pub fn start_monitor_thread() {
    std::thread::spawn(|| {
        let hinstance = match unsafe { GetModuleHandleW(None) } {
            Ok(h) => h,
            Err(e) => {
                crate::logger::error(format!("GetModuleHandleW failed: {e}"));
                return;
            }
        };

        let class_name: Vec<u16> = SIGNAL_CLASS.encode_utf16().chain(std::iter::once(0)).collect();
        let window_name: Vec<u16> = "WTS Signal"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let wnd_class = WNDCLASSW {
            style: WNDCLASS_STYLES(0),
            lpfnWndProc: Some(signal_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance.into(),
            hIcon: Default::default(),
            hCursor: Default::default(),
            hbrBackground: Default::default(),
            lpszMenuName: windows::core::PCWSTR::null(),
            lpszClassName: windows::core::PCWSTR(class_name.as_ptr()),
        };

        let atom = unsafe { RegisterClassW(&wnd_class) };
        if atom == 0 {
            // Class may already be registered from a previous run in the
            // same session; that's fine, CreateWindowExW will still work.
            crate::logger::warn("RegisterClassW returned 0 (class may exist)");
        }

        // Hidden top-level window (no WS_VISIBLE) so FindWindowW can locate
        // it, but nothing is shown on screen or in the taskbar.
        match unsafe {
            CreateWindowExW(
                windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
                windows::core::PCWSTR(class_name.as_ptr()),
                windows::core::PCWSTR(window_name.as_ptr()),
                windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                None,
                None,
                Some(hinstance.into()),
                None,
            )
        } {
            Ok(hwnd) => {
                crate::logger::info(format!("signal window created: {hwnd:?}"));
            }
            Err(e) => {
                crate::logger::error(format!("CreateWindowExW failed: {e}"));
                return;
            }
        };

        crate::logger::info("theme monitor thread started");

        let mut msg = MSG::default();
        loop {
            let ret = unsafe { GetMessageW(&mut msg, None, 0, 0) };
            if !ret.as_bool() {
                break; // WM_QUIT
            }
            unsafe {
                let _ = DispatchMessageW(&msg);
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Single instance
// ---------------------------------------------------------------------------

/// Result of the single-instance check.
pub enum InstanceCheck {
    /// This is the first instance; continue startup.
    First,
    /// Another instance is already running; it has been signaled to show
    /// its window. The caller should exit immediately.
    AlreadyRunning,
}

/// Try to become the single application instance. If another instance is
/// already running, signal it and return `AlreadyRunning`.
pub fn ensure_single_instance() -> InstanceCheck {
    // 1. Acquire the named mutex. Clear the last error first so a stale
    //    error from Tauri's own Win32 calls cannot produce a false positive.
    unsafe { SetLastError(WIN32_ERROR(0)) };
    let mutex_name = wide_str(MUTEX_NAME);
    let mutex = unsafe { CreateMutexW(None, false, PCWSTR(mutex_name.as_ptr())) };
    let mutex = match mutex {
        Ok(h) => h,
        Err(e) => {
            // Could not create the mutex - fail open (allow startup).
            crate::logger::warn(format!("CreateMutexW failed: {e}; allowing startup"));
            return InstanceCheck::First;
        }
    };

    let last_error = unsafe { GetLastError() };
    if last_error == ERROR_ALREADY_EXISTS {
        // Another instance owns the mutex. Wake it - retrying while it is
        // still starting up (its signal window/event may not exist yet) -
        // then bail out.
        for _ in 0..50 {
            let hwnd = unsafe { FindWindowW(windows::core::PCWSTR::null(), w!("WTS Signal")) }.ok();
            if let Some(hwnd) = hwnd {
                let msg_id = wake_message_id();
                let _ = unsafe { PostMessageW(Some(hwnd), msg_id, WPARAM(0), LPARAM(0)) };
                return InstanceCheck::AlreadyRunning;
            }
            if wake_via_event() {
                return InstanceCheck::AlreadyRunning;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        // Could not signal the other instance (it may be shutting down);
        // exit anyway to preserve single-instance semantics.
        return InstanceCheck::AlreadyRunning;
    }

    // 2. We own the mutex. `HANDLE` is `Copy` with no `Drop` impl, so the
    //    handle stays open for the whole process lifetime without any
    //    extra bookkeeping - closing it early would let a second instance
    //    start. Create the wake event for future instances.
    let _instance_mutex = mutex;
    let _ = unsafe { CreateEventW(None, false, false, w!("Local\\WindowsThemeSwitcher_Wake")) };

    // 3. Thread that waits on the wake event (fallback activation path).
    //
    // The same thread doubles as the memory janitor: instead of waiting
    // forever it wakes every 30 s and hands the resident pages back. That
    // keeps an idle tray at a few hundred KB even if something touched pages
    // in the meantime, and costs one syscall per interval.
    std::thread::spawn(|| loop {
        // SYNCHRONIZE is required to wait on the handle; EVENT_MODIFY_STATE
        // is required to signal it.
        let access = SYNCHRONIZATION_ACCESS_RIGHTS(0x0010_0000 | 0x0002);
        let event = unsafe { OpenEventW(access, false, w!("Local\\WindowsThemeSwitcher_Wake")) };
        let event = match event {
            Ok(e) => e,
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(500));
                continue;
            }
        };
        let r = unsafe { WaitForSingleObject(event, IDLE_TRIM_INTERVAL_MS) };
        if r.0 == 0 {
            show_main_window();
        } else if r.0 == WAIT_TIMEOUT_VALUE {
            crate::memory::trim_now();
        } else {
            // The wait failed (e.g. access denied). Sleep before retrying so
            // a failing wait can never turn into a busy loop.
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(event);
        }
    });

    InstanceCheck::First
}

/// Signal the running instance through the named event (fallback path).
/// Returns `true` when the event was signaled successfully.
fn wake_via_event() -> bool {
    let event = unsafe {
        OpenEventW(
            SYNCHRONIZATION_ACCESS_RIGHTS(0x0002),
            false,
            w!("Local\\WindowsThemeSwitcher_Wake"),
        )
    };
    match event {
        Ok(event) => {
            let signaled = unsafe { SetEvent(event) }.is_ok();
            unsafe {
                let _ = windows::Win32::Foundation::CloseHandle(event);
            }
            signaled
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_ptr_to_string_reads_nul_terminated() {
        let s: Vec<u16> = "ImmersiveColorSet".encode_utf16().chain([0]).collect();
        assert_eq!(wide_ptr_to_string(s.as_ptr()), "ImmersiveColorSet");
    }

    #[test]
    fn wide_ptr_null_is_empty() {
        assert_eq!(wide_ptr_to_string(std::ptr::null()), "");
    }

    #[test]
    fn wake_message_id_is_stable() {
        let a = wake_message_id();
        let b = wake_message_id();
        assert!(a >= 0xC000);
        assert_eq!(a, b);
    }
}
