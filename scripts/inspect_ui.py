#!/usr/bin/env python3
"""Capture the app's WebView content and print it as ASCII art.

Used to verify the UI renders correctly without opening an image viewer.

Usage: python scripts/inspect_ui.py [cols] [rows]
"""
import ctypes
import sys
from ctypes import wintypes

# Make this process DPI aware, otherwise Windows virtualizes window
# coordinates and PrintWindow only captures part of the window.
try:
    ctypes.windll.shcore.SetProcessDpiAwareness(2)  # PER_MONITOR_AWARE
except Exception:
    try:
        ctypes.windll.user32.SetProcessDPIAware()
    except Exception:
        pass

user32 = ctypes.windll.user32
gdi32 = ctypes.windll.gdi32


class BI(ctypes.Structure):
    _fields_ = [
        ("s", wintypes.DWORD), ("w", ctypes.c_long), ("h", ctypes.c_long),
        ("p", wintypes.WORD), ("b", wintypes.WORD), ("c", wintypes.DWORD),
        ("si", wintypes.DWORD), ("x", ctypes.c_long), ("y", ctypes.c_long),
        ("cu", wintypes.DWORD), ("ci", wintypes.DWORD),
    ]


def grab(hwnd):
    rect = wintypes.RECT()
    user32.GetWindowRect(hwnd, ctypes.byref(rect))
    w, h = rect.right - rect.left, rect.bottom - rect.top
    hdc = user32.GetWindowDC(0)
    mem = gdi32.CreateCompatibleDC(hdc)
    bmp = gdi32.CreateCompatibleBitmap(hdc, w, h)
    gdi32.SelectObject(mem, bmp)
    user32.PrintWindow(hwnd, mem, 2)  # PW_RENDERFULLCONTENT
    bi = BI()
    bi.s = ctypes.sizeof(BI)
    bi.w, bi.h = w, -h
    bi.p, bi.b = 1, 32
    buf = ctypes.create_string_buffer(w * h * 4)
    gdi32.GetDIBits(mem, bmp, 0, h, buf, ctypes.byref(bi), 0)
    gdi32.DeleteObject(bmp)
    gdi32.DeleteDC(mem)
    user32.ReleaseDC(0, hdc)
    return w, h, bytes(buf)


def find_app_windows():
    """Return (main_hwnd, webview_hwnd) for the running app process."""
    import subprocess

    out = subprocess.run(
        ["tasklist", "/FI", "IMAGENAME eq windows-theme-switcher.exe", "/FO", "CSV", "/NH"],
        capture_output=True,
        text=True,
    ).stdout
    if "windows-theme-switcher.exe" not in out:
        return None, None
    pid = int(out.split(",")[1].strip('"'))

    main_hwnd = None

    def cb(hwnd, _):
        nonlocal main_hwnd
        wpid = wintypes.DWORD()
        user32.GetWindowThreadProcessId(hwnd, ctypes.byref(wpid))
        if wpid.value == pid:
            cls = ctypes.create_unicode_buffer(256)
            user32.GetClassNameW(hwnd, cls, 256)
            if cls.value == "Tauri Window":
                main_hwnd = hwnd
        return True

    user32.EnumWindows(ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)(cb), 0)
    if not main_hwnd:
        return None, None

    found = []

    def cb2(child, _):
        cls = ctypes.create_unicode_buffer(256)
        user32.GetClassNameW(child, cls, 256)
        if cls.value == "WRY_WEBVIEW":
            found.append(child)
        return True

    user32.EnumChildWindows(
        main_hwnd, ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)(cb2), 0
    )
    return main_hwnd, (found[0] if found else None)


def main():
    cols = int(sys.argv[1]) if len(sys.argv) > 1 else 76
    rows = int(sys.argv[2]) if len(sys.argv) > 2 else 32

    main_hwnd, webview = find_app_windows()
    print("main window:", main_hwnd, "| webview:", webview)
    if not main_hwnd:
        print("app is not running")
        return 1
    if not webview:
        print("no WRY_WEBVIEW child found")
        return 1

    w, h, raw = grab(webview)
    print(f"webview: {w}x{h}")

    def px(x, y):
        i = (y * w + x) * 4
        return raw[i + 2], raw[i + 1], raw[i]

    for name, (x, y) in {
        "top-left": (4, 4),
        "titlebar": (w // 2, 18),
        "center": (w // 2, h // 2),
        "bottom": (w // 2, h - 6),
    }.items():
        print(f"  {name} rgb={px(x, y)}")

    chars = " .:-=+*#%@"
    print()
    for ry in range(rows):
        y = min(h - 1, int((ry + 0.5) * h / rows))
        line = ""
        for rx in range(cols):
            x = min(w - 1, int((rx + 0.5) * w / cols))
            r, g, b = px(x, y)
            lum = (r * 299 + g * 587 + b * 114) // 1000
            line += chars[min(len(chars) - 1, lum * len(chars) // 256)]
        print(f"{y * 100 // h:3d}% |{line}|")
    return 0


if __name__ == "__main__":
    sys.exit(main())
