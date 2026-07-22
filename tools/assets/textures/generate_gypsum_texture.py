"""生成石膏材料贴图，并把贴图打进石膏斜坡 GLB。

  gypsum        → assets/material_blocks/gypsum/texture.png（单位立方体贴图）
  gypsum_slope  → 以 quartz_slope 网格为底，替换内嵌 PNG

用法：
  python3 tools/assets/textures/generate_gypsum_texture.py

图标请烘焙：
  ./scripts/bake_scene_icons.sh --materials-only --only gypsum
  ./scripts/bake_scene_icons.sh --materials-only --only gypsum_slope
"""

from __future__ import annotations

from pathlib import Path
import sys
_TOOLS = Path(__file__).resolve().parents[1]
if str(_TOOLS) not in sys.path:
    sys.path.insert(0, str(_TOOLS))
from common.paths import REPO_ROOT
from common.glb_embed import replace_glb_texture

import hashlib
import io
import random

from PIL import Image

MAT_ROOT = REPO_ROOT / "assets" / "material_blocks"
SLOPE_SRC = REPO_ROOT / "assets" / "scene_blocks" / "quartz_slope" / "model.glb"


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
