#!/usr/bin/env python3
"""Render a PNG as ASCII art (pure stdlib) for text-only inspection.

Usage: python scripts/png_ascii.py <file.png> [cols] [rows]
"""
import struct
import sys
import zlib


def read_png(path):
    data = open(path, "rb").read()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "not a PNG"
    pos, idat, w, h, ctype = 8, b"", 0, 0, 0
    while pos < len(data):
        ln = struct.unpack(">I", data[pos:pos + 4])[0]
        typ = data[pos + 4:pos + 8]
        body = data[pos + 8:pos + 8 + ln]
        pos += 12 + ln
        if typ == b"IHDR":
            w, h, _bd, ctype = struct.unpack(">IIBB", body[:10])
        elif typ == b"IDAT":
            idat += body
        elif typ == b"IEND":
            break
    raw = zlib.decompress(idat)
    bpp = {0: 1, 2: 3, 4: 2, 6: 4}[ctype]
    stride = w * bpp
    rows, i, prev = [], 0, bytearray(stride)
    for _ in range(h):
        f = raw[i]
        i += 1
        line = bytearray(raw[i:i + stride])
        i += stride
        if f == 1:
            for x in range(bpp, stride):
                line[x] = (line[x] + line[x - bpp]) & 0xFF
        elif f == 2:
            for x in range(stride):
                line[x] = (line[x] + prev[x]) & 0xFF
        elif f == 3:
            for x in range(stride):
                a = line[x - bpp] if x >= bpp else 0
                line[x] = (line[x] + ((a + prev[x]) >> 1)) & 0xFF
        elif f == 4:
            for x in range(stride):
                a = line[x - bpp] if x >= bpp else 0
                b = prev[x]
                c = prev[x - bpp] if x >= bpp else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[x] = (line[x] + pr) & 0xFF
        rows.append(line)
        prev = line
    return w, h, bpp, rows


def main():
    path = sys.argv[1]
    cols = int(sys.argv[2]) if len(sys.argv) > 2 else 72
    rws = int(sys.argv[3]) if len(sys.argv) > 3 else 34
    w, h, bpp, rows = read_png(path)
    print(f"{path}  {w}x{h}  bpp={bpp}")
    chars = " .:-=+*#%@"
    for ry in range(rws):
        y = min(h - 1, int((ry + 0.5) * h / rws))
        line = ""
        for rx in range(cols):
            x = min(w - 1, int((rx + 0.5) * w / cols))
            o = x * bpp
            r, g, b = rows[y][o], rows[y][o + 1], rows[y][o + 2]
            lum = (r * 299 + g * 587 + b * 114) // 1000
            line += chars[min(len(chars) - 1, lum * len(chars) // 256)]
        print(f"{y * 100 // h:3d}% |{line}|")
    # dominant colors
    from collections import Counter
    c = Counter()
    for y in range(0, h, 3):
        for x in range(0, w, 3):
            o = x * bpp
            c[(rows[y][o], rows[y][o + 1], rows[y][o + 2])] += 1
    print("top colors:", c.most_common(5))


if __name__ == "__main__":
    main()
