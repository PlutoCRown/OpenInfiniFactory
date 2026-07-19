"""重做材料纹理：更明确的拉丝金属 / 陶土 / 玻璃，并修等距图标"""
from __future__ import annotations
import math
from pathlib import Path
from PIL import Image

SIZE = 32
ICON = 128
ROOT = Path("assets/material_blocks")


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


def basic_pixel(x, y):
    """陶土砖：暖米色 + 砂浆缝 + 颗粒（非金属）"""
    # 4x4 砖块网格
    brick_w, brick_h = 8, 6
    # 交错砌法
    row = y // brick_h
    ox = (brick_w // 2) if row % 2 else 0
    lx = (x + ox) % brick_w
    ly = y % brick_h
    mortar = lx == 0 or ly == 0
    n = noise(x, y, 131)
    if mortar:
        return lerp((120, 100, 78), (150, 128, 96), n / 255.0)
    # 砖面渐变：左上亮右下暗
    t = 0.35 + 0.4 * (1.0 - lx / brick_w) + 0.15 * (1.0 - ly / brick_h)
    c = lerp((168, 128, 72), (236, 206, 148), t)
    if ((x * 7 + y * 11 + n) % 19) < 2:
        c = shade(c, 18 if n > 128 else -22)
    return c


def iron_pixel(x, y):
    """拉丝钢：清晰横向刷痕 + 高光带 + 暗槽"""
    n = noise(x, y, 149)
    # 每行一种刷痕相位
    phase = noise(0, y, 9001)
    # 横向周期波（刷痕）
    wave = math.sin((x + phase / 40.0) * 0.9) * 0.5 + 0.5
    # 行间对比
    row_tint = ((phase % 5) - 2) * 6
    lo = (98, 108, 118)
    hi = (210, 216, 222)
    c = lerp(lo, hi, 0.25 + 0.55 * wave)
    c = shade(c, row_tint)
    # 主高光带（每隔若干行）
    if y % 7 == 2:
        c = lerp(c, (240, 244, 248), 0.55)
    if y % 7 == 3:
        c = lerp(c, (180, 188, 196), 0.35)
    # 细暗槽
    if (x + phase // 20) % 11 == 0:
        c = shade(c, -28)
    # 少量杂质
    if ((x * 13 + y * 5 + n) % 61) == 0:
        c = lerp(c, (70, 78, 95), 0.4)
    # 边框微暗
    if min(x, y, SIZE - 1 - x, SIZE - 1 - y) == 0:
        c = shade(c, -30)
    return c


def copper_pixel(x, y):
    """拉丝铜：暖金属刷痕 + 局部铜绿"""
    n = noise(x, y, 167)
    phase = noise(0, y, 4242)
    wave = math.sin((x + phase / 35.0) * 0.85) * 0.5 + 0.5
    lo = (120, 58, 32)
    hi = (236, 150, 88)
    c = lerp(lo, hi, 0.28 + 0.52 * wave)
    if y % 8 == 1:
        c = lerp(c, (255, 200, 140), 0.45)
    if y % 8 == 2:
        c = shade(c, -18)
    if (x + phase // 18) % 13 == 0:
        c = shade(c, -32)
    # 铜绿斑（成团，不是撒盐）
    blot = noise(x // 3, y // 3, 555)
    if blot > 210 and ((x + y) % 3 == 0):
        c = lerp(c, (48, 130, 100), 0.55)
    elif blot > 190 and n > 180:
        c = lerp(c, (70, 150, 120), 0.3)
    if min(x, y, SIZE - 1 - x, SIZE - 1 - y) == 0:
        c = shade(c, -28)
    return c


def glass_pixel(x, y):
    """玻璃面板：淡青底、边框、对角高光、气泡"""
    n = noise(x, y, 96)
    edge = min(x, y, SIZE - 1 - x, SIZE - 1 - y)
    # 外框
    if edge == 0:
        return (70, 120, 140)
    if edge == 1:
        return (140, 190, 210)
    # 径向略深
    cx = (SIZE - 1) / 2
    r = math.hypot(x - cx, y - cx) / (SIZE * 0.7)
    c = lerp((200, 230, 240), (140, 190, 210), min(1.0, r))
    # 对角高光（窗玻璃反光）
    d1 = abs(x - y)
    d2 = abs(x + y - (SIZE - 1))
    if d1 <= 1 or d2 <= 1:
        c = lerp(c, (255, 255, 255), 0.7)
    elif d1 <= 2 or d2 <= 2:
        c = lerp(c, (230, 245, 255), 0.4)
    # 稀少气泡
    if ((x * 5 + y * 11 + n) % 47) == 0:
        c = (255, 255, 255)
    elif ((x * 5 + y * 11 + n) % 47) == 1:
        c = lerp(c, (180, 210, 230), 0.5)
    return c


MATERIALS = {
    "basic": basic_pixel,
    "iron": iron_pixel,
    "copper": copper_pixel,
    "glass_material": glass_pixel,
}


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
    # 与 bake_scene_icons 接近的取景：立方体尽量铺满
    cx, cy = ICON * 0.5, ICON * 0.58
    s = 46.0

    def project(x, y, z):
        # x,y,z in [-1,1]
        sx = cx + (x - z) * s * 0.86
        sy = cy + (x + z) * s * 0.5 - y * s
        return (sx, sy)

    def shade_color(rgb, mul):
        return tuple(clamp(c * mul) for c in rgb)

    # 绘制顺序：左(暗) → 右 → 顶(亮)
    # Left face: x = -1
    for i in range(SIZE):
        for j in range(SIZE):
            u0, u1 = i / SIZE, (i + 1) / SIZE
            v0, v1 = j / SIZE, (j + 1) / SIZE
            z0, z1 = -1 + u0 * 2, -1 + u1 * 2
            y0, y1 = 1 - v0 * 2, 1 - v1 * 2
            c = shade_color(tex.getpixel((i, j)), 0.70)
            fill_quad(
                px,
                project(-1, y0, z0),
                project(-1, y0, z1),
                project(-1, y1, z1),
                project(-1, y1, z0),
                c,
            )

    # Right face: z = +1
    for i in range(SIZE):
        for j in range(SIZE):
            u0, u1 = i / SIZE, (i + 1) / SIZE
            v0, v1 = j / SIZE, (j + 1) / SIZE
            x0, x1 = -1 + u0 * 2, -1 + u1 * 2
            y0, y1 = 1 - v0 * 2, 1 - v1 * 2
            c = shade_color(tex.getpixel((i, j)), 0.88)
            fill_quad(
                px,
                project(x0, y0, 1),
                project(x1, y0, 1),
                project(x1, y1, 1),
                project(x0, y1, 1),
                c,
            )

    # Top face: y = +1
    for i in range(SIZE):
        for j in range(SIZE):
            u0, u1 = i / SIZE, (i + 1) / SIZE
            v0, v1 = j / SIZE, (j + 1) / SIZE
            x0, x1 = -1 + u0 * 2, -1 + u1 * 2
            z0, z1 = -1 + v0 * 2, -1 + v1 * 2
            c = shade_color(tex.getpixel((i, j)), 1.12)
            fill_quad(
                px,
                project(x0, 1, z0),
                project(x1, 1, z0),
                project(x1, 1, z1),
                project(x0, 1, z1),
                c,
            )

    return icon


for mid, fn in MATERIALS.items():
    d = ROOT / mid
    d.mkdir(parents=True, exist_ok=True)
    # 保留 meta，只换图
    tex = make_texture(fn)
    tex.save(d / "texture.png", optimize=True)
    icon = make_icon(tex)
    icon.save(d / "icon.png", optimize=True)
    print(mid, "ok", (d / "texture.png").stat().st_size, "bytes")

print("done")
