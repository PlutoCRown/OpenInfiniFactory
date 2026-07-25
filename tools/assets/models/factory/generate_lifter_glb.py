"""用 Blender 生成抬升器 (Lifter) 外观 GLB。

俯视：正方形切短斜角（非正八边形）。
侧视：蓝灰方柱 → 橙色外扩斜面 + 顶环；顶面凹槽底用径向渐变贴图。

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/factory/generate_lifter_glb.py
"""

from __future__ import annotations

from pathlib import Path
import math
import sys

_TOOLS = Path(__file__).resolve().parents[2]
if str(_TOOLS) not in sys.path:
    sys.path.insert(0, str(_TOOLS))
from common.paths import REPO_ROOT
from common.bpy_util import (
    apply_mat,
    apply_transforms,
    boolean_diff,
    clear_scene,
    export_glb,
    join_by_material,
    link,
    make_mat,
    mesh_cylinder,
    set_active,
)
from common.png_util import write_png_rgba

import bpy
import bmesh
from mathutils import Vector

OUT_DIR = REPO_ROOT / "assets" / "factory_blocks" / "lifter"
OUT_GLB = OUT_DIR / "model.glb"
OUT_DISK_TEX = OUT_DIR / "disk_albedo.png"

CELL = 0.5
Z_BOT = -CELL
Z_TOP = 0.48

# 底座半宽 / 切角（相对初版 ×1.5）
BASE_HALF = 0.34
BASE_CHAMFER = 0.0825
# 外扩后顶环半宽
TOP_HALF = 0.46
TOP_CHAMFER = 0.0975
# 凹槽：壁宽约为原先 1/3（原壁≈0.16 → ≈0.053）
WALL_T = (0.46 - 0.30) / 3.0
GROOVE_HALF = TOP_HALF - WALL_T
GROOVE_CHAMFER = 0.075
GROOVE_DEPTH = 0.11

DISK_H = 0.028
DISK_R = GROOVE_HALF - 0.01  # 圆盘略小于凹槽，避免穿边
DISK_TEX_SIZE = 256

# 竖直分段：柱身 → 45° 外扩 → 顶环
SHAFT_Z1 = 0.12
FLARE_DZ = TOP_HALF - BASE_HALF
FLARE_Z1 = SHAFT_Z1 + FLARE_DZ
RING_Z1 = Z_TOP


def chamfer_square_xy(half: float, chamfer: float) -> list[tuple[float, float]]:
    """轴对齐正方形切四角：长边 + 短斜边。逆时针，从 +X 边起。"""
    s = half
    c = min(chamfer, half * 0.45)
    return [
        (s, s - c),
        (s - c, s),
        (-(s - c), s),
        (-s, s - c),
        (-s, -(s - c)),
        (-(s - c), -s),
        (s - c, -s),
        (s, -(s - c)),
    ]


def mesh_loft(
    name: str,
    poly_bot: list[tuple[float, float]],
    poly_top: list[tuple[float, float]],
    z0: float,
    z1: float,
) -> bpy.types.Object:
    """两层同顶点数多边形 loft 成柱/台。"""
    assert len(poly_bot) == len(poly_top)
    n = len(poly_bot)
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bot = [bm.verts.new((x, y, z0)) for x, y in poly_bot]
    top = [bm.verts.new((x, y, z1)) for x, y in poly_top]
    bm.faces.new(bot)
    bm.faces.new(list(reversed(top)))
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new([bot[i], bot[j], top[j], top[i]])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    link(obj)
    return obj


def mesh_chamfer_prism(
    name: str, half: float, chamfer: float, z0: float, z1: float
) -> bpy.types.Object:
    """等高切角方柱。"""
    poly = chamfer_square_xy(half, chamfer)
    return mesh_loft(name, poly, poly, z0, z1)


def write_disk_albedo(path: Path, size: int = DISK_TEX_SIZE) -> bpy.types.Image:
    """径向渐变：中心亮 → 外缘暗。"""
    cx = cy = (size - 1) * 0.5
    inv_r = 1.0 / max(cx, 1.0)
    stops = [
        (0.00, (0.88, 0.90, 0.92)),
        (0.35, (0.78, 0.80, 0.83)),
        (0.65, (0.68, 0.70, 0.74)),
        (1.00, (0.58, 0.60, 0.64)),
    ]
    rgba = [0] * (size * size * 4)
    for y in range(size):
        for x in range(size):
            t = min(1.0, math.hypot(x - cx, y - cy) * inv_r)
            r = g = b = 0.0
            for i in range(len(stops) - 1):
                t0, c0 = stops[i]
                t1, c1 = stops[i + 1]
                if t <= t1 or i == len(stops) - 2:
                    u = 0.0 if t1 <= t0 else (t - t0) / (t1 - t0)
                    u = max(0.0, min(1.0, u))
                    r = c0[0] + (c1[0] - c0[0]) * u
                    g = c0[1] + (c1[1] - c0[1]) * u
                    b = c0[2] + (c1[2] - c0[2]) * u
                    break
            i = (y * size + x) * 4
            rgba[i : i + 4] = [int(r * 255), int(g * 255), int(b * 255), 255]
    write_png_rgba(path, size, size, rgba)
    return bpy.data.images.load(str(path))


