"""生成纯立方体材料的 texture.png + normal.png（无需 Blender / GLB）。

写入新增材料目录：
  material_1 — 粗框 + 深色碎石填芯
  material_2 — 单回字凹槽金属
  material_3 — 双回字嵌套凹槽
  material_4 — 米色工业面板（双槽 + 角螺孔）

material_5 有独立 model.glb（见 tools/assets/models/material/generate_aluminum_glb.py），不在此生成。

用法：
  python3 tools/assets/textures/generate_face_textures.py
"""

from __future__ import annotations

from pathlib import Path
import sys
_TOOLS = Path(__file__).resolve().parents[1]
if str(_TOOLS) not in sys.path:
    sys.path.insert(0, str(_TOOLS))
from common.paths import REPO_ROOT
from common.png_util import write_png_rgba

import math

OUT_ROOT = REPO_ROOT / "assets" / "material_blocks"
TEX_SIZE = 256

# 框填芯：外框厚度（UV 比例）
FRAME_T = 0.22
# 凹槽半宽 / 深度（写进高度场再转法线）
GROOVE_HALF = 0.018
GROOVE_DEPTH = 0.55
NORMAL_STRENGTH = 8.0


def hash2(ix: int, iy: int) -> float:
    """确定性噪声 [0,1)。"""
    n = (ix * 374761393 + iy * 668265263) & 0x7FFFFFFF
    n = (n ^ (n >> 13)) * 1274126177
    return ((n ^ (n >> 16)) & 0xFFFF) / 65536.0


def value_noise(u: float, v: float, scale: float) -> float:
    x = u * scale
    y = v * scale
    x0, y0 = int(math.floor(x)), int(math.floor(y))
    fx, fy = x - x0, y - y0
    fx = fx * fx * (3.0 - 2.0 * fx)
    fy = fy * fy * (3.0 - 2.0 * fy)
    a = hash2(x0, y0)
    b = hash2(x0 + 1, y0)
    c = hash2(x0, y0 + 1)
    d = hash2(x0 + 1, y0 + 1)
    return a + (b - a) * fx + (c - a) * fy + (a - b - c + d) * fx * fy


def fbm(u: float, v: float, octaves: int = 4) -> float:
    amp, freq, total, norm = 1.0, 1.0, 0.0, 0.0
    for _ in range(octaves):
        total += amp * value_noise(u, v, freq * 6.0)
        norm += amp
        amp *= 0.5
        freq *= 2.0
    return total / norm


def groove_profile(dist: float, center: float, half_w: float) -> float:
    """距 center 越近越高（凹槽），返回 0..1。"""
    t = abs(dist - center) / half_w
    if t >= 1.0:
        return 0.0
    return (1.0 - t * t) ** 2


def edge_dist(u: float, v: float) -> float:
    return min(u, 1.0 - u, v, 1.0 - v)


def height_to_normal(
    height: list[list[float]], size: int, strength: float
) -> list[int]:
    """高度场 → OpenGL 风格法线贴图 RGBA。"""
    rgba = [0] * (size * size * 4)
    for y in range(size):
        for x in range(size):
            h_l = height[y][(x - 1) % size]
            h_r = height[y][(x + 1) % size]
            h_d = height[(y - 1) % size][x]
            h_u = height[(y + 1) % size][x]
            dx = (h_r - h_l) * strength
            dy = (h_u - h_d) * strength
            nx, ny, nz = -dx, -dy, 1.0
            inv = 1.0 / math.sqrt(nx * nx + ny * ny + nz * nz)
            nx, ny, nz = nx * inv, ny * inv, nz * inv
            i = (y * size + x) * 4
            rgba[i] = int((nx * 0.5 + 0.5) * 255)
            rgba[i + 1] = int((ny * 0.5 + 0.5) * 255)
            rgba[i + 2] = int((nz * 0.5 + 0.5) * 255)
            rgba[i + 3] = 255
    return rgba


def brushed_rgb(
    u: float, v: float, base: tuple[int, int, int], amount: float
) -> tuple[int, int, int]:
    """轻微水平拉丝。"""
    n = value_noise(u * 0.15, v * 28.0, 1.0) * 2.0 - 1.0
    return tuple(max(0, min(255, int(c + n * amount * 255))) for c in base)  # type: ignore[return-value]


def apply_ao(rgb: tuple[int, int, int], ao: float) -> tuple[int, int, int]:
    return tuple(max(0, min(255, int(c * ao))) for c in rgb)  # type: ignore[return-value]


def write_pack(subdir: str, albedo: list[int], normal: list[int]) -> None:
    out = OUT_ROOT / subdir
    tex = out / "texture.png"
    nrm = out / "normal.png"
    write_png_rgba(tex, TEX_SIZE, TEX_SIZE, albedo)
    write_png_rgba(nrm, TEX_SIZE, TEX_SIZE, normal)
    print(f"Wrote {tex}", file=sys.stderr)
    print(f"Wrote {nrm}", file=sys.stderr)


