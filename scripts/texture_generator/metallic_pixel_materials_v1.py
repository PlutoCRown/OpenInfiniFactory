"""生成材料方块像素风纹理 + 等距立方体图标"""
from __future__ import annotations
import math
import struct
import zlib
from pathlib import Path
from PIL import Image, ImageDraw

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


def shade(rgb, n, amount):
    d = (n - 128) * amount / 128
    return tuple(clamp(c + d) for c in rgb)


def lerp(a, b, t):
    return tuple(clamp(a[i] + (b[i] - a[i]) * t) for i in range(3))


# --- per-material pixel generators ---

def basic_pixel(x, y):
    """米黄陶土 / 可加工基础料：颗粒 + 微裂纹"""
    n = noise(x, y, 131)
    n2 = noise(x // 2, y // 2, 77)
    base = (214, 186, 118)
    dark = (168, 132, 78)
    light = (236, 214, 158)
    t = n / 255.0
    c = lerp(dark, light, 0.35 + 0.45 * t)
    # 砂砾
    if ((x * 11 + y * 17 + n) % 23) < 3:
        c = shade(c, n, 40)
    # 细裂纹
    if (x + y * 2 + n2 // 40) % 17 == 0:
        c = shade(c, 40, -50)
    # 边缘稍暗，有块体质感
    edge = min(x, y, SIZE - 1 - x, SIZE - 1 - y)
    if edge < 2:
        c = shade(c, 80, -28)
    return c


def iron_pixel(x, y):
    """拉丝金属铁：水平刷痕 + 高光条 + 铆钉感斑点"""
    n = noise(x, y, 149)
    # 拉丝：按行微偏移亮度
    brush = noise(x, y // 2, 401)
    row = noise(0, y, 503)
    base = (152, 160, 168)
    # 金属各向异性：横向拉丝
    streak = ((brush + row) % 256) / 255.0
    c = lerp((110, 118, 126), (198, 204, 210), 0.25 + 0.55 * streak)
    c = shade(c, n, 12)
    # 高光带
    if (y + brush // 60) % 8 == 2:
        c = lerp(c, (230, 234, 238), 0.55)
    # 暗划痕
    if (x * 3 + y + n) % 41 < 2:
        c = shade(c, 30, -45)
    # 冷调斑点
    if ((x * 13 + y * 7 + n) % 37) < 2:
        c = lerp(c, (90, 100, 120), 0.35)
    edge = min(x, y, SIZE - 1 - x, SIZE - 1 - y)
    if edge == 0:
        c = shade(c, 60, -40)
    return c


def copper_pixel(x, y):
    """铜色金属：暖高光 + 绿锈斑"""
    n = noise(x, y, 167)
    brush = noise(x, y // 2, 619)
    base_hi = (232, 148, 86)
    base_lo = (148, 72, 42)
    streak = ((brush + noise(0, y, 701)) % 256) / 255.0
    c = lerp(base_lo, base_hi, 0.3 + 0.5 * streak)
    c = shade(c, n, 16)
    # 铜绿锈斑
    if ((x * 9 + y * 13 + n) % 29) < 3:
        rust = (64, 140, 108) if n > 140 else (48, 110, 88)
        c = lerp(c, rust, 0.55)
    # 暖高光
    if (y + brush // 50) % 9 == 1:
        c = lerp(c, (255, 200, 140), 0.4)
    # 深划痕
    if (x + y * 3 + n) % 47 < 2:
        c = shade(c, 20, -55)
    edge = min(x, y, SIZE - 1 - x, SIZE - 1 - y)
    if edge == 0:
        c = shade(c, 50, -35)
    return c


def glass_pixel(x, y):
    """玻璃：淡青底 + 对角高光 + 边框折射感（不透明占位，视觉像玻璃）"""
    n = noise(x, y, 96)
    base = (168, 214, 228)
    deep = (120, 176, 200)
    cx = (SIZE - 1) / 2
    dx = (x - cx) / cx
    dy = (y - cx) / cx
    r = math.sqrt(dx * dx + dy * dy)
    c = lerp(base, deep, min(1.0, r * 0.55))
    c = shade(c, n, 10)
    # 对角高光带
    if abs(x - y) <= 1 or abs(x + y - (SIZE - 1)) <= 0:
        c = lerp(c, (245, 252, 255), 0.65)
    elif abs(x - y) <= 3:
        c = lerp(c, (220, 240, 250), 0.35)
    # 边框暗线（窗框感）
    edge = min(x, y, SIZE - 1 - x, SIZE - 1 - y)
    if edge < 2:
        c = lerp(c, (90, 140, 160), 0.45 if edge == 0 else 0.2)
    # 内部气泡
    if ((x * 5 + y * 11 + n) % 53) == 0:
        c = lerp(c, (255, 255, 255), 0.5)
    return c


MATERIALS = {
    "basic": basic_pixel,
    "iron": iron_pixel,
    "copper": copper_pixel,
    "glass_material": glass_pixel,
}


def make_texture(fn) -> Image.Image:
    img = Image.new("RGB", (SIZE, SIZE))
    px = img.load()
    for y in range(SIZE):
        for x in range(SIZE):
            px[x, y] = fn(x, y)
    return img


def sample_tex(tex: Image.Image, u, v):
    """u,v in 0..1"""
    x = int(u * (SIZE - 1)) % SIZE
    y = int(v * (SIZE - 1)) % SIZE
    return tex.getpixel((x, y))


def make_icon(tex: Image.Image) -> Image.Image:
    """简易等距立方体图标（128x128），三面贴同一纹理"""
    icon = Image.new("RGBA", (ICON, ICON), (0, 0, 0, 0))
    px = icon.load()
    # 立方体投影参数（参考 bake 取景，尽量铺满）
    cx, cy = ICON // 2, ICON // 2 + 8
    scale = 38  # 半边投影尺度

    def project(x, y, z):
        # 经典游戏等距：
        sx = cx + (x - z) * scale
        sy = cy + (x + z) * scale * 0.5 - y * scale
        return sx, sy

    # 三面：顶(+Y)、左(-X 倾向)、右(+Z 倾向)
    # 用画家算法：先远后近 — 左、右、顶
    faces = []
    # left face (x=-1): u along z, v along y
    for i in range(SIZE):
        for j in range(SIZE):
            u, v = i / (SIZE - 1), j / (SIZE - 1)
            # corners of texel quad on left face: x=-1, z=u*2-1, y=1-v*2
            z0, z1 = u * 2 - 1, (i + 1) / SIZE * 2 - 1
            y0, y1 = 1 - v * 2, 1 - (j + 1) / SIZE * 2
            color = tex.getpixel((i, j))
            faces.append(("L", z0, z1, y0, y1, color))

    # Rasterize filled faces more simply with barycentric-ish scan
    def fill_quad(p0, p1, p2, p3, color, shade_mul):
        # bounding box
        pts = [p0, p1, p2, p3]
        xs = [p[0] for p in pts]
        ys = [p[1] for p in pts]
        minx, maxx = int(min(xs)), int(max(xs)) + 1
        miny, maxy = int(min(ys)), int(max(ys)) + 1
        c = tuple(clamp(ch * shade_mul) for ch in color) + (255,)
        for yy in range(max(0, miny), min(ICON, maxy)):
            for xx in range(max(0, minx), min(ICON, maxx)):
                # point in quad via two triangles
                if point_in_tri((xx + 0.5, yy + 0.5), p0, p1, p2) or point_in_tri(
                    (xx + 0.5, yy + 0.5), p0, p2, p3
                ):
                    px[xx, yy] = c

    def point_in_tri(p, a, b, c):
        def sign(p1, p2, p3):
            return (p1[0] - p3[0]) * (p2[1] - p3[1]) - (p2[0] - p3[0]) * (p1[1] - p3[1])

        d1, d2, d3 = sign(p, a, b), sign(p, b, c), sign(p, c, a)
        has_neg = (d1 < 0) or (d2 < 0) or (d3 < 0)
        has_pos = (d1 > 0) or (d2 > 0) or (d3 > 0)
        return not (has_neg and has_pos)

    # Draw faces from back to front using texel quads
    # Right face (+Z): brighter
    for i in range(SIZE):
        for j in range(SIZE):
            u0, u1 = i / SIZE, (i + 1) / SIZE
            v0, v1 = j / SIZE, (j + 1) / SIZE
            # right face: z=+1, x from -1..1, y from 1..-1
            x0, x1 = u0 * 2 - 1, u1 * 2 - 1
            y0, y1 = 1 - v0 * 2, 1 - v1 * 2
            color = tex.getpixel((i, j))
            p0 = project(x0, y0, 1)
            p1 = project(x1, y0, 1)
            p2 = project(x1, y1, 1)
            p3 = project(x0, y1, 1)
            fill_quad(p0, p1, p2, p3, color, 0.92)

    # Left face (-X / -Z side in iso): darker
    for i in range(SIZE):
        for j in range(SIZE):
            u0, u1 = i / SIZE, (i + 1) / SIZE
            v0, v1 = j / SIZE, (j + 1) / SIZE
            z0, z1 = u0 * 2 - 1, u1 * 2 - 1
            y0, y1 = 1 - v0 * 2, 1 - v1 * 2
            color = tex.getpixel((i, j))
            p0 = project(-1, y0, z0)
            p1 = project(-1, y0, z1)
            p2 = project(-1, y1, z1)
            p3 = project(-1, y1, z0)
            fill_quad(p0, p1, p2, p3, color, 0.72)

    # Top face (+Y)
    for i in range(SIZE):
        for j in range(SIZE):
            u0, u1 = i / SIZE, (i + 1) / SIZE
            v0, v1 = j / SIZE, (j + 1) / SIZE
            x0, x1 = u0 * 2 - 1, u1 * 2 - 1
            z0, z1 = v0 * 2 - 1, v1 * 2 - 1
            color = tex.getpixel((i, j))
            p0 = project(x0, 1, z0)
            p1 = project(x1, 1, z0)
            p2 = project(x1, 1, z1)
            p3 = project(x0, 1, z1)
            fill_quad(p0, p1, p2, p3, color, 1.08)

    return icon


for mid, fn in MATERIALS.items():
    d = ROOT / mid
    d.mkdir(parents=True, exist_ok=True)
    tex = make_texture(fn)
    tex.save(d / "texture.png", optimize=True)
    icon = make_icon(tex)
    icon.save(d / "icon.png", optimize=True)
    print(f"wrote {mid}: texture {tex.size}, icon {icon.size}")

print("done")
