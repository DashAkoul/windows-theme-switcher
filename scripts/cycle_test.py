"""Open/close the tray panel N times and report the app footprint after each cycle.

Usage: python scripts/cycle_test.py <exe> [cycles]
"""

import ctypes
import subprocess
import sys
import time
from ctypes import wintypes

try:
    ctypes.windll.shcore.SetProcessDpiAwareness(2)
except Exception:
    pass

user32 = ctypes.windll.user32
EnumProc = ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)


def app_pid() -> int | None:
    out = subprocess.run(
        ["tasklist", "/FI", "IMAGENAME eq windows-theme-switcher.exe", "/FO", "CSV", "/NH"],
        capture_output=True,
        text=True,
    ).stdout
    if "windows-theme" not in out:
        return None
    return int(out.split(",")[1].strip('"'))


def find_panel(pid: int) -> int | None:
    found = []

    def cb(hwnd, _):
        wpid = wintypes.DWORD()
        user32.GetWindowThreadProcessId(hwnd, ctypes.byref(wpid))
        if wpid.value == pid:
            cls = ctypes.create_unicode_buffer(256)
            user32.GetClassNameW(hwnd, cls, 256)
            if cls.value == "Tauri Window" and user32.IsWindowVisible(hwnd):
                found.append(hwnd)
        return True

    user32.EnumWindows(EnumProc(cb), 0)
    return found[0] if found else None


def stats(pid: int) -> str:
    cmd = (
        f"$a = Get-Process -Id {pid} -ErrorAction SilentlyContinue; "
        f"$w = Get-CimInstance Win32_Process -Filter \"Name='msedgewebview2.exe'\" | "
        f"Where-Object {{ $_.CommandLine -match 'themeswitcher' }}; "
        f"if ($a) {{ 'app=' + [int]($a.WS/1MB) + 'MB handles=' + $a.Handles + ' webviews=' + $w.Count }}"
    )
    return subprocess.run(
        ["powershell", "-NoProfile", "-Command", cmd], capture_output=True, text=True
    ).stdout.strip()


def main() -> int:
    exe = sys.argv[1]
    cycles = int(sys.argv[2]) if len(sys.argv) > 2 else 3
    pid = app_pid()
    if not pid:
        print("app is not running")
        return 1

    print("baseline:", stats(pid))
    for i in range(1, cycles + 1):
        subprocess.Popen([exe])
        panel = None
        for _ in range(40):
            time.sleep(0.2)
            panel = find_panel(pid)
            if panel:
                break
        if not panel:
            print(f"cycle {i}: panel never appeared")
            continue
        time.sleep(1.0)
        opened = stats(pid)
        user32.PostMessageW(panel, 0x0010, 0, 0)  # WM_CLOSE -> destroy
        time.sleep(2.5)
        print(f"cycle {i}: open {opened} -> closed {stats(pid)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
