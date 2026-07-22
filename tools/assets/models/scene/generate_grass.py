"""用 Blender 生成草方块 grass/model.glb（顶/侧/底 atlas + 锯齿侧面贴图）。

贴图在 bpy 内程序生成；立方体 UV 按 atlas 三段（顶 / 侧 / 底）。

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/scene/generate_grass.py
图标：
  ./scripts/bake_scene_icons.sh --scene-only --only grass
"""

from __future__ import annotations

from pathlib import Path
import sys

_TOOLS = Path(__file__).resolve().parents[2]
if str(_TOOLS) not in sys.path:
    sys.path.insert(0, str(_TOOLS))
from common.paths import SCENE_BLOCKS as ROOT
from common.bpy_util import (
    apply_transforms,
    clear_scene,
    export_glb,
    link,
    make_mat,
    set_active,
)

import random

import bpy
import bmesh
from mathutils import Vector

SIZE = 64
OUT = ROOT / "grass"


def _clamp(v: float, lo: float = 0.0, hi: float = 255.0) -> int:
    return max(int(lo), min(int(hi), int(v)))


def _noise2(x: int, y: int, seed: int) -> int:
    n = (x * 374761393 + y * 668265263 + seed * 982451653) & 0xFFFFFFFF
    n = (n ^ (n >> 13)) * 1274126177 & 0xFFFFFFFF
    return n & 255


def _shade(base: tuple[int, int, int], n: int, amount: float) -> tuple[int, int, int]:
    d = (n - 128) * amount / 128
    return tuple(_clamp(c + d) for c in base)


def _gen_top() -> list[tuple[int, int, int]]:
    px = []
    for y in range(SIZE):
        for x in range(SIZE):
            n = _noise2(x, y, 197)
            fleck = ((x * 7 + y * 19 + n) % 31) < 4
            base = (93, 157, 58) if not fleck else (66, 128, 45)
            px.append(_shade(base, n, 28))
    return px


def _gen_bottom() -> list[tuple[int, int, int]]:
    px = []
    for y in range(SIZE):
        for x in range(SIZE):
            n = _noise2(x, y, 211)
            fleck = ((x * 11 + y * 17 + n) % 29) < 3
            base = (134, 96, 67) if not fleck else (110, 78, 52)
            px.append(_shade(base, n, 22))
    return px


def _gen_side(
    top: list[tuple[int, int, int]], bottom: list[tuple[int, int, int]]
) -> list[tuple[int, int, int]]:
    """泥上草绿、锯齿分界（MC 风）。"""
    rng = random.Random(7)
    base = SIZE // 2 - 2
    heights = []
    h = base
    for x in range(SIZE):
        step = rng.choice([-2, -1, -1, 0, 0, 0, 1, 1, 2])
        if rng.random() < 0.12:
            step += rng.choice([-3, 3])
        h = max(SIZE // 3, min(SIZE * 2 // 3, h + step))
        heights.append(h)
    for i in range(1, SIZE - 1):
        heights[i] = int(
            round(0.2 * heights[i - 1] + 0.6 * heights[i] + 0.2 * heights[i + 1])
        )

    out: list[tuple[int, int, int]] = [(0, 0, 0)] * (SIZE * SIZE)
    for y in range(SIZE):
        for x in range(SIZE):
            n = _noise2(x, y, 40)
            if y < heights[x]:
                c = list(top[x + ((y * 2) % SIZE) * SIZE])
                if heights[x] - y <= 2:
                    c = [_clamp(c[i] * 0.85) for i in range(3)]
                rgb = _shade(tuple(c), n, 18)
            else:
                rgb = _shade(bottom[x + (y % SIZE) * SIZE], n, 14)
            if y >= heights[x] and y < heights[x] + 3 and _noise2(x, y, 91) > 210:
                rgb = _shade(top[x + ((x + y) % SIZE) * SIZE], n, 20)
            if y < heights[x] and y > heights[x] - 2 and _noise2(x, y, 55) > 230:
                rgb = _shade(bottom[x + (y % SIZE) * SIZE], n, 12)
            out[y * SIZE + x] = rgb
    return out


def make_atlas_image() -> bpy.types.Image:
    """竖排 atlas：匹配旧 UV——v[0,1/3]=顶，[1/3,2/3]=侧，[2/3,1]=底。

    Blender/glTF：v=0 在图像底，故像素缓冲底三段放顶草。
    """
    top, bottom = _gen_top(), _gen_bottom()
    side = _gen_side(top, bottom)
    h = SIZE * 3
    img = bpy.data.images.new("GrassAtlas", width=SIZE, height=h, alpha=False)
    pixels = [0.0] * (SIZE * h * 4)
    # 像素行 0 = 图像底 = UV v≈0 → 顶草带
    bands = [top, side, bottom]
    for bi, band in enumerate(bands):
        y0 = bi * SIZE
        for y in range(SIZE):
            for x in range(SIZE):
                r, g, b = band[y * SIZE + x]
                i = ((y0 + y) * SIZE + x) * 4
                pixels[i : i + 4] = [r / 255.0, g / 255.0, b / 255.0, 1.0]
    img.pixels = pixels
    img.pack()
    return img


def _face_uv(face_kind: str) -> list[tuple[float, float]]:
    """与旧手写脚本 cube_faces_uv 一致。"""
    if face_kind == "top":
        v0, v1 = 0.0, 1.0 / 3.0
    elif face_kind == "bottom":
        v0, v1 = 2.0 / 3.0, 1.0
    else:
        # 侧面：底边偏泥 (v=2/3)，顶边偏草 (v=1/3)
        v0, v1 = 2.0 / 3.0, 1.0 / 3.0
    return [(0.0, v0), (1.0, v0), (1.0, v1), (0.0, v1)]


def build_grass_cube(mat: bpy.types.Material) -> bpy.types.Object:
    """单位立方体，按法线赋 atlas UV。"""
    mesh = bpy.data.meshes.new("Grass")
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    uv_layer = bm.loops.layers.uv.new()
    for face in bm.faces:
        n = face.normal
        if n.z > 0.5:
            kind = "top"
        elif n.z < -0.5:
            kind = "bottom"
        else:
            kind = "side"
        uvs = _face_uv(kind)
        for loop, uv in zip(face.loops, uvs):
            loop[uv_layer].uv = uv
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Grass", mesh)
    link(obj)
    obj.data.materials.append(mat)
    apply_transforms(obj)
    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.mesh.quads_convert_to_tris(quad_method="BEAUTY", ngon_method="BEAUTY")
    bpy.ops.object.mode_set(mode="OBJECT")
    return obj


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    clear_scene()
    img = make_atlas_image()
    mat = make_mat("grass", (1, 1, 1, 1), roughness=0.96, texture=img)
    build_grass_cube(mat)
    export_glb(OUT / "model.glb")


if __name__ == "__main__":
    main()
