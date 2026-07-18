"""用 Blender 生成电线 (Wire) 外观 GLB。

1×1×1 格心为原点；6 个形态相同的节点，仅绕原点旋转朝向六面。
节点名按 export_yup 后的游戏局部轴：PosX / NegX / PosY / NegY / PosZ / NegZ
（游戏中可按面显隐）。

端面圆形接口嵌在橙臂端面上（样式对齐传感器供电口，但无大方板、不粗于线身）。
两根电线对接时橙臂贴齐成一根棍。

模板臂沿 Blender +Z 建模；export_yup：Blender +Z → 游戏 +Y。

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python scripts/factory_blocks/generate_wire_glb.py
"""

from __future__ import annotations

import math
import sys
from pathlib import Path

import bpy
import bmesh
from mathutils import Euler, Matrix, Vector

ROOT = Path(__file__).resolve().parents[2]
OUT_DIR = ROOT / "assets" / "factory_blocks" / "wire"
OUT_GLB = OUT_DIR / "model.glb"

CELL = 0.5

# 端面圆形接口：样式对齐 detector，但半径收进线身截面内（无大方板）
PORT_HOLE_R, PORT_HOLE_D = 0.095, 0.035
PORT_GOLD_MAJOR, PORT_GOLD_MINOR = 0.118, 0.009
PORT_OUTER_MAJOR, PORT_OUTER_MINOR = 0.130, 0.007
PORT_SCREW_R, PORT_SCREW_D = 0.011, 0.012
PORT_SCREW_OFF = 0.100

# 橙臂截面 ≈0.30；臂身贴齐格面，两根对接成一根棍
ARM_W = 0.30
ARM_Z0 = 0.06
ARM_Z1 = CELL  # 0.50 贴齐面，无外凸


def clear_scene() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    for coll in (
        bpy.data.meshes,
        bpy.data.materials,
        bpy.data.images,
        bpy.data.cameras,
        bpy.data.lights,
    ):
        for block in list(coll):
            coll.remove(block)


def link(obj: bpy.types.Object) -> bpy.types.Object:
    bpy.context.collection.objects.link(obj)
    return obj


def set_active(obj: bpy.types.Object) -> None:
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj


def apply_mat(obj: bpy.types.Object, mat: bpy.types.Material) -> None:
    obj.data.materials.clear()
    obj.data.materials.append(mat)


def apply_transforms(obj: bpy.types.Object) -> None:
    set_active(obj)
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)


def make_mat(
    name: str,
    color: tuple[float, float, float, float],
    *,
    metallic: float = 0.0,
    roughness: float = 0.55,
) -> bpy.types.Material:
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    nt = mat.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
    bsdf.inputs["Base Color"].default_value = color
    bsdf.inputs["Metallic"].default_value = metallic
    bsdf.inputs["Roughness"].default_value = roughness
    nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    return mat


def mesh_cube(name: str, size: Vector, loc: Vector) -> bpy.types.Object:
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    bmesh.ops.scale(bm, vec=size, verts=bm.verts)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    link(obj)
    obj.location = loc
    return obj


def mesh_cylinder(
    name: str,
    radius: float,
    depth: float,
    loc: Vector,
    *,
    rot: Euler | None = None,
    verts: int = 24,
    radius2: float | None = None,
) -> bpy.types.Object:
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cone(
        bm,
        cap_ends=True,
        cap_tris=False,
        segments=verts,
        radius1=radius,
        radius2=radius if radius2 is None else radius2,
        depth=depth,
    )
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    link(obj)
    obj.location = loc
    if rot is not None:
        obj.rotation_euler = rot
    return obj


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


def mesh_torus(
    name: str,
    major: float,
    minor: float,
    loc: Vector,
    *,
    rot: Euler | None = None,
) -> bpy.types.Object:
    bpy.ops.mesh.primitive_torus_add(
        major_radius=major,
        minor_radius=minor,
        major_segments=36,
        minor_segments=12,
        location=loc,
        rotation=rot or Euler((0, 0, 0)),
    )
    obj = bpy.context.active_object
    obj.name = name
    return obj


def boolean_diff(target: bpy.types.Object, cutter: bpy.types.Object) -> None:
    apply_transforms(cutter)
    set_active(target)
    mod = target.modifiers.new("Bool", "BOOLEAN")
    mod.operation = "DIFFERENCE"
    mod.solver = "EXACT"
    mod.object = cutter
    bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.data.objects.remove(cutter, do_unlink=True)


def join_objects(name: str, objs: list[bpy.types.Object]) -> bpy.types.Object:
    assert objs
    for obj in objs:
        apply_transforms(obj)
    bpy.ops.object.select_all(action="DESELECT")
    for obj in objs:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    if len(objs) > 1:
        bpy.ops.object.join()
    joined = bpy.context.active_object
    joined.name = name
    if joined.data:
        joined.data.name = f"Mesh_{name}"
    print(f"  {name}: {len(objs)} parts", file=sys.stderr)
    return joined


def finish(obj: bpy.types.Object, mat: bpy.types.Material) -> bpy.types.Object:
    apply_mat(obj, mat)
    apply_transforms(obj)
    return obj


