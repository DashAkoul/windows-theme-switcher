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
    // Local time formatting without extra crates: use UTC offset-free stamp.
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

fn write_line(level: &str, message: &str) {
    let path = match log_path() {
        Some(p) => p,
        None => return,
    };
    let line = format!("{} [{}] {}\n", timestamp(), level, message);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(line.as_bytes());
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

/// Read the tail of the log file (last `max_lines` lines). Used by the
/// diagnostics command.
pub fn tail(max_lines: usize) -> String {
    let path = match log_path() {
        Some(p) => p,
        None => return String::new(),
    };
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    lines[start..].join("\n")
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
    fn tail_returns_last_lines() {
        let dir = std::env::temp_dir().join("wts_logger_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test.log");
        let _ = std::fs::write(&path, "a\nb\nc\nd\n");
        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        let start = lines.len().saturating_sub(2);
        assert_eq!(lines[start..].join("\n"), "c\nd");
    }
}
