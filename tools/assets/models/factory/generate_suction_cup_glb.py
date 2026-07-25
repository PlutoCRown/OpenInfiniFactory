"""用 Blender 生成吸盘 (SuctionCup) 外观 GLB。

Blender Z-up；export_yup 后工作面 → 游戏局部 -Z（Blender +Y）：
  - 开口四棱锥 + 唇圈 + 橙色吸垫
  - 不烘焙供电口/电线（连通时由运行时工厂电线臂生成）

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/factory/generate_suction_cup_glb.py
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
    export_factory_glb,
    join_by_material,
    link,
    make_mat,
    mesh_cube,
)

import bpy
import bmesh
from mathutils import Matrix, Vector

OUT_DIR = REPO_ROOT / "assets" / "factory_blocks" / "suction_cup"
OUT_GLB = OUT_DIR / "model.glb"


def mesh_cylinder_y(
    name: str, radius: float, depth: float, loc: Vector, *, verts: int = 24
) -> bpy.types.Object:
    """圆柱沿 Blender +Y（前进方向）。"""
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cone(
        bm,
        cap_ends=True,
        cap_tris=False,
        segments=verts,
        radius1=radius,
        radius2=radius,
        depth=depth,
    )
    bmesh.ops.rotate(
        bm,
        cent=(0, 0, 0),
        matrix=Matrix.Rotation(math.radians(90), 3, "X"),
        verts=bm.verts,
    )
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    link(obj)
    obj.location = loc
    return obj


def mesh_from_faces(
    name: str, positions: list[Vector], faces: list[list[int]]
) -> bpy.types.Object:
    """按顶点与面索引建网格。"""
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    verts = [bm.verts.new(p) for p in positions]
    for face in faces:
        bm.faces.new([verts[i] for i in face])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    return link(bpy.data.objects.new(name, mesh))


def build_suction_cup() -> None:
    """开口四棱锥吸腔；不画机身、不画电线。"""
    mat_cup = make_mat("Cup", (0.78, 0.80, 0.78, 1.0), metallic=0.05, roughness=0.62)
    mat_lip = make_mat("Lip", (0.52, 0.55, 0.53, 1.0), metallic=0.18, roughness=0.50)
    mat_pad = make_mat("Pad", (0.92, 0.40, 0.06, 1.0), metallic=0.05, roughness=0.48)

    # 底口朝 +Y，顶点在格心
    base = [
        Vector((-0.46, 0.48, -0.46)),
        Vector((0.46, 0.48, -0.46)),
        Vector((0.46, 0.48, 0.46)),
        Vector((-0.46, 0.48, 0.46)),
    ]
    apex = Vector((0.0, 0.0, 0.0))
    cup = mesh_from_faces(
        "Cup",
        list(base) + [apex],
        [[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
    )
    apply_mat(cup, mat_cup)
    apply_transforms(cup)

    # 唇圈：贴齐工作面，中间挖空
    lip = mesh_cube("Lip", Vector((0.98, 0.07, 0.98)), Vector((0.0, 0.465, 0.0)))
    apply_mat(lip, mat_lip)
    apply_transforms(lip)
    boolean_diff(
        lip,
        mesh_cube("LipCut", Vector((0.70, 0.12, 0.70)), Vector((0.0, 0.465, 0.0))),
    )
    apply_mat(lip, mat_lip)

    # 吸垫沉在开口内
    pad = mesh_cylinder_y("Pad", 0.20, 0.032, Vector((0.0, 0.44, 0.0)), verts=24)
    apply_mat(pad, mat_pad)
    apply_transforms(pad)


def main() -> None:
    clear_scene()
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    print("building suction_cup…", file=sys.stderr)
    build_suction_cup()
    join_by_material()
    export_factory_glb(OUT_GLB)
    print(f"Wrote {OUT_GLB}", file=sys.stderr)


if __name__ == "__main__":
    main()
