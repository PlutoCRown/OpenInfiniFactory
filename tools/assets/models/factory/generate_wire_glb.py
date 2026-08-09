"""用 Blender 生成电线 (Wire) 外观 GLB。

1×1×1 格心为原点；6 个形态相同的节点，仅绕原点旋转朝向六面。
节点名按 export_yup 后的游戏局部轴：PosX / NegX / PosY / NegY / PosZ / NegZ
（游戏中可按面显隐）。

另有恒定显示的 Core：三轴贯穿八角柱求交，填满拐角接缝。

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
    boolean_intersect,
    clear_scene,
    export_factory_glb,
    finish,
    join_objects,
    link,
    make_mat,
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

# 通电条：单面片宽（嵌在 V 口略内侧）；长度与凹槽同长
POWER_W = 0.028
GROOVE_W = 0.04
GROOVE_D = 0.035
GROOVE_LEN_RATIO = 0.88


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


def mesh_v_groove_cutter(
    name: str,
    out: Vector,
    tangent: Vector,
    surface_r: float,
    groove_w: float,
    groove_d: float,
    z0: float,
    z1: float,
) -> bpy.types.Object:
    """三角棱柱切割体：扁平面开口宽 = groove_w，尖朝内，布尔挖 V 槽。"""
    half = groove_w * 0.5
    tip = out * (surface_r - groove_d)
    # 开口落在扁平面上，再沿 tip→开口 射线略伸出表面，保证布尔切穿
    left_s = out * surface_r + tangent * half
    right_s = out * surface_r - tangent * half

    def extend_past(p: Vector) -> Vector:
        d = p - tip
        return tip + d * ((d.length + 0.03) / d.length)

    left = extend_past(left_s)
    right = extend_past(right_s)

    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    v = [
        bm.verts.new((tip.x, tip.y, z0)),
        bm.verts.new((left.x, left.y, z0)),
        bm.verts.new((right.x, right.y, z0)),
        bm.verts.new((tip.x, tip.y, z1)),
        bm.verts.new((left.x, left.y, z1)),
        bm.verts.new((right.x, right.y, z1)),
    ]
    bm.faces.new((v[0], v[1], v[2]))
    bm.faces.new((v[3], v[5], v[4]))
    bm.faces.new((v[0], v[3], v[4], v[1]))
    bm.faces.new((v[1], v[4], v[5], v[2]))
    bm.faces.new((v[2], v[5], v[3], v[0]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    if bm.calc_volume() < 0.0:
        bmesh.ops.reverse_faces(bm, faces=bm.faces[:])
    bm.to_mesh(mesh)
    bm.free()
    mesh.validate(clean_customdata=True)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    link(obj)
    return obj


def mesh_outward_quad(
    name: str,
    out: Vector,
    tangent: Vector,
    radius: float,
    half_w: float,
    z0: float,
    z1: float,
) -> bpy.types.Object:
    """单面片（法线朝 out），作通电灯条。"""
    a = out * radius + tangent * half_w
    b = out * radius - tangent * half_w
    # 绕序使法线朝外
    verts = [
        (a.x, a.y, z0),
        (b.x, b.y, z0),
        (b.x, b.y, z1),
        (a.x, a.y, z1),
    ]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], [(0, 1, 2, 3)])
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    link(obj)
    return obj


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
    surface_r = ARM_W * 0.5

    arm = mesh_oct_prism("Arm", oct_r, arm_h, Vector((0, 0, arm_z)))
    apply_mat(arm, mat_orange)
    apply_transforms(arm)

    groove_len = arm_h * GROOVE_LEN_RATIO
    gz0 = arm_z - groove_len * 0.5
    gz1 = arm_z + groove_len * 0.5
    # 灯条与凹槽同长
    # 灯条略沉入 V 口，避免与外皮共面闪烁
    strip_r = surface_r - GROOVE_D * 0.22

    for ang in (0.0, 90.0, 180.0, 270.0):
        rad = math.radians(ang)
        out = Vector((math.cos(rad), math.sin(rad), 0.0))
        tangent = Vector((-math.sin(rad), math.cos(rad), 0.0))
        cutter = mesh_v_groove_cutter(
            f"Groove_{ang}",
            out,
            tangent,
            surface_r,
            GROOVE_W,
            GROOVE_D,
            gz0,
            gz1,
        )
        boolean_diff(arm, cutter)

        glow = mesh_outward_quad(
            f"Power_{ang}",
            out,
            tangent,
            strip_r,
            POWER_W * 0.5,
            gz0,
            gz1,
        )
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


def build_core(mat_orange: bpy.types.Material) -> bpy.types.Object:
    """三轴贯穿八角柱求交 → 中心核（恒定显示，填拐角）。"""
    oct_r = ARM_W * 0.5 / math.cos(math.radians(22.5))
    # 沿格边贯穿：长度 1，中心在原点
    along_z = mesh_oct_prism("CoreZ", oct_r, 1.0, Vector((0, 0, 0)))
    apply_mat(along_z, mat_orange)
    apply_transforms(along_z)

    along_x = mesh_oct_prism("CoreX", oct_r, 1.0, Vector((0, 0, 0)))
    along_x.rotation_euler = Euler((0, math.radians(90), 0))
    apply_mat(along_x, mat_orange)
    apply_transforms(along_x)

    along_y = mesh_oct_prism("CoreY", oct_r, 1.0, Vector((0, 0, 0)))
    along_y.rotation_euler = Euler((math.radians(90), 0, 0))
    apply_mat(along_y, mat_orange)
    apply_transforms(along_y)

    boolean_intersect(along_z, along_x)
    boolean_intersect(along_z, along_y)
    # 略缩小，避免与臂根部共面 z-fighting（GLB 无法设绘制优先级）
    along_z.scale = Vector((0.94, 0.94, 0.94))
    apply_transforms(along_z)
    along_z.name = "Core"
    if along_z.data:
        along_z.data.name = "Mesh_Core"
    return along_z


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

    print("building Core…", file=sys.stderr)
    build_core(mat_orange)

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
