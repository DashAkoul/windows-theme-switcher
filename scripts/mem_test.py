#!/usr/bin/env python3
"""Open/close the app window N times and report memory + handle counts.

Verifies that closing the window releases the WebView2 footprint and that
repeated cycles do not leak.
"""
import ctypes
import subprocess
import sys
import time
from ctypes import wintypes

user32 = ctypes.windll.user32
kernel32 = ctypes.windll.kernel32
psapi = ctypes.windll.psapi


class PMC(ctypes.Structure):
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
    ]


PROCESS_QUERY_INFORMATION = 0x0400
PROCESS_VM_READ = 0x0010


def app_pid():
    out = subprocess.run(
        ["tasklist", "/FI", "IMAGENAME eq windows-theme-switcher.exe", "/FO", "CSV", "/NH"],
        capture_output=True,
        text=True,
    ).stdout
    if "windows-theme-switcher.exe" not in out:
        return None
    return int(out.split(",")[1].strip('"'))


def stat(pid):
    h = kernel32.OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, False, pid)
    if not h:
        return "n/a"
    pmc = PMC()
    pmc.cb = ctypes.sizeof(PMC)
    psapi.GetProcessMemoryInfo(h, ctypes.byref(pmc), pmc.cb)
    handle_count = wintypes.DWORD()
    ctypes.windll.kernel32.GetProcessHandleCount(h, ctypes.byref(handle_count))
    kernel32.CloseHandle(h)
    return f"ram={int(pmc.WorkingSetSize / 1024 / 1024)}MB handles={handle_count.value}"


def find_window(pid):
    found = []

    def cb(hwnd, _):
        wpid = wintypes.DWORD()
        user32.GetWindowThreadProcessId(hwnd, ctypes.byref(wpid))
        if wpid.value == pid:
            cls = ctypes.create_unicode_buffer(256)
            user32.GetClassNameW(hwnd, cls, 256)
            if cls.value == "Tauri Window":
                found.append(hwnd)
        return True

    user32.EnumWindows(ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)(cb), 0)
    return found[0] if found else None


def webview_count():
    out = subprocess.run(
        ["tasklist", "/FI", "IMAGENAME eq msedgewebview2.exe", "/FO", "CSV", "/NH"],
        capture_output=True,
        text=True,
    ).stdout
    return out.count("msedgewebview2.exe")


def main():
    exe = sys.argv[1]
    cycles = int(sys.argv[2]) if len(sys.argv) > 2 else 3

    pid = app_pid()
    print(f"baseline (tray only): {stat(pid) if pid else 'not running'} webviews={webview_count()}")

    for i in range(1, cycles + 1):
        subprocess.run([exe], check=False)  # wakes the running instance
        time.sleep(6)
        pid = app_pid()
        hwnd = find_window(pid) if pid else None
        opened = stat(pid) if pid else "n/a"
        wv_open = webview_count()
        if hwnd:
            user32.PostMessageW(hwnd, 0x0010, 0, 0)  # WM_CLOSE
        time.sleep(4)
        pid = app_pid()
        closed = stat(pid) if pid else "DEAD"
        print(f"cycle {i}: open[{opened} webviews={wv_open}] -> closed[{closed} webviews={webview_count()}]")


if __name__ == "__main__":
    main()
