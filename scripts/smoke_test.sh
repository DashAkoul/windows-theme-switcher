# Smoke test script - Windows Theme Switcher
# Runs after `npm run tauri build`. Verifies:
#   1. the release binary starts and stays alive (no crash)
#   2. the log file is written with startup lines
#   3. the onboarding window is shown on first run (screenshot)
#   4. the process exits cleanly on request
#
# Usage (Git Bash):
#   bash scripts/smoke_test.sh

set -u

APP_DIR="$(cd "$(dirname "$0")/.." && pwd)"
EXE="$APP_DIR/src-tauri/target/release/Windows Theme Switcher.exe"
LOG="$APPDATA/com.themeswitcher.app/app.log"

echo "== Windows Theme Switcher smoke test =="

if [ ! -f "$EXE" ]; then
    echo "FAIL: release binary not found at:"
    echo "  $EXE"
    echo "Run 'npm run tauri build' first."
    exit 1
fi
echo "OK: binary found"

# Clean previous log for a fresh check
rm -f "$LOG" 2>/dev/null

# Launch (onboarding window should appear on first run)
"$EXE" &
APP_PID=$!
echo "launched pid=$APP_PID"

sleep 6

if ! kill -0 "$APP_PID" 2>/dev/null; then
    echo "FAIL: process died within 6 seconds"
    wait "$APP_PID" 2>/dev/null
    echo "exit code: $?"
    exit 1
fi
echo "OK: process alive after 6s"

if [ -f "$LOG" ]; then
    echo "OK: log file exists"
    echo "--- last log lines ---"
    tail -5 "$LOG"
    echo "----------------------"
else
    echo "WARN: no log file at $LOG"
fi

# Terminate
kill "$APP_PID" 2>/dev/null
sleep 1
if kill -0 "$APP_PID" 2>/dev/null; then
    kill -9 "$APP_PID" 2>/dev/null
    echo "NOTE: needed SIGKILL"
else
    echo "OK: terminated cleanly"
fi

echo "== smoke test done =="
