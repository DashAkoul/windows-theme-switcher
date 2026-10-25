#!/usr/bin/env python3
"""Convert a 24-bit BMP to PNG (pure stdlib, no Pillow)."""
import struct
import sys
import zlib


def bmp_to_png(src: str, dst: str) -> None:
    with open(src, "rb") as f:
        data = f.read()

    assert data[:2] == b"BM", "not a BMP"
    pixel_offset = struct.unpack_from("<I", data, 10)[0]
    header_size = struct.unpack_from("<I", data, 14)[0]
    width, height = struct.unpack_from("<ii", data, 18)
    bit_count = struct.unpack_from("<H", data, 28)[0]
    assert bit_count == 24, f"expected 24-bit BMP, got {bit_count}"

    flip = height > 0
    height = abs(height)
    row_size = (width * 3 + 3) // 4 * 4

    # Read rows (BMP stores bottom-up by default)
    rows = []
    for y in range(height):
        src_y = height - 1 - y if flip else y
        base = pixel_offset + src_y * row_size
        row = []
        for x in range(width):
            i = base + x * 3
            b, g, r = data[i], data[i + 1], data[i + 2]
            row.append((r, g, b, 255))
        rows.append(row)

    # Write PNG
    def chunk(typ: bytes, payload: bytes) -> bytes:
        return (
            struct.pack(">I", len(payload))
            + typ
            + payload
            + struct.pack(">I", zlib.crc32(typ + payload) & 0xFFFFFFFF)
        )

    raw = bytearray()
    for row in rows:
        raw.append(0)
        for px in row:
            raw += bytes(px)

    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(bytes(raw), 9))
    png += chunk(b"IEND", b"")

    with open(dst, "wb") as f:
        f.write(png)
    print(f"saved {dst} ({width}x{height})")


if __name__ == "__main__":
    bmp_to_png(sys.argv[1], sys.argv[2])
