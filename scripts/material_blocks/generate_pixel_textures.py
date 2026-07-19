"""生成普通材料 32×32 像素风 texture.png（对话里 ten_material_packs 定稿）。

覆盖：basic / iron / copper / glass_material / gold / aluminum / wood / stone / coal / crystal

用法：
  python3 scripts/material_blocks/generate_pixel_textures.py
  python3 scripts/material_blocks/generate_pixel_textures.py --only copper,iron

图标请用官方烘焙，不要用手绘：
  ./scripts/bake_scene_icons.sh --materials-only --only copper
"""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
OUT_ROOT = ROOT / "assets" / "material_blocks"
SCHEMA = "../../../schemas/material_block.meta.schema.json"
SIZE = 32


def clamp(v: float) -> int:
    return max(0, min(255, int(round(v))))


def noise(x: int, y: int, seed: int) -> int:
    v = (x * 73856093 + y * 19349663 + seed * 83492791) & 0xFFFFFFFF
    v ^= v >> 13
    v = (v * 1274126177) & 0xFFFFFFFF
    return (v ^ (v >> 16)) & 0xFF


def lerp(
    a: tuple[int, int, int], b: tuple[int, int, int], t: float
) -> tuple[int, int, int]:
    t = max(0.0, min(1.0, t))
    return tuple(clamp(a[i] + (b[i] - a[i]) * t) for i in range(3))


def shade(rgb: tuple[int, int, int], amount: float) -> tuple[int, int, int]:
    return tuple(clamp(c + amount) for c in rgb)


