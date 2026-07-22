# 替换 GLB 内嵌第一张 PNG（只换皮、不改网格）
"""读写 glTF binary，替换 images[0] 对应 bufferView 里的 PNG。"""

from __future__ import annotations

import json
import struct
from pathlib import Path


def replace_glb_texture(
    glb_path: Path,
    new_png: bytes,
    out_path: Path,
    material_name: str | None = None,
) -> None:
    """用 new_png 替换 glb 内嵌第一张图，写出到 out_path。"""
    data = glb_path.read_bytes()
    assert data[:4] == b"glTF"
    json_len, json_type = struct.unpack_from("<II", data, 12)
    assert json_type == 0x4E4F534A
    js = json.loads(data[20 : 20 + json_len])
    bin_off = 20 + json_len
    if bin_off % 4:
        bin_off += 4 - (bin_off % 4)
    bin_len, bin_type = struct.unpack_from("<II", data, bin_off)
    assert bin_type == 0x004E4942
    old_bin = bytearray(data[bin_off + 8 : bin_off + 8 + bin_len])

    bv_index = js["images"][0]["bufferView"]
    bv = js["bufferViews"][bv_index]
    old_off = bv["byteOffset"]
    old_len = bv["byteLength"]
    new_img = bytearray(new_png)
    delta = len(new_img) - old_len
    bv["byteLength"] = len(new_img)
    for i, view in enumerate(js["bufferViews"]):
        if i == bv_index:
            continue
        off = view.get("byteOffset", 0)
        if off > old_off:
            view["byteOffset"] = off + delta
    new_bin = old_bin[:old_off] + new_img + old_bin[old_off + old_len :]
    while len(new_bin) % 4:
        new_bin += b"\x00"
    js["buffers"][0]["byteLength"] = len(new_bin)
    if material_name is not None and js.get("materials"):
        js["materials"][0]["name"] = material_name

    json_bytes = json.dumps(js, separators=(",", ":")).encode("utf-8")
    while len(json_bytes) % 4:
        json_bytes += b" "

    out = bytearray()
    out += b"glTF"
    out += struct.pack("<I", 2)
    total = 12 + 8 + len(json_bytes) + 8 + len(new_bin)
    out += struct.pack("<I", total)
    out += struct.pack("<II", len(json_bytes), 0x4E4F534A)
    out += json_bytes
    out += struct.pack("<II", len(new_bin), 0x004E4942)
    out += new_bin
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(bytes(out))
    print(f"wrote {out_path} (texture delta {delta:+d})")