def build_base(mat: bpy.types.Material) -> None:
    """蓝灰柱身（外扩斜面归橙色）。"""
    shaft = mesh_chamfer_prism("Shaft", BASE_HALF, BASE_CHAMFER, Z_BOT, SHAFT_Z1)
    apply_mat(shaft, mat)
    apply_transforms(shaft)


def build_orange(mat_orange: bpy.types.Material, disk_img: bpy.types.Image) -> None:
    """橙色外扩斜面 + 顶环 + 凹槽底贴图盘。"""
    flare = mesh_loft(
        "Flare",
        chamfer_square_xy(BASE_HALF, BASE_CHAMFER),
        chamfer_square_xy(TOP_HALF, TOP_CHAMFER),
        SHAFT_Z1,
        FLARE_Z1,
    )
    apply_mat(flare, mat_orange)
    apply_transforms(flare)

    ring = mesh_chamfer_prism("Ring", TOP_HALF, TOP_CHAMFER, FLARE_Z1, RING_Z1)
    apply_mat(ring, mat_orange)
    apply_transforms(ring)

    cut_half = GROOVE_HALF + 0.01
    cut_chamfer = GROOVE_CHAMFER + 0.005
    cutter = mesh_chamfer_prism(
        "GrooveCut",
        cut_half,
        cut_chamfer,
        RING_Z1 - GROOVE_DEPTH,
        RING_Z1 + 0.04,
    )
    boolean_diff(ring, cutter)
    apply_mat(ring, mat_orange)

    floor_z0 = RING_Z1 - GROOVE_DEPTH
    disk_z = floor_z0 + DISK_H * 0.5
    disk = mesh_cylinder("Disk", DISK_R, DISK_H, Vector((0, 0, disk_z)), verts=32)
    mat_disk = make_mat(
        "Disk",
        (0.72, 0.74, 0.78, 1.0),
        metallic=0.08,
        roughness=0.50,
        texture=disk_img,
    )
    apply_mat(disk, mat_disk)
    apply_transforms(disk)

    # 顶面 UV：圆心映射到贴图中心，半径铺满
    set_active(disk)
    bpy.ops.object.mode_set(mode="EDIT")
    bm = bmesh.from_edit_mesh(disk.data)
    uv = bm.loops.layers.uv.verify()
    for face in bm.faces:
        for loop in face.loops:
            p = loop.vert.co
            loop[uv].uv = (p.x / (DISK_R * 2) + 0.5, p.y / (DISK_R * 2) + 0.5)
    bmesh.update_edit_mesh(disk.data)
    bpy.ops.object.mode_set(mode="OBJECT")
    apply_mat(disk, mat_disk)


def main() -> None:
    clear_scene()
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    print("writing disk albedo…", file=sys.stderr)
    disk_img = write_disk_albedo(OUT_DISK_TEX)

    mat_base = make_mat("Base", (0.28, 0.38, 0.48, 1.0), metallic=0.12, roughness=0.55)
    mat_orange = make_mat(
        "Orange", (0.92, 0.40, 0.06, 1.0), metallic=0.08, roughness=0.40
    )

    print(
        f"base half={BASE_HALF} chamfer={BASE_CHAMFER} "
        f"top half={TOP_HALF} groove half={GROOVE_HALF:.3f} wall={WALL_T:.3f}",
        file=sys.stderr,
    )
    print("building base…", file=sys.stderr)
    build_base(mat_base)
    print("building orange…", file=sys.stderr)
    build_orange(mat_orange, disk_img)
    print("joining…", file=sys.stderr)
    join_by_material()
    export_glb(OUT_GLB)
    print(f"Wrote {OUT_DISK_TEX}", file=sys.stderr)
    print(f"Wrote {OUT_GLB}", file=sys.stderr)


if __name__ == "__main__":
    main()
