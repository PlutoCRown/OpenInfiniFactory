"""生成石膏材料贴图，并把贴图打进石膏斜坡 GLB。

  gypsum        → assets/material_blocks/gypsum/texture.png（单位立方体贴图）
  gypsum_slope  → 以 quartz_slope 网格为底，替换内嵌 PNG

用法：
  python3 scripts/material_blocks/generate_gypsum_texture.py

图标请烘焙：
  ./scripts/bake_scene_icons.sh --materials-only --only gypsum
  ./scripts/bake_scene_icons.sh --materials-only --only gypsum_slope
"""

from __future__ import annotations

import hashlib
import io
import json
import random
import struct
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
MAT_ROOT = ROOT / "assets" / "material_blocks"
SLOPE_SRC = ROOT / "assets" / "scene_blocks" / "quartz_slope" / "model.glb"


def gypsum_texture(seed: int = 7) -> Image.Image:
    """32×32 暖白石膏：浅层理 + 气孔 + 粉笔亮点（有意区别于石英灰脉）。"""
    rng = random.Random(seed)
    img = Image.new("RGB", (32, 32))
    px = img.load()
    for y in range(32):
        for x in range(32):
            n = hashlib.md5(bytes([x, y, seed & 0xFF])).digest()[0] / 255.0
            r = int(228 + n * 18 + rng.uniform(-4, 4))
            g = int(220 + n * 14 + rng.uniform(-4, 4))
            b = int(205 + n * 12 + rng.uniform(-5, 3))
            if y % 8 < 1:
                r -= 8
                g -= 7
                b -= 6
            px[x, y] = (
                max(0, min(255, r)),
                max(0, min(255, g)),
                max(0, min(255, b)),
            )
    for _ in range(48):
        x, y = rng.randrange(32), rng.randrange(32)
        shade = rng.randint(-28, -10)
        r, g, b = px[x, y]
        px[x, y] = (max(0, r + shade), max(0, g + shade), max(0, b + shade))
    for _ in range(20):
        x, y = rng.randrange(32), rng.randrange(32)
        r, g, b = px[x, y]
        px[x, y] = (min(255, r + 22), min(255, g + 20), min(255, b + 16))
    return img


def png_bytes(img: Image.Image) -> bytes:
    buf = io.BytesIO()
    img.save(buf, format="PNG", optimize=True)
    return buf.getvalue()


def replace_glb_texture(
    glb_path: Path, new_png: bytes, out_path: Path, material_name: str
) -> None:
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
    if js.get("materials"):
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


def main() -> None:
    gypsum_dir = MAT_ROOT / "gypsum"
    gypsum_dir.mkdir(parents=True, exist_ok=True)
    tex = gypsum_texture(seed=7)
    tex.save(gypsum_dir / "texture.png", optimize=True)
    print(f"wrote {gypsum_dir / 'texture.png'}")
    model = gypsum_dir / "model.glb"
    if model.exists():
        model.unlink()
        print(f"removed {model} (use texture cube)")

    if not SLOPE_SRC.is_file():
        raise SystemExit(f"missing slope source mesh: {SLOPE_SRC}")
    slope_dir = MAT_ROOT / "gypsum_slope"
    slope_dir.mkdir(parents=True, exist_ok=True)
    tex2 = gypsum_texture(seed=11)
    replace_glb_texture(
        SLOPE_SRC, png_bytes(tex2), slope_dir / "model.glb", "gypsum_slope"
    )
    print(
        "done — bake icons with ./scripts/bake_scene_icons.sh --materials-only --only gypsum"
    )


if __name__ == "__main__":
    main()
