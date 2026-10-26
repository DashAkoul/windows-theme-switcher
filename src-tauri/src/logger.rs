//! Minimal append-only file logger.
//!
//! Writes timestamped lines to `<config_dir>/app.log`. The logger never
//! panics and never blocks the caller for long: on failure it silently
//! degrades (the UI never depends on logging).

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_FILE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Initialize the logger with the target log file path.
/// Safe to call multiple times; the last call wins.
pub fn init(path: PathBuf) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut guard) = LOG_FILE.lock() {
        *guard = Some(path);
    }
}

/// Return the current log file path (if initialized).
pub fn log_path() -> Option<PathBuf> {
    LOG_FILE.lock().ok().and_then(|g| g.clone())
}

fn timestamp() -> String {
    // Timestamps are UTC (no timezone crate). Keep that in mind when
    // correlating log lines with local-time observations.
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86_400;
    let time_of_day = secs % 86_400;
    let (h, m, s) = (
        time_of_day / 3600,
        (time_of_day % 3600) / 60,
        time_of_day % 60,
    );
    // Convert days since epoch to Y-M-D (civil calendar algorithm).
    let (y, mo, d) = civil_from_days(days as i64);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{m:02}:{s:02}")
}

/// Howard Hinnant's civil_from_days algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Rotate the log once it grows past this size, so a long-running tray app
/// cannot fill the user's profile with log lines.
const MAX_LOG_BYTES: u64 = 256 * 1024;

fn write_line(level: &str, message: &str) {
    let path = match log_path() {
        Some(p) => p,
        None => return,
    };
    rotate_if_needed(&path);
    let line = format!("{} [{}] {}\n", timestamp(), level, message);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(line.as_bytes());
    }
}

/// Keep the previous generation at `<log>.1` and start a fresh file.
fn rotate_if_needed(path: &Path) {
    let too_big = std::fs::metadata(path)
        .map(|m| m.len() > MAX_LOG_BYTES)
        .unwrap_or(false);
    if too_big {
        let _ = std::fs::rename(path, path.with_extension("log.1"));
    }
}

pub fn info(message: impl AsRef<str>) {
    write_line("INFO", message.as_ref());
}

pub fn warn(message: impl AsRef<str>) {
    write_line("WARN", message.as_ref());
}

pub fn error(message: impl AsRef<str>) {
    write_line("ERROR", message.as_ref());
}

/// Ensure a fresh file handle exists (used in tests).
#[allow(dead_code)]
pub fn debug_create(path: &Path) -> std::io::Result<File> {
    File::create(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_date_known_values() {
        // 2026-10-25 is day 20,751 since the Unix epoch
        assert_eq!(civil_from_days(20_751), (2026, 10, 25));
        // 1970-01-01
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // 2000-03-01 (leap year edge)
        assert_eq!(civil_from_days(11_017), (2000, 3, 1));
    }

    #[test]
    fn timestamp_is_iso_like() {
        let ts = timestamp();
        assert_eq!(ts.len(), 19);
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[10..11], " ");
    }

    #[test]
    fn oversized_log_is_rotated() {
        let dir = std::env::temp_dir().join("wts_logger_rotate");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("rotate.log");
        let old = dir.join("rotate.log.1");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&old);

        // Just over the cap.
        std::fs::write(&path, vec![b'x'; (MAX_LOG_BYTES + 1) as usize]).unwrap();
        rotate_if_needed(&path);
        assert!(old.exists());
        assert!(!path.exists());

        let _ = std::fs::remove_file(&old);
    }
}
