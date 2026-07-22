"""用 Blender 生成抬升器 (Lifter) 外观 GLB。

Blender Z-up：
  - 蓝灰八角柱底座（底边略收）
  - 顶部稍宽的橙色八角环
  - 环内下凹盘：无贴图，同心环颜色做径向渐变

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/factory/generate_lifter_glb.py
"""

from __future__ import annotations

from pathlib import Path
import sys
_TOOLS = Path(__file__).resolve().parents[2]
if str(_TOOLS) not in sys.path:
    sys.path.insert(0, str(_TOOLS))
from common.paths import REPO_ROOT
from common.bpy_util import apply_mat, apply_transforms, boolean_diff, clear_scene, export_glb, join_by_material, link, make_mat, mesh_cylinder, set_active

import math

import bpy
import bmesh
from mathutils import Matrix, Vector

OUT_DIR = REPO_ROOT / "assets" / "factory_blocks" / "lifter"
OUT_GLB = OUT_DIR / "model.glb"

CELL = 0.5
# 外接圆半径；旋转 22.5° 使平面朝向坐标轴
BASE_R = 0.455
FOOT_R = 0.40
FOOT_H = 0.06
BASE_H = 0.70
BASE_Z0 = -CELL
BASE_Z1 = BASE_Z0 + FOOT_H + BASE_H

RING_R = 0.48
RING_INNER = 0.36
RING_H = 0.16
DISK_R = 0.385
DISK_H = 0.035
DISK_GAP_Z = 0.014
OCT_VERTS = 8
OCT_ROT = math.radians(22.5)

# 盘面同心环：中心亮 → 外缘暗
DISK_BANDS = [
    (0.14, (0.88, 0.90, 0.92, 1.0)),
    (0.24, (0.78, 0.80, 0.83, 1.0)),
    (0.32, (0.68, 0.70, 0.74, 1.0)),
    (DISK_R, (0.58, 0.60, 0.64, 1.0)),
]


def mesh_oct_prism(
    name: str, radius: float, depth: float, loc: Vector
) -> bpy.types.Object:
    """八角柱，平面朝向坐标轴。"""
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cone(
        bm,
        cap_ends=True,
        cap_tris=False,
        segments=OCT_VERTS,
        radius1=radius,
        radius2=radius,
        depth=depth,
    )
    bmesh.ops.rotate(
        bm,
        cent=(0, 0, 0),
        matrix=Matrix.Rotation(OCT_ROT, 3, "Z"),
        verts=bm.verts,
    )
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    link(obj)
    obj.location = loc
    return obj


def build_base(mat: bpy.types.Material) -> None:
    foot_z = BASE_Z0 + FOOT_H * 0.5
    foot = mesh_oct_prism("Foot", FOOT_R, FOOT_H, Vector((0, 0, foot_z)))
    apply_mat(foot, mat)
    apply_transforms(foot)

    body_z = BASE_Z0 + FOOT_H + BASE_H * 0.5
    body = mesh_oct_prism("Base", BASE_R, BASE_H, Vector((0, 0, body_z)))
    apply_mat(body, mat)
    apply_transforms(body)


def build_top(mat_orange: bpy.types.Material) -> None:
    ring_z = BASE_Z1 + RING_H * 0.5
    ring = mesh_oct_prism("Ring", RING_R, RING_H, Vector((0, 0, ring_z)))
    apply_mat(ring, mat_orange)
    apply_transforms(ring)

    boolean_diff(
        ring,
        mesh_cylinder("RingCut", RING_INNER, RING_H + 0.05, Vector((0, 0, ring_z))),
    )
    apply_mat(ring, mat_orange)

    # 同心环：由外到内挖孔，颜色由深到浅，形成径向渐变
    disk_top = BASE_Z1 + RING_H - DISK_GAP_Z
    disk_z = disk_top - DISK_H * 0.5
    prev_r = 0.0
    for i, (outer_r, color) in enumerate(DISK_BANDS):
        band = mesh_cylinder(
            f"DiskBand_{i}", outer_r, DISK_H, Vector((0, 0, disk_z)), verts=48
        )
        if prev_r > 1e-4:
            boolean_diff(
                band,
                mesh_cylinder(
                    f"DiskCut_{i}",
                    prev_r,
                    DISK_H + 0.04,
                    Vector((0, 0, disk_z)),
                    verts=48,
                ),
            )
        mat = make_mat(f"Disk_{i}", color, metallic=0.08, roughness=0.50)
        apply_mat(band, mat)
        apply_transforms(band)
        prev_r = outer_r


def main() -> None:
    clear_scene()
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    # 清掉旧贴图
    old_tex = OUT_DIR / "disk_albedo.png"
    if old_tex.exists():
        old_tex.unlink()

    mat_base = make_mat("Base", (0.28, 0.38, 0.48, 1.0), metallic=0.12, roughness=0.55)
    mat_orange = make_mat(
        "Orange", (0.92, 0.40, 0.06, 1.0), metallic=0.08, roughness=0.40
    )

    print("building base…", file=sys.stderr)
    build_base(mat_base)
    print("building top…", file=sys.stderr)
    build_top(mat_orange)
    print("joining…", file=sys.stderr)
    join_by_material()
    export_glb(OUT_GLB)
    print(f"Wrote {OUT_GLB}", file=sys.stderr)


if __name__ == "__main__":
    main()
