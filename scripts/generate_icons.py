#!/usr/bin/env python3
"""Generate all icons for Windows Theme Switcher.

Pure standard library (zlib + struct) PNG/ICO writer - no Pillow required.

Outputs (into src-tauri/icons/):
  - tray-moon.png       32x32  tray icon shown while Windows is LIGHT
  - tray-sun.png        32x32  tray icon shown while Windows is DARK
  - 32x32.png           32x32  app icon
  - 128x128.png         128x128 app icon
  - 128x128@2x.png      256x256 app icon
  - icon.ico            multi-size ICO (PNG-compressed entries)
"""

import math
import os
import struct
import zlib

OUT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "src-tauri", "icons")

# ---------------------------------------------------------------------------
# Minimal PNG writer (RGBA, 8-bit, filter 0)
# ---------------------------------------------------------------------------

def write_png(path, width, height, pixels):
    """pixels: list of rows, each a list of (r, g, b, a) tuples."""
    def chunk(typ, data):
        return (
            struct.pack(">I", len(data))
            + typ
            + data
            + struct.pack(">I", zlib.crc32(typ + data) & 0xFFFFFFFF)
        )

    raw = bytearray()
    for y in range(height):
        raw.append(0)  # filter type: none
        for px in pixels[y]:
            raw += bytes(px)
    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(bytes(raw), 9))
    png += chunk(b"IEND", b"")
    with open(path, "wb") as f:
        f.write(png)


# ---------------------------------------------------------------------------
# Minimal ICO writer (PNG-compressed entries, Vista+)
# ---------------------------------------------------------------------------

def write_ico(path, png_entries):
    """png_entries: list of (size, png_bytes)."""
    count = len(png_entries)
    header = struct.pack("<HHH", 0, 1, count)  # reserved, type=icon, count
    offset = 6 + 16 * count
    dir_entries = b""
    image_data = b""
    for size, data in png_entries:
        dim = 0 if size >= 256 else size  # 256 is encoded as 0
        dir_entries += struct.pack("<BBBBHHII", dim, dim, 0, 0, 1, 32, len(data), offset)
        image_data += data
        offset += len(data)
    with open(path, "wb") as f:
        f.write(header + dir_entries + image_data)


# ---------------------------------------------------------------------------
# Drawing helpers (supersampled for anti-aliasing)
# ---------------------------------------------------------------------------

SS = 4  # supersampling factor


def render(size, draw_fn):
    """Render at size*SS, box-downsample to size. draw_fn(x, y) -> (r,g,b,a)
    receives normalized coordinates in [0,1)."""
    big = size * SS
    rows = []
    for by in range(size):
        row = []
        for bx in range(size):
            r = g = b = a = 0
            for sy in range(SS):
                for sx in range(SS):
                    px, py = (bx * SS + sx + 0.5) / big, (by * SS + sy + 0.5) / big
                    pr, pg, pb, pa = draw_fn(px, py)
                    r += pr * pa / 255.0  # premultiplied with normalized alpha
                    g += pg * pa / 255.0
                    b += pb * pa / 255.0
                    a += pa
            n = SS * SS
            a_avg = a / n  # 0..255
            if a_avg > 0:
                scale = 255.0 / a_avg
                r = min(255, max(0, round((r / n) * scale)))
                g = min(255, max(0, round((g / n) * scale)))
                b = min(255, max(0, round((b / n) * scale)))
            else:
                r = g = b = 0
            row.append((r, g, b, int(round(a_avg))))
        rows.append(row)
    return rows


def lerp(a, b, t):
    return a + (b - a) * t


def lerp_color(c1, c2, t):
    return tuple(int(round(lerp(c1[i], c2[i], t))) for i in range(3))


def in_circle(x, y, cx, cy, r):
    return (x - cx) ** 2 + (y - cy) ** 2 <= r * r


def in_rounded_rect(x, y, w, h, radius, cx=0.5, cy=0.5):
    # x,y in [0,1]; rect centered at (cx, cy) covering w x h (normalized)
    x0, x1 = cx - w / 2, cx + w / 2
    y0, y1 = cy - h / 2, cy + h / 2
    if x0 + radius <= x <= x1 - radius or y0 + radius <= y <= y1 - radius:
        return x0 <= x <= x1 and y0 <= y <= y1
    # corner circles
    for ccx, ccy in ((x0 + radius, y0 + radius), (x1 - radius, y0 + radius),
                     (x0 + radius, y1 - radius), (x1 - radius, y1 - radius)):
        if (x - ccx) ** 2 + (y - ccy) ** 2 <= radius ** 2:
            return True
    return False


def smoothstep(edge0, edge1, x):
    t = max(0.0, min(1.0, (x - edge0) / (edge1 - edge0)))
    return t * t * (3 - 2 * t)


# ---------------------------------------------------------------------------
# Icon designs
# ---------------------------------------------------------------------------

