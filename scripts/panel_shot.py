"""Open the tray panel, capture it while visible, then report memory after close.

Usage: python scripts/panel_shot.py <exe> [out.png]
"""

import ctypes
import os
import subprocess
import sys
import tempfile
import time
from ctypes import wintypes

try:
    ctypes.windll.shcore.SetProcessDpiAwareness(2)
except Exception:
    pass

user32 = ctypes.windll.user32
gdi32 = ctypes.windll.gdi32

EnumProc = ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)


def app_pid() -> int | None:
    out = subprocess.run(
        ["tasklist", "/FI", "IMAGENAME eq windows-theme-switcher.exe", "/FO", "CSV", "/NH"],
        capture_output=True,
        text=True,
    ).stdout
    if "windows-theme" not in out:
        return None
    try:
        return int(out.split(",")[1].strip('"'))
    except (IndexError, ValueError):
        return None


def find_panel(pid: int) -> int | None:
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

    user32.EnumWindows(EnumProc(cb), 0)
    return found[0] if found else None


def find_webview(hwnd: int) -> int | None:
    found = []

    def cb(child, _):
        cls = ctypes.create_unicode_buffer(256)
        user32.GetClassNameW(child, cls, 256)
        if cls.value == "WRY_WEBVIEW":
            found.append(child)
        return True

    user32.EnumChildWindows(hwnd, EnumProc(cb), 0)
    return found[0] if found else None


def grab(hwnd: int):
    rect = wintypes.RECT()
    user32.GetWindowRect(hwnd, ctypes.byref(rect))
    w, h = rect.right - rect.left, rect.bottom - rect.top
    if w <= 0 or h <= 0:
        return None
    hdc = user32.GetWindowDC(0)
    mem = gdi32.CreateCompatibleDC(hdc)
    bmp = gdi32.CreateCompatibleBitmap(hdc, w, h)
    gdi32.SelectObject(mem, bmp)
    user32.PrintWindow(hwnd, mem, 2)

    class BITMAPINFOHEADER(ctypes.Structure):
        _fields_ = [
            ("biSize", wintypes.DWORD),
            ("biWidth", ctypes.c_long),
            ("biHeight", ctypes.c_long),
            ("biPlanes", wintypes.WORD),
            ("biBitCount", wintypes.WORD),
            ("biCompression", wintypes.DWORD),
            ("biSizeImage", wintypes.DWORD),
            ("biXPelsPerMeter", ctypes.c_long),
            ("biYPelsPerMeter", ctypes.c_long),
            ("biClrUsed", wintypes.DWORD),
            ("biClrImportant", wintypes.DWORD),
        ]

    bi = BITMAPINFOHEADER()
    bi.biSize = ctypes.sizeof(BITMAPINFOHEADER)
    bi.biWidth = w
    bi.biHeight = -h
    bi.biPlanes = 1
    bi.biBitCount = 32
    buf = ctypes.create_string_buffer(w * h * 4)
    gdi32.GetDIBits(mem, bmp, 0, h, buf, ctypes.byref(bi), 0)
    raw = bytes(buf)
    gdi32.DeleteObject(bmp)
    gdi32.DeleteDC(mem)
    return w, h, raw


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: panel_shot.py <exe> [out.png]")
        return 2
    exe = sys.argv[1]
    out = sys.argv[2] if len(sys.argv) > 2 else os.path.join(tempfile.gettempdir(), "panel_v2.png")

    pid = app_pid()
    if not pid:
        print("app is not running")
        return 1

    subprocess.Popen([exe])
    shot = None
    for attempt in range(40):
        time.sleep(0.2)
        panel = find_panel(pid)
        if not panel or not user32.IsWindowVisible(panel):
            continue
        webview = find_webview(panel)
        if not webview:
            continue
        got = grab(webview)
        if not got:
            continue
        w, h, raw = got

        def px(x, y):
            i = (y * w + x) * 4
            return raw[i + 2], raw[i + 1], raw[i]

        # Skip all-black frames (webview has not painted yet).
        samples = [px(x, y) for y in range(0, h, 9) for x in range(0, w, 9)]
        black = sum(1 for r, g, b in samples if r + g + b < 24)
        if black > len(samples) * 0.9:
            continue

        from PIL import Image

        img = Image.new("RGB", (w, h))
        img.putdata([px(x, y) for y in range(h) for x in range(w)])
        img.save(out)
        shot = (w, h, out)
        print(f"captured {w}x{h} at attempt {attempt} -> {out}")
        break

    if not shot:
        print("panel was not captured (hidden too fast)")
        return 1

    # Close it and confirm the webview goes away.
    panel = find_panel(pid)
    if panel:
        user32.PostMessageW(panel, 0x0010, 0, 0)  # WM_CLOSE
    time.sleep(4)

    wv = subprocess.run(
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
    print(f"webviews after close: {wv}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
