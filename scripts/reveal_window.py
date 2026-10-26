#!/usr/bin/env python3
"""Minimize the current foreground window, wait, restore it.

Used to reveal our app window for screenshot verification.
Usage: python scripts/reveal_window.py <seconds-to-keep-minimized>
"""
import ctypes
import sys
import time
from ctypes import wintypes

user32 = ctypes.windll.user32

SW_MINIMIZE = 6
SW_RESTORE = 9

hwnd = user32.GetForegroundWindow()
if not hwnd:
    print("no foreground window")
    sys.exit(1)

length = user32.GetWindowTextLengthW(hwnd)
buf = ctypes.create_unicode_buffer(length + 1)
user32.GetWindowTextW(hwnd, buf, length + 1)
print(f"foreground window: '{buf.value}' (hwnd={hwnd})")

keep = float(sys.argv[1]) if len(sys.argv) > 1 else 4.0
user32.ShowWindow(hwnd, SW_MINIMIZE)
print(f"minimized for {keep}s...")
time.sleep(keep)
user32.ShowWindow(hwnd, SW_RESTORE)
print("restored")
