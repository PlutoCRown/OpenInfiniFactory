# 不依赖 PIL 的 PNG 写出（Blender 内嵌 Python 常用）
"""最小 RGBA PNG 编码器。"""

from __future__ import annotations

import struct
import zlib
from pathlib import Path


def write_png_rgba(path: Path, width: int, height: int, rgba: list[int]) -> None:
    """写出 8-bit RGBA PNG；rgba 为 length=width*height*4 的 0–255 列表。"""

    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    raw = bytearray()
    for y in range(height):
        raw.append(0)
        i = y * width * 4
        raw.extend(rgba[i : i + width * 4])
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n")
        f.write(chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)))
        f.write(chunk(b"IDAT", zlib.compress(bytes(raw), 9)))
        f.write(chunk(b"IEND", b""))
