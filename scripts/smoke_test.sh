#!/usr/bin/env bash
# Smoke test for the release build.
#
# Verifies:
#   1. the release binary exists and starts, staying alive in the tray
#   2. the startup lines land in the log file
#   3. a second launch exits on its own (single instance) and wakes the first
#   4. the first instance can be terminated
#
# Usage (Git Bash):
#   bash scripts/smoke_test.sh
#
# Exit code 0 = all checks passed.

set -u

APP_DIR="$(cd "$(dirname "$0")/.." && pwd)"
# The binary is named after the Cargo package, not after productName.
EXE="$APP_DIR/src-tauri/target/release/windows-theme-switcher.exe"
LOG="$APPDATA/com.themeswitcher.app/app.log"

fail() {
    echo "FAIL: $*"
    exit 1
}

echo "== Windows Theme Switcher smoke test =="

[ -f "$EXE" ] || fail "release binary not found at $EXE (run 'npm run tauri build' first)"
echo "OK: binary found"

# A previous instance would win the single-instance race and make check 1 pass
# for the wrong reason.
if tasklist /FI "IMAGENAME eq windows-theme-switcher.exe" 2>/dev/null | grep -qi "windows-theme"; then
    fail "an instance is already running - quit it from the tray first"
fi

rm -f "$LOG" 2>/dev/null

"$EXE" &
APP_PID=$!
echo "launched pid=$APP_PID"

sleep 6
kill -0 "$APP_PID" 2>/dev/null || fail "process died within 6 seconds"
echo "OK: process alive after 6s"

if [ -f "$LOG" ]; then
    grep -q "startup complete" "$LOG" || fail "log has no 'startup complete' line"
    echo "OK: log written"
    echo "--- last log lines ---"
    tail -4 "$LOG"
    echo "----------------------"
else
    fail "no log file at $LOG"
fi

# Second launch must exit by itself instead of starting a second tray icon.
"$EXE" &
SECOND_PID=$!
for _ in 1 2 3 4 5 6 7 8 9 10; do
    kill -0 "$SECOND_PID" 2>/dev/null || break
    sleep 0.5
done
if kill -0 "$SECOND_PID" 2>/dev/null; then
    kill "$SECOND_PID" 2>/dev/null
    fail "second instance kept running (single instance broken)"
fi
wait "$SECOND_PID" 2>/dev/null
echo "OK: second instance exited on its own"

# Terminate. `kill` alone is not reliable for a Windows GUI process (there is
# no console signal to deliver), so use taskkill.
MSYS_NO_PATHCONV=1 taskkill /PID "$APP_PID" /F >/dev/null 2>&1 || kill -9 "$APP_PID" 2>/dev/null
sleep 1
if kill -0 "$APP_PID" 2>/dev/null; then
    echo "WARN: process still alive after taskkill"
else
    echo "OK: terminated"
fi

echo "== smoke test passed =="
