"""用 Blender 生成 short_grass（交叉面片）与 glass（透明边框立方体）。

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/scene/generate_short_grass_and_glass.py
图标：
  ./scripts/bake_scene_icons.sh --scene-only --only <id>
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

import json
import random

import bpy
import bmesh
from mathutils import Vector


def write_meta(
    path: Path, block_id: str, connectable: list[bool], *, collision: bool = True
) -> None:
    """写 scene_block meta.json。"""
    meta = {
        "$schema": "../../../schemas/scene_block.meta.schema.json",
        "id": block_id,
        "collision": collision,
        "connectable": connectable,
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {path}", file=sys.stderr)


def make_grass_plant_image(size: int = 16, seed: int = 7) -> bpy.types.Image:
    """MC 风草丛剪影（透明底）。"""
    rng = random.Random(seed)
    img = bpy.data.images.new("ShortGrass", width=size, height=size, alpha=True)
    pixels = [0.0] * (size * size * 4)
    cx = size // 2
    for blade in range(7):
        x0 = cx + rng.randint(-5, 5)
        top = rng.randint(1, 4)
        width = 1 if rng.random() < 0.7 else 2
        green = (
            (46 + rng.randint(0, 40)) / 255.0,
            (120 + rng.randint(0, 50)) / 255.0,
            (40 + rng.randint(0, 30)) / 255.0,
        )
        dark = (
            (30 + rng.randint(0, 20)) / 255.0,
            (80 + rng.randint(0, 30)) / 255.0,
            (25 + rng.randint(0, 20)) / 255.0,
        )
        x = float(x0)
        for y in range(size - 1, top - 1, -1):
            lean = (size - 1 - y) * rng.uniform(-0.08, 0.08) + (blade - 3) * 0.05
            x += lean
            ix = int(round(x))
            col = green if (y + blade) % 3 else dark
            for dx in range(width):
                xx = ix + dx
                if 0 <= xx < size:
                    # bpy 行从下到上；逻辑 y=0 在顶 → 存到图像上方
                    iy = size - 1 - y
                    i = (iy * size + xx) * 4
                    pixels[i : i + 4] = [col[0], col[1], col[2], 1.0]
            if y <= top + 2:
                width = 1
    img.pixels = pixels
    img.pack()
    return img


def make_glass_image(size: int = 16, border: int = 2) -> bpy.types.Image:
    """浅蓝边框、中心透明。"""
    img = bpy.data.images.new("Glass", width=size, height=size, alpha=True)
    pixels = [0.0] * (size * size * 4)
    edge = (160 / 255, 210 / 255, 230 / 255, 140 / 255)
    corner = (190 / 255, 230 / 255, 245 / 255, 180 / 255)
    inset = (200 / 255, 235 / 255, 245 / 255, 70 / 255)
    for y in range(size):
        for x in range(size):
            on_edge = (
                x < border or y < border or x >= size - border or y >= size - border
            )
            if not on_edge:
                continue
            near_corner = (
                (x < border and y < border)
                or (x < border and y >= size - border)
                or (x >= size - border and y < border)
                or (x >= size - border and y >= size - border)
            )
            iy = size - 1 - y
            i = (iy * size + x) * 4
            c = corner if near_corner else edge
            pixels[i : i + 4] = list(c)
    if border >= 1:
        for i in range(border, size - border):
            for x, y in (
                (border, i),
                (size - border - 1, i),
                (i, border),
                (i, size - border - 1),
            ):
                iy = size - 1 - y
                pi = (iy * size + x) * 4
                if pixels[pi + 3] == 0.0:
                    pixels[pi : pi + 4] = list(inset)
    img.pixels = pixels
    img.pack()
    return img


def build_short_grass(mat: bpy.types.Material) -> None:
    """两片交叉面片：XY（Blender）与 ZY。"""
    h = 0.5
    # 片1：Blender XY 平面 → 游戏 X(-Z) 面，朝 +Z 游戏？
    # 保持与旧网格等价：游戏 XY 面朝 +Z、ZY 面朝 +X
    # 游戏 (x,y,z) → Blender (x,-z,y)
    # 游戏平面 XY z=0：(-h,-h,0),(h,-h,0),(h,h,0),(-h,h,0)
    # → bl: (-h,0,-h),(h,0,-h),(h,0,h),(-h,0,h)
    quads = [
        [
            Vector((-h, 0.0, -h)),
            Vector((h, 0.0, -h)),
            Vector((h, 0.0, h)),
            Vector((-h, 0.0, h)),
        ],
        # 游戏 ZY x=0：(0,-h,h),(0,-h,-h),(0,h,-h),(0,h,h)
        # → bl: (0,-h,-h),(0,h,-h),(0,h,h),(0,-h,h)
        [
            Vector((0.0, -h, -h)),
            Vector((0.0, h, -h)),
            Vector((0.0, h, h)),
            Vector((0.0, -h, h)),
        ],
    ]
    for qi, corners in enumerate(quads):
        mesh = bpy.data.meshes.new(f"Blade{qi}")
        bm = bmesh.new()
        vs = [bm.verts.new(c) for c in corners]
        bm.faces.new(vs)
        uv_layer = bm.loops.layers.uv.new()
        uvs = [(0, 0), (1, 0), (1, 1), (0, 1)]
        for loop, uv in zip(bm.faces[0].loops, uvs):
            loop[uv_layer].uv = uv
        bm.to_mesh(mesh)
        bm.free()
        obj = bpy.data.objects.new(f"Blade{qi}", mesh)
        link(obj)
        obj.data.materials.append(mat)
        apply_transforms(obj)
        set_active(obj)
        bpy.ops.object.mode_set(mode="EDIT")
        bpy.ops.mesh.select_all(action="SELECT")
        bpy.ops.mesh.quads_convert_to_tris(quad_method="BEAUTY", ngon_method="BEAUTY")
        bpy.ops.object.mode_set(mode="OBJECT")


def build_glass_cube(mat: bpy.types.Material) -> bpy.types.Object:
    """单位玻璃立方体。"""
    mesh = bpy.data.meshes.new("Glass")
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Glass", mesh)
    link(obj)
    obj.data.materials.append(mat)
    apply_transforms(obj)
    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.cube_project(cube_size=1.0)
    bpy.ops.mesh.quads_convert_to_tris(quad_method="BEAUTY", ngon_method="BEAUTY")
    bpy.ops.object.mode_set(mode="OBJECT")
    return obj


def export_short_grass() -> None:
    clear_scene()
    img = make_grass_plant_image()
    mat = make_mat(
        "short_grass",
        (1, 1, 1, 1),
        roughness=0.95,
        texture=img,
        backface_culling=False,
        blend_method="CLIP",
        alpha_cutoff=0.1,
    )
    build_short_grass(mat)
    out = ROOT / "short_grass"
    out.mkdir(parents=True, exist_ok=True)
    export_glb(out / "model.glb")
    write_meta(out / "meta.json", "short_grass", [False] * 6, collision=False)


def export_glass() -> None:
    clear_scene()
    img = make_glass_image()
    mat = make_mat(
        "glass",
        (1, 1, 1, 1),
        roughness=0.05,
        texture=img,
        blend_method="BLEND",
    )
    build_glass_cube(mat)
    out = ROOT / "glass"
    out.mkdir(parents=True, exist_ok=True)
    export_glb(out / "model.glb")
    write_meta(out / "meta.json", "glass", [True] * 6, collision=True)


def main() -> None:
    export_short_grass()
    export_glass()


if __name__ == "__main__":
    main()
