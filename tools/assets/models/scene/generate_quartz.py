"""用 Blender 生成石英场景块：quartz / quartz_pillar / quartz_slope。

大理石贴图在 bpy 内程序生成（NEAREST）；网格用 bpy，export_yup。

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/scene/generate_quartz.py
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

import math
import random

import bpy
import bmesh
from mathutils import Vector


def make_marble_image(size: int = 32, seed: int = 42) -> bpy.types.Image:
    """32×32 浅灰大理石（噪点 + 细脉）。"""
    rng = random.Random(seed)
    img = bpy.data.images.new("QuartzMarble", width=size, height=size, alpha=False)
    pixels = [0.0] * (size * size * 4)
    for y in range(size):
        for x in range(size):
            n = (rng.random() - 0.5) * 10
            r = max(0, min(255, 236 + n)) / 255.0
            g = max(0, min(255, 234 + n * 0.9)) / 255.0
            b = max(0, min(255, 228 + n * 0.7)) / 255.0
            i = (y * size + x) * 4
            pixels[i : i + 4] = [r, g, b, 1.0]
    for _ in range(5):
        x = rng.uniform(0, size)
        y = rng.uniform(0, size)
        angle = rng.uniform(-0.6, 0.6) + (math.pi * 0.15 if rng.random() < 0.5 else -0.2)
        length = rng.uniform(size * 0.6, size * 1.4)
        thickness = rng.choice([1, 1, 2])
        shade = rng.randint(185, 210) / 255.0
        steps = int(length * 3)
        for s in range(steps):
            t = s / max(1, steps - 1)
            cx = x + math.cos(angle) * length * t + math.sin(t * 6) * 0.8
            cy = y + math.sin(angle) * length * t + math.cos(t * 5) * 0.6
            for dy in range(-thickness, thickness + 1):
                for dx in range(-thickness, thickness + 1):
                    if dx * dx + dy * dy > thickness * thickness:
                        continue
                    ix, iy = int(cx + dx), int(cy + dy)
                    if 0 <= ix < size and 0 <= iy < size:
                        i = (iy * size + ix) * 4
                        f = 0.35 if (dx == 0 and dy == 0) else 0.18
                        pixels[i] = pixels[i] * (1 - f) + shade * f
                        pixels[i + 1] = pixels[i + 1] * (1 - f) + (shade - 2 / 255) * f
                        pixels[i + 2] = pixels[i + 2] * (1 - f) + (shade - 4 / 255) * f
    for _ in range(size * 2):
        x, y = rng.randrange(size), rng.randrange(size)
        i = (y * size + x) * 4
        pixels[i] = min(1.0, pixels[i] + 12 / 255)
        pixels[i + 1] = min(1.0, pixels[i + 1] + 12 / 255)
        pixels[i + 2] = min(1.0, pixels[i + 2] + 10 / 255)
    img.pixels = pixels
    img.pack()
    return img


def build_cube(mat: bpy.types.Material) -> bpy.types.Object:
    """1×1×1 立方体。"""
    mesh = bpy.data.meshes.new("Quartz")
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Quartz", mesh)
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


def build_slope(mat: bpy.types.Material) -> bpy.types.Object:
    """斜坡：高边在 Blender +Y（游戏 -Z），低边在 -Y（游戏 +Z）。"""
    h = 0.5
    # 游戏→Blender：bl=(gx, -gz, gy)
    verts = [
        Vector((-h, h, -h)),   # a
        Vector((h, h, -h)),    # b
        Vector((h, -h, -h)),   # c
        Vector((-h, -h, -h)),  # d
        Vector((-h, h, h)),    # e
        Vector((h, h, h)),     # f
    ]
    faces = [
        [0, 1, 2, 3],  # bottom
        [1, 0, 4, 5],  # back (high)
        [1, 5, 2],     # +X tri
        [0, 3, 4],     # -X tri
        [4, 3, 2, 5],  # slope
    ]
    mesh = bpy.data.meshes.new("QuartzSlope")
    mesh.from_pydata([tuple(v) for v in verts], [], faces)
    mesh.update()
    obj = bpy.data.objects.new("QuartzSlope", mesh)
    link(obj)
    obj.data.materials.append(mat)
    apply_transforms(obj)
    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.cube_project(cube_size=1.0)
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.mesh.quads_convert_to_tris(quad_method="BEAUTY", ngon_method="BEAUTY")
    bpy.ops.object.mode_set(mode="OBJECT")
    return obj


def build_pillar(
    mat: bpy.types.Material, half: float = 0.42, chamfer: float = 0.08
) -> bpy.types.Object:
    """八角柱（平面朝向轴，倒角环）。"""
    w, c = half, chamfer
    ring_xz = [
        (-w + c, w),
        (w - c, w),
        (w, w - c),
        (w, -w + c),
        (w - c, -w),
        (-w + c, -w),
        (-w, -w + c),
        (-w, w - c),
    ]
    # 游戏环在 XZ、柱沿 Y → Blender 环在 X(-Y)、柱沿 Z
    # game (x,y,z) → bl (x, -z, y)：环 game (x,z) → bl (x, -z)，y=±0.5 → z=±0.5
    z0, z1 = -0.5, 0.5
    verts: list[Vector] = []
    for x, gz in ring_xz:
        verts.append(Vector((x, -gz, z0)))
    for x, gz in ring_xz:
        verts.append(Vector((x, -gz, z1)))
    faces = []
    for i in range(8):
        a, b = i, (i + 1) % 8
        faces.append([a, b, b + 8, a + 8])
    # 顶 / 底扇形
    verts.append(Vector((0.0, 0.0, z1)))
    top_c = len(verts) - 1
    for i in range(8):
        faces.append([top_c, 8 + i, 8 + (i + 1) % 8])
    verts.append(Vector((0.0, 0.0, z0)))
    bot_c = len(verts) - 1
    for i in range(8):
        faces.append([bot_c, (i + 1) % 8, i])

    mesh = bpy.data.meshes.new("QuartzPillar")
    mesh.from_pydata([tuple(v) for v in verts], [], faces)
    mesh.update()
    obj = bpy.data.objects.new("QuartzPillar", mesh)
    link(obj)
    obj.data.materials.append(mat)
    apply_transforms(obj)
    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.cylinder_project(direction="VIEW", align="POLAR_ZX")
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.mesh.quads_convert_to_tris(quad_method="BEAUTY", ngon_method="BEAUTY")
    bpy.ops.object.mode_set(mode="OBJECT")
    return obj


def export_named(name: str, builder) -> None:
    """清空、建材质网格、导出到 scene_blocks/<name>/model.glb。"""
    clear_scene()
    img = make_marble_image()
    mat = make_mat(name, (1, 1, 1, 1), roughness=0.45, texture=img)
    builder(mat)
    out = ROOT / name / "model.glb"
    out.parent.mkdir(parents=True, exist_ok=True)
    export_glb(out)


def main() -> None:
    export_named("quartz", build_cube)
    export_named("quartz_slope", build_slope)
    export_named("quartz_pillar", build_pillar)


if __name__ == "__main__":
    main()
