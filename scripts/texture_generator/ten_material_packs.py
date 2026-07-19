"""生成约 10 种材料：纹理 + 图标 + meta.json"""
from __future__ import annotations
import json
import math
from pathlib import Path
from PIL import Image

SIZE = 32
ICON = 128
ROOT = Path("assets/material_blocks")
SCHEMA = "../../../schemas/material_block.meta.schema.json"


def clamp(v):
    return max(0, min(255, int(round(v))))


def noise(x, y, seed):
    v = (x * 73856093 + y * 19349663 + seed * 83492791) & 0xFFFFFFFF
    v ^= v >> 13
    v = (v * 1274126177) & 0xFFFFFFFF
    return (v ^ (v >> 16)) & 0xFF


def lerp(a, b, t):
    t = max(0.0, min(1.0, t))
    return tuple(clamp(a[i] + (b[i] - a[i]) * t) for i in range(3))


def shade(rgb, amount):
    return tuple(clamp(c + amount) for c in rgb)


# ---- pixel generators ----

def basic_pixel(x, y):
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


def iron_pixel(x, y):
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


def copper_pixel(x, y):
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


def glass_pixel(x, y):
    edge = min(x, y, SIZE - 1 - x, SIZE - 1 - y)
    n = noise(x, y, 96)
    if edge == 0:
        return (55, 105, 125)
    if edge == 1:
        return (120, 175, 198)
    cx = (SIZE - 1) / 2
    r = math.hypot(x - cx, y - cx) / (SIZE * 0.75)
    c = lerp((210, 236, 245), (150, 198, 218), min(1.0, r * 0.85))
    band = abs((x - y) - 2)
    if band <= 0:
        c = lerp(c, (255, 255, 255), 0.75)
    elif band <= 1:
        c = lerp(c, (235, 248, 255), 0.4)
    if band >= 3 and abs(x + y - (SIZE + 4)) <= 0:
        c = lerp(c, (255, 255, 255), 0.35)
    if n > 200:
        c = lerp(c, (255, 255, 255), 0.15)
    elif n < 40:
        c = lerp(c, (130, 180, 200), 0.2)
    if ((x * 5 + y * 11 + n) % 59) == 0:
        c = (255, 255, 255)
    return c


