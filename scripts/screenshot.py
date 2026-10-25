#!/usr/bin/env python3
"""Capture the primary screen to a BMP file using ctypes (no dependencies)."""
import ctypes
import struct
import sys
from ctypes import wintypes

SRCCOPY = 0x00CC0020
user32 = ctypes.windll.user32
gdi32 = ctypes.windll.gdi32

width = user32.GetSystemMetrics(0)
height = user32.GetSystemMetrics(1)

hdc_screen = user32.GetWindowDC(0)
hdc_mem = gdi32.CreateCompatibleDC(hdc_screen)
hbm = gdi32.CreateCompatibleBitmap(hdc_screen, width, height)
gdi32.SelectObject(hdc_mem, hbm)
gdi32.BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, 0, 0, SRCCOPY)


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
bi.biWidth = width
bi.biHeight = -height  # top-down rows
bi.biPlanes = 1
bi.biBitCount = 32
bi.biCompression = 0  # BI_RGB

buffer = ctypes.create_string_buffer(width * height * 4)
got = gdi32.GetDIBits(hdc_mem, hbm, 0, height, buffer, ctypes.byref(bi), 0)
if got == 0:
    print("GetDIBits failed")
    sys.exit(1)

out = sys.argv[1] if len(sys.argv) > 1 else "screenshot.bmp"
raw = bytes(buffer)  # indexable as ints
row_size = (width * 3 + 3) // 4 * 4
image_size = row_size * height
with open(out, "wb") as f:
    f.write(b"BM")
    f.write(struct.pack("<I", 14 + 40 + image_size))
    f.write(struct.pack("<HH", 0, 0))
    f.write(struct.pack("<I", 14 + 40))
    f.write(struct.pack("<IiiHHIIiiII", 40, width, height, 1, 24, 0, image_size, 2835, 2835, 0, 0))
    for y in range(height):
        row = bytearray()
        base = y * width * 4
        for x in range(width):
            i = base + x * 4
            row += bytes((raw[i], raw[i + 1], raw[i + 2]))  # B,G,R
        row += b"\x00" * (row_size - len(row))
        f.write(row)

print(f"saved {out} ({width}x{height})")
