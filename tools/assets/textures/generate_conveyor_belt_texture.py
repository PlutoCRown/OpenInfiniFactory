"""生成传送带人字细槽 albedo + normal（上下无缝，可做滚动动画）。

参考图自觉旋转为人字尖朝上；底色用原橡胶黑；槽为同色明暗双线凹感。

用法：
  python tools/assets/textures/generate_conveyor_belt_texture.py
"""

from __future__ import annotations

from pathlib import Path
import math
import sys

_TOOLS = Path(__file__).resolve().parents[1]
if str(_TOOLS) not in sys.path:
    sys.path.insert(0, str(_TOOLS))
from common.paths import REPO_ROOT
from common.png_util import write_png_rgba

OUT_DIR = REPO_ROOT / "assets" / "factory_blocks" / "conveyor"
OUT_ALBEDO = OUT_DIR / "belt_chevron.png"
OUT_NORMAL = OUT_DIR / "belt_chevron_normal.png"

# 原图橡胶底色
BASE = (0.07, 0.07, 0.08)
SIZE = 512
# 每张图沿 V 铺几道人字；period=1/N 保证上下无缝
N_GROOVES = 5
# 人字臂相对水平的倾角（尖朝上）；约 32° 较钝，接近参考
ARM_DEG = 32.0
# 槽半宽（UV）；细槽
GROOVE_HALF = 0.0045
NORMAL_STRENGTH = 6.0


def _smoothstep(edge0: float, edge1: float, x: float) -> float:
    t = max(0.0, min(1.0, (x - edge0) / (edge1 - edge0)))
    return t * t * (3.0 - 2.0 * t)


def _groove_signed(u: float, v: float, k: float, period: float) -> float:
    """到最近人字槽中心线的近似垂直有符号距离（UV）。

    槽中心线：v - k*|u-0.5| = n*period（尖在 u=0.5 更高 → 箭头朝上）。
    u=0 与 u=1 处 |u-0.5| 相同，槽接到左右边缘；v 按 period 周期无缝。
    """
    phase = (v - k * abs(u - 0.5)) % period
    if phase > period * 0.5:
        phase -= period
    # ∇f = (-k*sign, 1)，|∇f|=sqrt(k²+1)
    return phase / math.sqrt(k * k + 1.0)


def write_belt_textures(
    out_dir: Path | None = None, size: int = SIZE
) -> tuple[Path, Path]:
    """写出 belt_chevron.png 与 belt_chevron_normal.png，返回路径。"""
    dest = out_dir or OUT_DIR
    albedo_path = dest / "belt_chevron.png"
    normal_path = dest / "belt_chevron_normal.png"

    period = 1.0 / N_GROOVES
    k = math.tan(math.radians(ARM_DEG))
    half = GROOVE_HALF
    aa = 1.25 / size

    base_r, base_g, base_b = BASE
    hi = (min(1.0, base_r * 1.55), min(1.0, base_g * 1.55), min(1.0, base_b * 1.55))
    lo = (base_r * 0.42, base_g * 0.42, base_b * 0.42)

    height = [[1.0] * size for _ in range(size)]
    albedo = [0] * (size * size * 4)

    for y in range(size):
        # Blender/PNG 常把 v=0 放底；生成时 y=0 为底，尖朝 +v（动画滚动仍无缝）
        v = (y + 0.5) / size
        for x in range(size):
            u = (x + 0.5) / size
            sd = _groove_signed(u, v, k, period)
            ad = abs(sd)

            # 凹槽高度：中心低、两侧回到平面
            t = 1.0 - _smoothstep(half - aa, half + aa, ad)
            h = 1.0 - 0.9 * t * t
            height[y][x] = h

            # 明暗双线：+侧亮唇、−侧暗唇（同色系），中心略压暗
            hi_w = _smoothstep(-aa, aa, sd) * (1.0 - _smoothstep(half * 0.25, half, ad))
            lo_w = (1.0 - _smoothstep(-aa, aa, sd)) * (
                1.0 - _smoothstep(half * 0.25, half, ad)
            )
            hi_w *= t
            lo_w *= t
            mid = t * (1.0 - _smoothstep(0.0, half * 0.55, ad))

            r = base_r
            g = base_g
            b = base_b
            r = r * (1.0 - hi_w) + hi[0] * hi_w
            g = g * (1.0 - hi_w) + hi[1] * hi_w
            b = b * (1.0 - hi_w) + hi[2] * hi_w
            r = r * (1.0 - lo_w) + lo[0] * lo_w
            g = g * (1.0 - lo_w) + lo[1] * lo_w
            b = b * (1.0 - lo_w) + lo[2] * lo_w
            # 槽底再略压暗
            dark = 1.0 - 0.18 * mid
            r *= dark
            g *= dark
            b *= dark

            i = (y * size + x) * 4
            albedo[i : i + 4] = [
                int(max(0, min(255, r * 255))),
                int(max(0, min(255, g * 255))),
                int(max(0, min(255, b * 255))),
                255,
            ]

    # 高度 → OpenGL 法线；%size 差分保证上下左右无缝
    normal = [0] * (size * size * 4)
    for y in range(size):
        for x in range(size):
            h_l = height[y][(x - 1) % size]
            h_r = height[y][(x + 1) % size]
            h_d = height[(y - 1) % size][x]
            h_u = height[(y + 1) % size][x]
            dx = (h_r - h_l) * NORMAL_STRENGTH
            dy = (h_u - h_d) * NORMAL_STRENGTH
            nx, ny, nz = -dx, -dy, 1.0
            inv = 1.0 / math.sqrt(nx * nx + ny * ny + nz * nz)
            nx, ny, nz = nx * inv, ny * inv, nz * inv
            i = (y * size + x) * 4
            normal[i] = int((nx * 0.5 + 0.5) * 255)
            normal[i + 1] = int((ny * 0.5 + 0.5) * 255)
            normal[i + 2] = int((nz * 0.5 + 0.5) * 255)
            normal[i + 3] = 255

    write_png_rgba(albedo_path, size, size, albedo)
    write_png_rgba(normal_path, size, size, normal)
    print(f"Wrote {albedo_path}", file=sys.stderr)
    print(f"Wrote {normal_path}", file=sys.stderr)
    return albedo_path, normal_path


if __name__ == "__main__":
    write_belt_textures()