def gold_pixel(x, y):
    """金：暖黄拉丝 + 强高光"""
    phase = noise(0, y, 777)
    wave = math.sin((x + phase / 30.0) * 1.0) * 0.5 + 0.5
    c = lerp((140, 90, 20), (255, 220, 100), 0.3 + 0.55 * wave)
    if y % 6 == 1:
        c = lerp(c, (255, 245, 180), 0.6)
    if (x + phase // 16) % 9 == 0:
        c = shade(c, -35)
    n = noise(x, y, 333)
    if ((x * 3 + y * 7 + n) % 43) == 0:
        c = lerp(c, (255, 255, 220), 0.5)
    if min(x, y, SIZE - 1 - x, SIZE - 1 - y) == 0:
        c = shade(c, -40)
    return c


def aluminum_pixel(x, y):
    """铝：冷白拉丝金属"""
    phase = noise(0, y, 1200)
    wave = math.sin((x + phase / 45.0) * 1.1) * 0.5 + 0.5
    c = lerp((160, 168, 176), (236, 240, 244), 0.3 + 0.5 * wave)
    if y % 5 == 0:
        c = lerp(c, (255, 255, 255), 0.45)
    if (x + phase // 22) % 10 == 0:
        c = shade(c, -22)
    if min(x, y, SIZE - 1 - x, SIZE - 1 - y) == 0:
        c = shade(c, -25)
    return c


def wood_pixel(x, y):
    """木板：竖向木纹 + 年轮感"""
    n = noise(x, y, 211)
    grain = noise(x // 2, y, 800)
    # 竖纹
    wave = math.sin(x * 0.55 + grain / 80.0) * 0.5 + 0.5
    c = lerp((96, 58, 28), (186, 122, 58), 0.25 + 0.55 * wave)
    # 深木纹线
    if (x + grain // 30) % 7 == 0:
        c = shade(c, -30)
    # 节疤
    knot = noise(x // 4, y // 4, 999)
    if knot > 230 and abs(x % 8 - 4) < 2 and abs(y % 10 - 5) < 2:
        c = lerp(c, (60, 36, 18), 0.7)
    if ((x * 5 + y * 3 + n) % 29) < 2:
        c = shade(c, 12 if n > 128 else -14)
    if min(x, y, SIZE - 1 - x, SIZE - 1 - y) == 0:
        c = shade(c, -20)
    return c


def stone_pixel(x, y):
    """石材：不规则碎石块"""
    n = noise(x, y, 404)
    cell = noise(x // 5, y // 5, 505)
    base = lerp((110, 112, 118), (170, 172, 178), cell / 255.0)
    c = shade(base, (n - 128) / 8)
    # 裂缝
    if (x + y * 2 + cell // 20) % 13 == 0:
        c = shade(c, -40)
    if ((x * 9 + y * 5 + n) % 31) < 3:
        c = shade(c, 20)
    # 青苔点
    if n > 220 and cell % 3 == 0:
        c = lerp(c, (90, 120, 70), 0.35)
    if min(x, y, SIZE - 1 - x, SIZE - 1 - y) == 0:
        c = shade(c, -25)
    return c


def coal_pixel(x, y):
    """煤炭：深黑晶面 + 微光"""
    n = noise(x, y, 606)
    facet = noise(x // 3, y // 3, 707)
    c = lerp((18, 18, 22), (55, 55, 62), facet / 255.0)
    c = shade(c, (n - 128) / 10)
    # 晶面高光
    if facet > 200 and ((x + y) % 4 == 0):
        c = lerp(c, (90, 95, 110), 0.55)
    if ((x * 7 + y * 11 + n) % 53) == 0:
        c = (120, 125, 140)
    if min(x, y, SIZE - 1 - x, SIZE - 1 - y) == 0:
        c = (8, 8, 10)
    return c


def crystal_pixel(x, y):
    """水晶：紫青晶簇感（脆弱）"""
    n = noise(x, y, 808)
    cx = (SIZE - 1) / 2
    # 放射状晶面
    ang = math.atan2(y - cx, x - cx)
    radial = (math.sin(ang * 3.0) * 0.5 + 0.5)
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


MATERIALS = [
    # id, fragile, color_rgb 0-255, pixel_fn
    ("basic", False, (214, 186, 118), basic_pixel),
    ("iron", False, (160, 168, 176), iron_pixel),
    ("copper", False, (200, 110, 58), copper_pixel),
    ("glass_material", True, (168, 214, 228), glass_pixel),
    ("gold", False, (232, 190, 70), gold_pixel),
    ("aluminum", False, (200, 208, 216), aluminum_pixel),
    ("wood", False, (150, 95, 48), wood_pixel),
    ("stone", False, (140, 142, 148), stone_pixel),
    ("coal", False, (36, 36, 40), coal_pixel),
    ("crystal", True, (140, 120, 220), crystal_pixel),
]


def make_texture(fn):
    img = Image.new("RGB", (SIZE, SIZE))
    px = img.load()
    for y in range(SIZE):
        for x in range(SIZE):
            px[x, y] = fn(x, y)
    return img


def point_in_tri(p, a, b, c):
    def cross(o, p1, p2):
        return (p1[0] - o[0]) * (p2[1] - o[1]) - (p1[1] - o[1]) * (p2[0] - o[0])
    b1 = cross(a, b, p) < 0.0
    b2 = cross(b, c, p) < 0.0
    b3 = cross(c, a, p) < 0.0
    return b1 == b2 == b3


def fill_quad(px, p0, p1, p2, p3, color):
    pts = [p0, p1, p2, p3]
    minx = max(0, int(min(p[0] for p in pts)))
    maxx = min(ICON - 1, int(max(p[0] for p in pts)) + 1)
    miny = max(0, int(min(p[1] for p in pts)))
    maxy = min(ICON - 1, int(max(p[1] for p in pts)) + 1)
    rgba = color + (255,)
    for yy in range(miny, maxy + 1):
        for xx in range(minx, maxx + 1):
            p = (xx + 0.5, yy + 0.5)
            if point_in_tri(p, p0, p1, p2) or point_in_tri(p, p0, p2, p3):
                px[xx, yy] = rgba


def make_icon(tex: Image.Image) -> Image.Image:
    icon = Image.new("RGBA", (ICON, ICON), (0, 0, 0, 0))
    px = icon.load()
    cx, cy, s = ICON * 0.5, ICON * 0.58, 46.0

    def project(x, y, z):
        return (cx + (x - z) * s * 0.86, cy + (x + z) * s * 0.5 - y * s)

    def shade_color(rgb, mul):
        return tuple(clamp(c * mul) for c in rgb)

    for i in range(SIZE):
        for j in range(SIZE):
            u0, u1 = i / SIZE, (i + 1) / SIZE
            v0, v1 = j / SIZE, (j + 1) / SIZE
            z0, z1 = -1 + u0 * 2, -1 + u1 * 2
            y0, y1 = 1 - v0 * 2, 1 - v1 * 2
            c = shade_color(tex.getpixel((i, j)), 0.70)
            fill_quad(px, project(-1, y0, z0), project(-1, y0, z1), project(-1, y1, z1), project(-1, y1, z0), c)
    for i in range(SIZE):
        for j in range(SIZE):
            u0, u1 = i / SIZE, (i + 1) / SIZE
            v0, v1 = j / SIZE, (j + 1) / SIZE
            x0, x1 = -1 + u0 * 2, -1 + u1 * 2
            y0, y1 = 1 - v0 * 2, 1 - v1 * 2
            c = shade_color(tex.getpixel((i, j)), 0.88)
            fill_quad(px, project(x0, y0, 1), project(x1, y0, 1), project(x1, y1, 1), project(x0, y1, 1), c)
    for i in range(SIZE):
        for j in range(SIZE):
            u0, u1 = i / SIZE, (i + 1) / SIZE
            v0, v1 = j / SIZE, (j + 1) / SIZE
            x0, x1 = -1 + u0 * 2, -1 + u1 * 2
            z0, z1 = -1 + v0 * 2, -1 + v1 * 2
            c = shade_color(tex.getpixel((i, j)), 1.12)
            fill_quad(px, project(x0, 1, z0), project(x1, 1, z0), project(x1, 1, z1), project(x0, 1, z1), c)
    return icon


for mid, fragile, color, fn in MATERIALS:
    d = ROOT / mid
    d.mkdir(parents=True, exist_ok=True)
    meta = {
        "$schema": SCHEMA,
        "id": mid,
        "fragile": fragile,
        "directional": False,
        "connectable": [True] * 6,
    }
    (d / "meta.json").write_text(json.dumps(meta, indent=2) + "\n")
    tex = make_texture(fn)
    tex.save(d / "texture.png", optimize=True)
    make_icon(tex).save(d / "icon.png", optimize=True)
    print(f"  {mid:16s} fragile={fragile}")

print(f"total {len(MATERIALS)} materials")
# emit rust/color helper data
print("COLORS = {")
for mid, _, color, _ in MATERIALS:
    print(f'  "{mid}": {color},')
print("}")
