"""Report the real footprint of the tray process.

Prints working set (what Task Manager calls "Memory"), private commit,
CPU time, handles and threads at several points in time, so the effect of
the working-set trimming can be verified instead of guessed.

Usage: python scripts/mem_report.py [exe]
"""

import ctypes
import subprocess
import sys
import time
from ctypes import wintypes

psapi = ctypes.windll.psapi
kernel32 = ctypes.windll.kernel32
EnumProc = ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)


class PROCESS_MEMORY_COUNTERS_EX(ctypes.Structure):
    _fields_ = [
        ("cb", wintypes.DWORD),
        ("PageFaultCount", wintypes.DWORD),
        ("PeakWorkingSetSize", ctypes.c_size_t),
        ("WorkingSetSize", ctypes.c_size_t),
        ("QuotaPeakPagedPoolUsage", ctypes.c_size_t),
        ("QuotaPagedPoolUsage", ctypes.c_size_t),
        ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t),
        ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
        ("PagefileUsage", ctypes.c_size_t),
        ("PeakPagefileUsage", ctypes.c_size_t),
        ("PrivateUsage", ctypes.c_size_t),
    ]


def app_pid() -> int | None:
    out = subprocess.run(
        ["tasklist", "/FI", "IMAGENAME eq windows-theme-switcher.exe", "/FO", "CSV", "/NH"],
        capture_output=True,
        text=True,
    ).stdout
    if "windows-theme" not in out:
        return None
    return int(out.split(",")[1].strip('"'))


def open_handle(pid: int) -> int:
    return kernel32.OpenProcess(0x1F0FFF, False, pid)


def sample(handle: int) -> dict:
    c = PROCESS_MEMORY_COUNTERS_EX()
    c.cb = ctypes.sizeof(PROCESS_MEMORY_COUNTERS_EX)
    psapi.GetProcessMemoryInfo(handle, ctypes.byref(c), c.cb)
    cpu = kernel32.GetProcessTimes
    creation, exit_, kernel, user = (
        wintypes.FILETIME(),
        wintypes.FILETIME(),
        wintypes.FILETIME(),
        wintypes.FILETIME(),
    )
    cpu(
        handle, ctypes.byref(creation), ctypes.byref(exit_), ctypes.byref(kernel), ctypes.byref(user)
    )
    to_s = lambda ft: (ft.dwHighDateTime << 32 | ft.dwLowDateTime) / 1e7  # noqa: E731
    return {
        "ws_kb": c.WorkingSetSize // 1024,
        "private_kb": c.PrivateUsage // 1024,
        "peak_ws_kb": c.PeakWorkingSetSize // 1024,
        "cpu_s": to_s(kernel) + to_s(user),
    }


def proc_counts(pid: int) -> tuple[int, int]:
    out = subprocess.run(
        [
            "powershell",
            "-NoProfile",
            "-Command",
            f"$p = Get-Process -Id {pid}; \"$($p.HandleCount) $($p.Threads.Count)\"",
        ],
        capture_output=True,
        text=True,
    ).stdout.split()
    if len(out) >= 2:
        return int(out[0]), int(out[1])
    return 0, 0


def webviews() -> int:
    out = subprocess.run(
        [
            "powershell",
            "-NoProfile",
            "-Command",
            "(Get-CimInstance Win32_Process -Filter \"Name='msedgewebview2.exe'\" | "
            "Where-Object { $_.CommandLine -match 'themeswitcher' }).Count",
        ],
        capture_output=True,
        text=True,
    ).stdout.strip()
    return int(out or 0)


def panel_visible(pid: int) -> bool:
    found = []

    def cb(hwnd, _):
        wpid = wintypes.DWORD()
        ctypes.windll.user32.GetWindowThreadProcessId(hwnd, ctypes.byref(wpid))
        if wpid.value == pid:
            cls = ctypes.create_unicode_buffer(256)
            ctypes.windll.user32.GetClassNameW(hwnd, cls, 256)
            if cls.value == "Tauri Window" and ctypes.windll.user32.IsWindowVisible(hwnd):
                found.append(hwnd)
        return True

    ctypes.windll.user32.EnumWindows(EnumProc(cb), 0)
    return bool(found)


def row(label: str, s: dict, handles: int, threads: int, wv: int) -> None:
    print(
        f"{label:<26} WS={s['ws_kb']:>6} KB  private={s['private_kb']:>5} KB  "
        f"peak={s['peak_ws_kb']:>6} KB  cpu={s['cpu_s']:>6.2f}s  "
        f"handles={handles:>4}  threads={threads:>2}  webviews={wv}"
    )


def main() -> int:
    exe = sys.argv[1] if len(sys.argv) > 1 else None
    pid = app_pid()
    if pid is None and exe:
        subprocess.Popen([exe])
        for _ in range(40):
            time.sleep(0.5)
            pid = app_pid()
            if pid:
                break
    if pid is None:
        print("app is not running and no exe given")
        return 1

    handle = open_handle(pid)
    print("--- cold start ---")
    for wait, label in [(2, "t+2s"), (4, "t+6s"), (6, "t+12s")]:
        time.sleep(wait)
        s = sample(handle)
        h, t = proc_counts(pid)
        row(label, s, h, t, webviews())

    print("--- idle CPU over 30s ---")
    before = sample(handle)["cpu_s"]
    time.sleep(30)
    after = sample(handle)
    print(f"CPU used while idle: {after['cpu_s'] - before:.2f}s over 30s")
    h, t = proc_counts(pid)
    row("after 30s idle", after, h, t, webviews())

    if panel_visible(pid):
        print("--- panel is open ---")
        s = sample(handle)
        h, t = proc_counts(pid)
        row("panel open", s, h, t, webviews())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