def build_arm_along_pos_z(
    mat_orange: bpy.types.Material,
    mat_metal: bpy.types.Material,
    mat_gold: bpy.types.Material,
    mat_dark: bpy.types.Material,
) -> list[bpy.types.Object]:
    """沿 Blender +Z 的单臂：橙身贴齐格面，端面内嵌圆形接口（不粗于线身）。"""
    parts: list[bpy.types.Object] = []
    arm_h = ARM_Z1 - ARM_Z0
    arm_z = (ARM_Z0 + ARM_Z1) * 0.5
    oct_r = ARM_W * 0.5 / math.cos(math.radians(22.5))

    arm = mesh_oct_prism("Arm", oct_r, arm_h, Vector((0, 0, arm_z)))
    apply_mat(arm, mat_orange)
    apply_transforms(arm)

    # 四侧纵槽（不改变外轮廓最大宽度）
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

    # 端面浅凹，给接口留出嵌面
    boolean_diff(
        arm,
        mesh_cylinder(
            "FaceRecess",
            PORT_OUTER_MAJOR + 0.008,
            0.04,
            Vector((0, 0, ARM_Z1 - 0.01)),
        ),
    )
    parts.append(arm)

    # 端面接口：全部落在橙臂端面上，外径不超出线身
    z_face = ARM_Z1 - 0.018
    parts.append(
        finish(
            mesh_cylinder(
                "Hole",
                PORT_HOLE_R,
                PORT_HOLE_D,
                Vector((0, 0, z_face)),
            ),
            mat_dark,
        )
    )
    parts.append(
        finish(
            mesh_torus(
                "Gold",
                PORT_GOLD_MAJOR,
                PORT_GOLD_MINOR,
                Vector((0, 0, z_face + 0.008)),
            ),
            mat_gold,
        )
    )
    parts.append(
        finish(
            mesh_torus(
                "Outer",
                PORT_OUTER_MAJOR,
                PORT_OUTER_MINOR,
                Vector((0, 0, z_face + 0.006)),
            ),
            mat_metal,
        )
    )
    for i, (sx, sy) in enumerate(
        (
            (-PORT_SCREW_OFF, -PORT_SCREW_OFF),
            (PORT_SCREW_OFF, -PORT_SCREW_OFF),
            (-PORT_SCREW_OFF, PORT_SCREW_OFF),
            (PORT_SCREW_OFF, PORT_SCREW_OFF),
        )
    ):
        parts.append(
            finish(
                mesh_cylinder(
                    f"Screw_{i}",
                    PORT_SCREW_R,
                    PORT_SCREW_D,
                    Vector((sx, sy, z_face + 0.010)),
                    verts=10,
                ),
                mat_dark,
            )
        )
    return parts


def rotate_objects(objs: list[bpy.types.Object], rot: Euler) -> None:
    for obj in objs:
        obj.rotation_euler = rot
        apply_transforms(obj)


# 模板沿 Blender +Z → export 后游戏 +Y。
# 下列旋转把 +Z 转到对应 Blender 轴，节点名用游戏轴。
FACE_ORIENTATIONS: list[tuple[str, Euler]] = [
    ("PosY", Euler((0, 0, 0))),  # +Z → +Y
    ("NegY", Euler((math.radians(180), 0, 0))),  # +Z → -Z → -Y
    ("PosZ", Euler((math.radians(90), 0, 0))),  # +Z → -Y → +Z
    ("NegZ", Euler((math.radians(-90), 0, 0))),  # +Z → +Y → -Z
    ("PosX", Euler((0, math.radians(90), 0))),  # +Z → +X
    ("NegX", Euler((0, math.radians(-90), 0))),  # +Z → -X
]


def export_glb(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    for obj in bpy.context.scene.objects:
        if obj.type == "MESH":
            obj.select_set(True)
    bpy.ops.export_scene.gltf(
        filepath=str(path),
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_texcoords=True,
        export_normals=True,
        export_materials="EXPORT",
        export_yup=True,
    )


def main() -> None:
    clear_scene()
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    mat_orange = make_mat(
        "Orange", (0.92, 0.40, 0.06, 1.0), metallic=0.08, roughness=0.40
    )
    mat_metal = make_mat(
        "Metal", (0.58, 0.60, 0.62, 1.0), metallic=0.90, roughness=0.24
    )
    mat_gold = make_mat("Gold", (0.88, 0.68, 0.16, 1.0), metallic=0.95, roughness=0.20)
    mat_dark = make_mat("Dark", (0.04, 0.04, 0.05, 1.0), metallic=0.40, roughness=0.42)

    for name, rot in FACE_ORIENTATIONS:
        print(f"building {name}…", file=sys.stderr)
        parts = build_arm_along_pos_z(mat_orange, mat_metal, mat_gold, mat_dark)
        rotate_objects(parts, rot)
        join_objects(name, parts)

    export_glb(OUT_GLB)
    print(f"Wrote {OUT_GLB}", file=sys.stderr)


if __name__ == "__main__":
    main()