def app_icon_pixel(x, y):
    """Rounded-square gradient tile with an eclipse (sun + crescent) glyph."""
    # Background gradient: sky blue -> indigo (light, friendly)
    grad_top = (14, 165, 233)    # #0EA5E9
    grad_bottom = (79, 70, 229)  # #4F46E5
    bg = lerp_color(grad_top, grad_bottom, y)

    if not in_rounded_rect(x, y, 0.92, 0.92, 0.22):
        return (0, 0, 0, 0)

    # Glyph geometry (normalized coords)
    cx, cy = 0.5, 0.5
    sun_r = 0.30
    # Crescent cut circle, offset up-right, slightly smaller
    cut_cx, cut_cy = 0.62, 0.40
    cut_r = 0.26

    d_sun = math.hypot(x - cx, y - cy)
    d_cut = math.hypot(x - cut_cx, y - cut_cy)

    white = (255, 255, 255)
    # Sun rays (8 rays) drawn as thick segments
    ray_inner = sun_r + 0.055
    ray_outer = sun_r + 0.135
    ray_half_width = 0.028  # half-thickness of each ray

    on_ray = False
    if ray_inner <= d_sun <= ray_outer:
        ang = math.atan2(y - cy, x - cx)
        # project distance from the ray axis
        axis_ang = round(ang / (math.pi / 4)) * (math.pi / 4)
        # distance from the ray center line
        perp = abs(d_sun * math.sin(ang - axis_ang))
        if perp <= ray_half_width:
            on_ray = True

    in_sun = d_sun <= sun_r
    in_cut = d_cut <= cut_r

    alpha = 255
    if on_ray or (in_sun and not in_cut):
        # Anti-alias the sun edge and the cut edge
        edge = 1.0 - smoothstep(sun_r - 0.012, sun_r + 0.012, d_sun) if in_sun else 1.0
        cut_edge = smoothstep(cut_r - 0.012, cut_r + 0.012, d_cut)
        a = edge * (1.0 - (cut_edge if in_sun else 0.0))
        if on_ray:
            a = 1.0
        alpha = int(round(255 * max(0.0, min(1.0, a))))
        return (white[0], white[1], white[2], alpha)

    return (bg[0], bg[1], bg[2], alpha)


def tray_moon_pixel(x, y):
    """Crescent moon, dark slate - readable on a LIGHT taskbar."""
    ink = (38, 38, 44)  # near-black slate
    cx, cy = 0.5, 0.52
    r = 0.34
    cut_cx, cut_cy, cut_r = 0.66, 0.38, 0.30
    if not (in_circle(x, y, cx, cy, r) and not in_circle(x, y, cut_cx, cut_cy, cut_r)):
        return (0, 0, 0, 0)
    # soft anti-aliased edges
    d = math.hypot(x - cx, y - cy)
    dc = math.hypot(x - cut_cx, y - cut_cy)
    outer = 1.0 - smoothstep(r - 0.015, r + 0.015, d)
    inner = smoothstep(cut_r - 0.015, cut_r + 0.015, dc)
    a = outer * (1.0 - inner)
    return (ink[0], ink[1], ink[2], int(round(255 * a)))


def tray_sun_pixel(x, y):
    """Bright sun, amber - readable on a DARK taskbar."""
    ink = (255, 196, 66)  # warm amber
    cx, cy = 0.5, 0.5
    r = 0.21
    ray_inner, ray_outer, ray_hw = r + 0.05, r + 0.13, 0.030
    d = math.hypot(x - cx, y - cy)
    if d <= r:
        a = 1.0 - smoothstep(r - 0.015, r + 0.015, d)
        return (ink[0], ink[1], ink[2], int(round(255 * a)))
    if ray_inner <= d <= ray_outer:
        ang = math.atan2(y - cy, x - cx)
        axis = round(ang / (math.pi / 4)) * (math.pi / 4)
        perp = abs(d * math.sin(ang - axis))
        if perp <= ray_hw:
            a = 1.0 - smoothstep(ray_outer - 0.015, ray_outer + 0.015, d)
            return (ink[0], ink[1], ink[2], int(round(255 * a)))
    return (0, 0, 0, 0)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    os.makedirs(OUT_DIR, exist_ok=True)

    # Tray icons (32x32)
    write_png(os.path.join(OUT_DIR, "tray-moon.png"), 32, 32, render(32, tray_moon_pixel))
    write_png(os.path.join(OUT_DIR, "tray-sun.png"), 32, 32, render(32, tray_sun_pixel))

    # App icons
    write_png(os.path.join(OUT_DIR, "32x32.png"), 32, 32, render(32, app_icon_pixel))
    write_png(os.path.join(OUT_DIR, "128x128.png"), 128, 128, render(128, app_icon_pixel))
    write_png(os.path.join(OUT_DIR, "128x128@2x.png"), 256, 256, render(256, app_icon_pixel))

    # ICO with PNG-compressed entries (16/32/48/64/128/256)
    import io

    def png_bytes(size):
        buf = io.BytesIO()
        # reuse write_png logic by writing to a temp buffer
        pixels = render(size, app_icon_pixel)
        tmp = os.path.join(OUT_DIR, f".tmp_{size}.png")
        write_png(tmp, size, size, pixels)
        data = open(tmp, "rb").read()
        os.remove(tmp)
        return data

    entries = [(s, png_bytes(s)) for s in (16, 32, 48, 64, 128, 256)]
    write_ico(os.path.join(OUT_DIR, "icon.ico"), entries)

    print("Generated icons:")
    for name in sorted(os.listdir(OUT_DIR)):
        path = os.path.join(OUT_DIR, name)
        print(f"  {name} ({os.path.getsize(path)} bytes)")


if __name__ == "__main__":
    main()