def sd_box(u: float, v: float, x0: float, y0: float, x1: float, y1: float) -> float:
    """轴对齐矩形 SDF：内部为负。"""
    dx = max(x0 - u, u - x1)
    dy = max(y0 - v, v - y1)
    out = math.sqrt(max(dx, 0.0) ** 2 + max(dy, 0.0) ** 2)
    insides = max(dx, dy)
    return insides if insides < 0.0 else out


def sd_circle(u: float, v: float, cx: float, cy: float, r: float) -> float:
    return math.hypot(u - cx, v - cy) - r


def soft_step(d: float, w: float) -> float:
    """d<=0 → 1，越过 w 平滑到 0。"""
    if w <= 1e-8:
        return 1.0 if d <= 0.0 else 0.0
    t = clamp01(1.0 - d / w)
    return t * t * (3.0 - 2.0 * t)


def clamp01(x: float) -> float:
    return 0.0 if x < 0.0 else 1.0 if x > 1.0 else x


# --- 面型 ---


def gen_framed_fill() -> None:
    """粗框 + 深色碎石填芯（material_1）。"""
    size = TEX_SIZE
    frame_rgb = (52, 54, 58)
    height = [[0.5] * size for _ in range(size)]
    albedo = [0] * (size * size * 4)

    out_c = 0.028
    lip = FRAME_T
    half_w = GROOVE_HALF

    for y in range(size):
        for x in range(size):
            u = (x + 0.5) / size
            v = (y + 0.5) / size
            d = edge_dist(u, v)
            g = groove_profile(d, out_c, half_w * 0.9)
            g = max(g, groove_profile(d, lip, half_w))
            # d 小=靠边（框），d 大=靠中（填芯）
            fill = d >= lip
            rock = fbm(u + 0.17, v + 0.31) if fill else 0.0
            height[y][x] = (
                0.5 - GROOVE_DEPTH * g - (0.14 + 0.12 * rock if fill else 0.0)
            )

            if fill:
                # 煤/碎石：深色块状
                lump = fbm(u * 1.4, v * 1.4)
                speck = hash2(x // 3, y // 3)
                shade = 14 + int(lump * 26) + int(speck * 14)
                rgb = (shade, shade + 1, shade + 3)
                ao = 0.70 + 0.18 * lump
                rgb = apply_ao(rgb, ao)
            else:
                rgb = brushed_rgb(u, v, frame_rgb, 0.04)
                # 槽底略暗
                ao = 1.0 - 0.35 * g
                rgb = apply_ao(rgb, ao)

            i = (y * size + x) * 4
            albedo[i : i + 3] = list(rgb)
            albedo[i + 3] = 255

    write_pack("material_1", albedo, height_to_normal(height, size, NORMAL_STRENGTH))


def gen_single_groove() -> None:
    """单回字凹槽（material_2）。"""
    size = TEX_SIZE
    base = (78, 82, 88)
    height = [[0.5] * size for _ in range(size)]
    albedo = [0] * (size * size * 4)
    center = 0.11  # 凹槽到边的距离

    for y in range(size):
        for x in range(size):
            u = (x + 0.5) / size
            v = (y + 0.5) / size
            d = edge_dist(u, v)
            g = groove_profile(d, center, GROOVE_HALF)
            height[y][x] = 0.5 - GROOVE_DEPTH * g

            rgb = brushed_rgb(u, v, base, 0.05)
            ao = 1.0 - 0.42 * g
            rgb = apply_ao(rgb, ao)
            i = (y * size + x) * 4
            albedo[i : i + 3] = list(rgb)
            albedo[i + 3] = 255

    write_pack("material_2", albedo, height_to_normal(height, size, NORMAL_STRENGTH))


def gen_nested_groove() -> None:
    """双回字嵌套凹槽（material_3）。"""
    size = TEX_SIZE
    base = (70, 72, 76)
    height = [[0.5] * size for _ in range(size)]
    albedo = [0] * (size * size * 4)
    c0, c1 = 0.08, 0.22

    for y in range(size):
        for x in range(size):
            u = (x + 0.5) / size
            v = (y + 0.5) / size
            d = edge_dist(u, v)
            g = max(
                groove_profile(d, c0, GROOVE_HALF * 0.95),
                groove_profile(d, c1, GROOVE_HALF * 0.95),
            )
            height[y][x] = 0.5 - GROOVE_DEPTH * g

            # 极轻石纹
            grain = (fbm(u, v, 3) - 0.5) * 10
            rgb = brushed_rgb(u, v, base, 0.035)
            rgb = tuple(max(0, min(255, int(c + grain))) for c in rgb)  # type: ignore[assignment]
            ao = 1.0 - 0.42 * g
            rgb = apply_ao(rgb, ao)
            i = (y * size + x) * 4
            albedo[i : i + 3] = list(rgb)
            albedo[i + 3] = 255

    write_pack("material_3", albedo, height_to_normal(height, size, NORMAL_STRENGTH))


def gen_basic_panel() -> None:
    """米色工业面板：外框 + 角螺孔 + 双暗槽 + 底小方块（material_4）。"""
    size = TEX_SIZE
    body = (183, 172, 142)  # #B7AC8E
    slot = (40, 44, 52)  # 深灰蓝槽
    height = [[0.5] * size for _ in range(size)]
    albedo = [0] * (size * size * 4)

    # 布局（v 增大 = 图像下方 = 面板底部）
    recess = 0.118
    bevel = 0.012
    screw_r = 0.038
    screw_inset = 0.148
    slot_x0, slot_x1 = 0.18, 0.82
    slot0 = (0.20, 0.34)  # 上槽 v0..v1
    slot1 = (0.40, 0.54)  # 下槽
    latch = (0.42, 0.68, 0.58, 0.80)  # x0,y0,x1,y1 底小方

    screws = [
        (screw_inset, screw_inset),
        (1.0 - screw_inset, screw_inset),
        (screw_inset, 1.0 - screw_inset),
        (1.0 - screw_inset, 1.0 - screw_inset),
    ]

    for y in range(size):
        for x in range(size):
            u = (x + 0.5) / size
            v = (y + 0.5) / size

            # 高度：外框 0.5，内凹约 -0.16，暗槽再 -0.22，螺孔 -0.20，闩 +0.06
            h = 0.5
            d_edge = edge_dist(u, v)
            # 外沿微倒角
            h -= 0.10 * soft_step(d_edge - 0.012, bevel)

            in_recess = soft_step(
                sd_box(u, v, recess, recess, 1.0 - recess, 1.0 - recess), bevel
            )
            h -= 0.16 * in_recess

            in_slot0 = soft_step(
                sd_box(u, v, slot_x0, slot0[0], slot_x1, slot0[1]), bevel * 0.8
            )
            in_slot1 = soft_step(
                sd_box(u, v, slot_x0, slot1[0], slot_x1, slot1[1]), bevel * 0.8
            )
            slot_m = max(in_slot0, in_slot1)
            h -= 0.22 * slot_m

            screw_m = 0.0
            for cx, cy in screws:
                screw_m = max(
                    screw_m, soft_step(sd_circle(u, v, cx, cy, screw_r), bevel)
                )
            h -= 0.18 * screw_m

            latch_m = soft_step(sd_box(u, v, *latch), bevel)
            # 只在内凹区抬起闩，避免抬到外框
            h += 0.07 * latch_m * in_recess

            height[y][x] = h

            # 颜色
            grain = (value_noise(u * 4.0, v * 4.0, 1.0) - 0.5) * 8.0
            rgb = tuple(max(0, min(255, int(c + grain))) for c in body)
            if slot_m > 0.35:
                # 槽内深色，边缘略混
                t = clamp01((slot_m - 0.35) / 0.65)
                rgb = tuple(int(rgb[i] * (1.0 - t) + slot[i] * t) for i in range(3))
            # 螺孔略暗（仍保持米色，不是黑点）
            if screw_m > 0.15:
                rgb = apply_ao(rgb, 1.0 - 0.28 * screw_m)
            # 内凹 AO + 槽更深
            ao = 1.0 - 0.18 * in_recess - 0.28 * slot_m
            # 外框与内凹交界再压一点
            lip = (
                soft_step(abs(d_edge - recess) - 0.004, 0.012)
                if d_edge < recess + 0.03
                else 0.0
            )
            ao -= 0.12 * lip
            rgb = apply_ao(rgb, clamp01(ao))
            # 闩略亮
            if latch_m > 0.4 and in_recess > 0.5:
                rgb = tuple(min(255, int(c * 1.06)) for c in rgb)

            i = (y * size + x) * 4
            albedo[i : i + 3] = list(rgb)
            albedo[i + 3] = 255

    write_pack("material_4", albedo, height_to_normal(height, size, NORMAL_STRENGTH))


def main() -> None:
    print("generating material_1 (framed fill)…", file=sys.stderr)
    gen_framed_fill()
    print("generating material_2 (single groove)…", file=sys.stderr)
    gen_single_groove()
    print("generating material_3 (nested groove)…", file=sys.stderr)
    gen_nested_groove()
    print("generating material_4 (panel)…", file=sys.stderr)
    gen_basic_panel()


if __name__ == "__main__":
    main()