def basic_pixel(x: int, y: int) -> tuple[int, int, int]:
    brick_w, brick_h = 8, 6
    row = y // brick_h
    ox = (brick_w // 2) if row % 2 else 0
    lx = (x + ox) % brick_w
    ly = y % brick_h
    n = noise(x, y, 131)
    if lx == 0 or ly == 0:
        return lerp((120, 100, 78), (150, 128, 96), n / 255.0)
    t = 0.35 + 0.4 * (1.0 - lx / brick_w) + 0.15 * (1.0 - ly / brick_h)
    c = lerp((168, 128, 72), (236, 206, 148), t)
    if ((x * 7 + y * 11 + n) % 19) < 2:
        c = shade(c, 18 if n > 128 else -22)
    return c


def iron_pixel(x: int, y: int) -> tuple[int, int, int]:
    n = noise(x, y, 149)
    phase = noise(0, y, 9001)
    wave = math.sin((x + phase / 40.0) * 0.9) * 0.5 + 0.5
    row_tint = ((phase % 5) - 2) * 6
    c = lerp((98, 108, 118), (210, 216, 222), 0.25 + 0.55 * wave)
    c = shade(c, row_tint)
    if y % 7 == 2:
        c = lerp(c, (240, 244, 248), 0.55)
    if y % 7 == 3:
        c = lerp(c, (180, 188, 196), 0.35)
    if (x + phase // 20) % 11 == 0:
        c = shade(c, -28)
    if ((x * 13 + y * 5 + n) % 61) == 0:
        c = lerp(c, (70, 78, 95), 0.4)
    if min(x, y, SIZE - 1 - x, SIZE - 1 - y) == 0:
        c = shade(c, -30)
    return c


def copper_pixel(x: int, y: int) -> tuple[int, int, int]:
    n = noise(x, y, 167)
    phase = noise(0, y, 4242)
    wave = math.sin((x + phase / 35.0) * 0.85) * 0.5 + 0.5
    c = lerp((120, 58, 32), (236, 150, 88), 0.28 + 0.52 * wave)
    if y % 8 == 1:
        c = lerp(c, (255, 200, 140), 0.45)
    if y % 8 == 2:
        c = shade(c, -18)
    if (x + phase // 18) % 13 == 0:
        c = shade(c, -32)
    blot = noise(x // 3, y // 3, 555)
    if blot > 210 and ((x + y) % 3 == 0):
        c = lerp(c, (48, 130, 100), 0.55)
    elif blot > 190 and n > 180:
        c = lerp(c, (70, 150, 120), 0.3)
    if min(x, y, SIZE - 1 - x, SIZE - 1 - y) == 0:
        c = shade(c, -28)
    return c


def glass_pixel(x: int, y: int) -> tuple[int, int, int]:
    edge = min(x, y, SIZE - 1 - x, SIZE - 1 - y)
    n = noise(x, y, 96)
    if edge == 0:
        return (55, 105, 125)
    if edge == 1:
        return (120, 175, 198)
    cx = (SIZE - 1) / 2
    dx, dy = x - cx, y - cx
    r = math.hypot(dx, dy) / (SIZE * 0.55)
    shine = max(0.0, 1.0 - abs(r - 0.35) * 3.0)
    c = lerp((140, 190, 210), (220, 245, 255), 0.35 + 0.4 * shine)
    if ((x + y + n) % 23) == 0:
        c = lerp(c, (255, 255, 255), 0.35)
    return c


def gold_pixel(x: int, y: int) -> tuple[int, int, int]:
    n = noise(x, y, 333)
    phase = noise(0, y, 777)
    wave = math.sin((x + phase / 30.0) * 0.95) * 0.5 + 0.5
    c = lerp((150, 100, 20), (255, 220, 90), 0.3 + 0.5 * wave)
    if y % 6 == 1:
        c = lerp(c, (255, 245, 180), 0.5)
    if ((x * 9 + y * 3 + n) % 47) == 0:
        c = shade(c, -40)
    if min(x, y, SIZE - 1 - x, SIZE - 1 - y) == 0:
        c = shade(c, -35)
    return c


def aluminum_pixel(x: int, y: int) -> tuple[int, int, int]:
    n = noise(x, y, 401)
    phase = noise(0, y, 1200)
    wave = math.sin((x + phase / 45.0) * 1.05) * 0.5 + 0.5
    c = lerp((150, 158, 168), (230, 236, 242), 0.25 + 0.55 * wave)
    if y % 5 == 2:
        c = lerp(c, (255, 255, 255), 0.4)
    if ((x + phase // 25) % 9) == 0:
        c = shade(c, -22)
    if min(x, y, SIZE - 1 - x, SIZE - 1 - y) == 0:
        c = shade(c, -28)
    return c


def wood_pixel(x: int, y: int) -> tuple[int, int, int]:
    n = noise(x, y, 512)
    grain = noise(x, y // 2, 88)
    ring = abs(math.sin((x + grain / 40.0) * 0.55))
    c = lerp((90, 55, 28), (190, 130, 70), 0.25 + 0.55 * ring)
    if y % 4 == 0:
        c = shade(c, -18)
    if ((x * 5 + y + n) % 29) < 2:
        c = shade(c, -30)
    if min(x, y, SIZE - 1 - x, SIZE - 1 - y) == 0:
        c = shade(c, -25)
    return c


def stone_pixel(x: int, y: int) -> tuple[int, int, int]:
    """花岗岩感灰岩（避免与场景 stone 撞名时可用 granite id；此处写 stone 贴图算法）。"""
    n = noise(x, y, 640)
    n2 = noise(x // 2, y // 2, 641)
    c = lerp((110, 112, 118), (170, 172, 178), n / 255.0)
    if n2 > 200:
        c = lerp(c, (90, 92, 98), 0.45)
    if ((x * 3 + y * 7 + n) % 37) < 2:
        c = shade(c, 25)
    if min(x, y, SIZE - 1 - x, SIZE - 1 - y) == 0:
        c = shade(c, -30)
    return c


def coal_pixel(x: int, y: int) -> tuple[int, int, int]:
    n = noise(x, y, 701)
    c = lerp((18, 18, 20), (55, 55, 60), n / 255.0)
    if ((x * 11 + y * 13 + n) % 19) < 2:
        c = lerp(c, (90, 90, 95), 0.5)
    if min(x, y, SIZE - 1 - x, SIZE - 1 - y) == 0:
        c = (8, 8, 10)
    return c


def crystal_pixel(x: int, y: int) -> tuple[int, int, int]:
    n = noise(x, y, 808)
    cx = (SIZE - 1) / 2
    ang = math.atan2(y - cx, x - cx)
    radial = math.sin(ang * 3.0) * 0.5 + 0.5
    r = math.hypot(x - cx, y - cx) / (SIZE * 0.7)
    c = lerp((90, 50, 160), (160, 230, 255), radial * 0.55 + (1.0 - min(1.0, r)) * 0.45)
    if abs((x - y) % 6) == 0:
        c = lerp(c, (255, 255, 255), 0.45)
    if n > 230:
        c = (255, 255, 255)
    edge = min(x, y, SIZE - 1 - x, SIZE - 1 - y)
    if edge == 0:
        c = lerp(c, (40, 20, 80), 0.5)
    return c


# id → (fragile, pixel_fn)；stone 算法产出 granite 包（避开场景 stone）
MATERIALS = [
    ("basic", False, basic_pixel),
    ("iron", False, iron_pixel),
    ("copper", False, copper_pixel),
    ("glass_material", True, glass_pixel),
    ("gold", False, gold_pixel),
    ("aluminum", False, aluminum_pixel),
    ("wood", False, wood_pixel),
    ("granite", False, stone_pixel),
    ("coal", False, coal_pixel),
    ("crystal", True, crystal_pixel),
]


def make_texture(fn) -> Image.Image:
    img = Image.new("RGB", (SIZE, SIZE))
    px = img.load()
    for y in range(SIZE):
        for x in range(SIZE):
            px[x, y] = fn(x, y)
    return img


def write_pack(mid: str, fragile: bool, fn) -> None:
    d = OUT_ROOT / mid
    d.mkdir(parents=True, exist_ok=True)
    meta = {
        "$schema": SCHEMA,
        "id": mid,
        "fragile": fragile,
        "directional": False,
        "connectable": [True] * 6,
    }
    (d / "meta.json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    make_texture(fn).save(d / "texture.png", optimize=True)
    print(f"wrote {d / 'texture.png'}  fragile={fragile}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--only",
        type=str,
        default="",
        help="逗号分隔材料 id，默认全部",
    )
    parser.add_argument(
        "--write-meta",
        action="store_true",
        help="同时重写 meta.json（默认只写 texture.png）",
    )
    args = parser.parse_args()
    only = {s.strip() for s in args.only.split(",") if s.strip()}

    for mid, fragile, fn in MATERIALS:
        if only and mid not in only:
            continue
        d = OUT_ROOT / mid
        d.mkdir(parents=True, exist_ok=True)
        if args.write_meta:
            write_pack(mid, fragile, fn)
        else:
            make_texture(fn).save(d / "texture.png", optimize=True)
            print(f"wrote {d / 'texture.png'}")
    print("done — icons: ./scripts/bake_scene_icons.sh --materials-only [--only ID]")


if __name__ == "__main__":
    main()
