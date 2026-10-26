//! Working-set trimming.
//!
//! A tray app is idle almost all of the time, but the resident set keeps
//! whatever it touched during startup (Rust runtime, Tauri, the shell DLLs,
//! and — after the panel has been opened once — the WebView2 runtime pages).
//! Windows will not hand those pages back on its own, so the process keeps
//! reporting ~13 MB (and ~45 MB after the first panel) while doing nothing.
//!
//! `EmptyWorkingSet` asks the memory manager to move the process' pages to
//! the standby list. Private commit is unchanged, so nothing is lost: the
//! pages come back on the next touch. In practice the resident set of an idle
//! tray drops from ~13 MB to a few hundred KB.
//!
//! Trimming is deliberately event-driven (after startup, after the panel is
//! destroyed, after a tray action) rather than periodic: a timer would wake
//! the process up and burn CPU for no reason.

use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
use windows::Win32::System::Threading::GetCurrentProcess;

/// Ask Windows to page out as much of this process as it can.
pub fn trim_now() {
    unsafe {
        let _ = EmptyWorkingSet(GetCurrentProcess());
    }
}

/// Trim, then trim again a moment later.
///
/// A single call leaves the pages that get touched while the call itself
/// unwinds (thread teardown, bookkeeping) resident, which is why one trim
/// lands around 1 MB instead of the ~200 KB floor. The second pass catches
/// those.
pub fn trim_settle() {
    trim_now();
    std::thread::sleep(std::time::Duration::from_millis(350));
    trim_now();
}

/// Trim after a short delay.
///
/// The delay matters: trimming in the middle of startup (or while a theme
/// change is still broadcasting) would just page the code back in again and
/// cost page faults. Once the work is done the pages stay out.
pub fn trim_soon(delay_ms: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        trim_settle();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_does_not_panic() {
        trim_now();
        trim_now();
    }
}
