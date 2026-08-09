"""用 Blender 生成电线 (Wire) 外观 GLB。

1×1×1 格心为原点；6 个形态相同的节点，仅绕原点旋转朝向六面。
节点名按 export_yup 后的游戏局部轴：PosX / NegX / PosY / NegY / PosZ / NegZ
（游戏中可按面显隐）。

端面圆形接口：直接把供电口贴图 UV 到八角柱 +Z 端面（不多加圆盘）。
两根电线对接时橙臂贴齐成一根棍。

模板臂沿 Blender +Z 建模；export_yup：Blender +Z → 游戏 +Y。

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/factory/generate_wire_glb.py
"""

from __future__ import annotations

from pathlib import Path
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
    finish,
    join_objects,
    link,
    make_mat,
    mesh_cube,
    set_active,
)
from common.power_port_geom import WIRE_PORT, make_power_port_material

import math

import bpy
import bmesh
from mathutils import Euler, Matrix, Vector

OUT_DIR = REPO_ROOT / "assets" / "factory_blocks" / "wire"
OUT_GLB = OUT_DIR / "model.glb"

CELL = 0.5

ARM_W = 0.30
ARM_Z0 = 0.0
ARM_Z1 = CELL

POWER_W = 0.028
POWER_D = 0.018
POWER_LEN_RATIO = 0.72


def mesh_oct_prism(
    name: str, radius: float, depth: float, loc: Vector
) -> bpy.types.Object:
    """八角柱，平面朝向 XY 轴；柱沿 +Z。"""
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cone(
        bm,
        cap_ends=True,
        cap_tris=False,
        segments=8,
        radius1=radius,
        radius2=radius,
        depth=depth,
    )
    bmesh.ops.rotate(
        bm,
        cent=(0, 0, 0),
        matrix=Matrix.Rotation(math.radians(22.5), 3, "Z"),
        verts=bm.verts,
    )
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    link(obj)
    obj.location = loc
    return obj


def paint_power_port_on_arm_end(
    arm: bpy.types.Object,
    mat_orange: bpy.types.Material,
    mat_port: bpy.types.Material,
) -> None:
    """八角柱 +Z 端面：第二材质槽 + XY→UV（与烘焙 face_size=ARM_W 对齐）。"""
    mesh = arm.data
    mesh.materials.clear()
    mesh.materials.append(mat_orange)
    mesh.materials.append(mat_port)

    set_active(arm)
    bpy.ops.object.mode_set(mode="EDIT")
    bm = bmesh.from_edit_mesh(mesh)
    uv_layer = bm.loops.layers.uv.verify()
    # 烘焙平面边长 = WIRE_PORT.face_size（与 ARM_W 相同）
    span = WIRE_PORT.face_size
    for face in bm.faces:
        n = face.normal
        c = face.calc_center_median()
        if n.z > 0.85 and c.z > ARM_Z1 - 0.08:
            face.material_index = 1
            for loop in face.loops:
                p = loop.vert.co
                loop[uv_layer].uv = (p.x / span + 0.5, p.y / span + 0.5)
        else:
            face.material_index = 0
            for loop in face.loops:
                loop[uv_layer].uv = (0.0, 0.0)
    bmesh.update_edit_mesh(mesh)
    bpy.ops.object.mode_set(mode="OBJECT")


def build_arm_along_pos_z(
    mat_orange: bpy.types.Material,
    mat_port: bpy.types.Material,
    mat_power: bpy.types.Material,
) -> tuple[list[bpy.types.Object], list[bpy.types.Object]]:
    """沿 Blender +Z 的单臂：橙身端面直接贴供电口；返回 (外观, 通电条)。"""
    parts: list[bpy.types.Object] = []
    power_parts: list[bpy.types.Object] = []
    arm_h = ARM_Z1 - ARM_Z0
    arm_z = (ARM_Z0 + ARM_Z1) * 0.5
    oct_r = ARM_W * 0.5 / math.cos(math.radians(22.5))

    arm = mesh_oct_prism("Arm", oct_r, arm_h, Vector((0, 0, arm_z)))
    apply_mat(arm, mat_orange)
    apply_transforms(arm)

    groove_w, groove_d = 0.04, 0.035
    for ang in (0.0, 90.0, 180.0, 270.0):
        rad = math.radians(ang)
        ox = math.cos(rad) * (ARM_W * 0.5 - groove_d * 0.35)
        oy = math.sin(rad) * (ARM_W * 0.5 - groove_d * 0.35)
        cutter = mesh_cube(
            f"Groove_{ang}",
            Vector((groove_d, groove_w, arm_h * 0.88)),
            Vector((ox, oy, arm_z)),
        )
        cutter.rotation_euler = Euler((0, 0, rad))
        apply_transforms(cutter)
        boolean_diff(arm, cutter)

        px = math.cos(rad) * (ARM_W * 0.5 - groove_d * 0.55)
        py = math.sin(rad) * (ARM_W * 0.5 - groove_d * 0.55)
        glow = mesh_cube(
            f"Power_{ang}",
            Vector((POWER_D, POWER_W, arm_h * POWER_LEN_RATIO)),
            Vector((px, py, arm_z)),
        )
        glow.rotation_euler = Euler((0, 0, rad))
        power_parts.append(finish(glow, mat_power))

    paint_power_port_on_arm_end(arm, mat_orange, mat_port)
    parts.append(arm)
    return parts, power_parts


def rotate_objects(objs: list[bpy.types.Object], rot: Euler) -> None:
    for obj in objs:
        obj.rotation_euler = rot
        apply_transforms(obj)


FACE_ORIENTATIONS: list[tuple[str, Euler]] = [
    ("PosY", Euler((0, 0, 0))),
    ("NegY", Euler((math.radians(180), 0, 0))),
    ("PosZ", Euler((math.radians(90), 0, 0))),
    ("NegZ", Euler((math.radians(-90), 0, 0))),
    ("PosX", Euler((0, math.radians(90), 0))),
    ("NegX", Euler((0, math.radians(-90), 0))),
]


def main() -> None:
    clear_scene()
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    mat_orange = make_mat(
        "Orange", (0.92, 0.40, 0.06, 1.0), metallic=0.08, roughness=0.40
    )
    mat_port = make_power_port_material("wire")
    mat_power = make_mat("Power", (1.0, 1.0, 1.0, 1.0), metallic=0.0, roughness=0.35)
    nt = mat_power.node_tree
    bsdf = next(n for n in nt.nodes if n.type == "BSDF_PRINCIPLED")
    if "Emission Color" in bsdf.inputs:
        bsdf.inputs["Emission Color"].default_value = (1.0, 1.0, 1.0, 1.0)
        bsdf.inputs["Emission Strength"].default_value = 4.0
    elif "Emission" in bsdf.inputs:
        bsdf.inputs["Emission"].default_value = (1.0, 1.0, 1.0, 1.0)

    for name, rot in FACE_ORIENTATIONS:
        print(f"building {name}…", file=sys.stderr)
        parts, power_parts = build_arm_along_pos_z(mat_orange, mat_port, mat_power)
        rotate_objects(parts, rot)
        rotate_objects(power_parts, rot)
        join_objects(name, parts)
        if power_parts:
            join_objects(f"{name}_Power", power_parts)

    export_factory_glb(OUT_GLB, export_tangents=True)
    print(f"Wrote {OUT_GLB}", file=sys.stderr)


if __name__ == "__main__":
    main()
