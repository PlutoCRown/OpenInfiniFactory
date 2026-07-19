"""只重做玻璃：更像磨砂玻璃面板，减少大 X 感"""
import math
from pathlib import Path
from PIL import Image

SIZE = 32
ICON = 128
d = Path("assets/material_blocks/glass_material")

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
    # 窄高光条（像玻璃反射），不要整条对角叉
    band = abs((x - y) - 2)
    if band <= 0:
        c = lerp(c, (255, 255, 255), 0.75)
    elif band <= 1:
        c = lerp(c, (235, 248, 255), 0.4)
    # 第二条高光点
    if band >= 3 and abs(x + y - (SIZE + 4)) <= 0:
        c = lerp(c, (255, 255, 255), 0.35)
    # 磨砂微噪
    if n > 200:
        c = lerp(c, (255, 255, 255), 0.15)
    elif n < 40:
        c = lerp(c, (130, 180, 200), 0.2)
    # 气泡
    if ((x * 5 + y * 11 + n) % 59) == 0:
        c = (255, 255, 255)
    return c

tex = Image.new("RGB", (SIZE, SIZE))
px = tex.load()
for y in range(SIZE):
    for x in range(SIZE):
        px[x, y] = glass_pixel(x, y)
tex.save(d / "texture.png", optimize=True)

# 复用上一版 icon 逻辑（简化：直接再跑一遍小脚本）
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

icon = Image.new("RGBA", (ICON, ICON), (0, 0, 0, 0))
ipx = icon.load()
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
        fill_quad(ipx, project(-1, y0, z0), project(-1, y0, z1), project(-1, y1, z1), project(-1, y1, z0), c)
for i in range(SIZE):
    for j in range(SIZE):
        u0, u1 = i / SIZE, (i + 1) / SIZE
        v0, v1 = j / SIZE, (j + 1) / SIZE
        x0, x1 = -1 + u0 * 2, -1 + u1 * 2
        y0, y1 = 1 - v0 * 2, 1 - v1 * 2
        c = shade_color(tex.getpixel((i, j)), 0.88)
        fill_quad(ipx, project(x0, y0, 1), project(x1, y0, 1), project(x1, y1, 1), project(x0, y1, 1), c)
for i in range(SIZE):
    for j in range(SIZE):
        u0, u1 = i / SIZE, (i + 1) / SIZE
        v0, v1 = j / SIZE, (j + 1) / SIZE
        x0, x1 = -1 + u0 * 2, -1 + u1 * 2
        z0, z1 = -1 + v0 * 2, -1 + v1 * 2
        c = shade_color(tex.getpixel((i, j)), 1.12)
        fill_quad(ipx, project(x0, 1, z0), project(x1, 1, z0), project(x1, 1, z1), project(x0, 1, z1), c)

icon.save(d / "icon.png", optimize=True)
print("glass refreshed")
