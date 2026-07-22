"""用 Blender 生成电线 (Wire) 外观 GLB。

1×1×1 格心为原点；6 个形态相同的节点，仅绕原点旋转朝向六面。
节点名按 export_yup 后的游戏局部轴：PosX / NegX / PosY / NegY / PosZ / NegZ
（游戏中可按面显隐）。

端面圆形接口嵌在橙臂端面上（样式对齐传感器供电口，但无大方板、不粗于线身）。
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
from common.bpy_util import apply_mat, apply_transforms, boolean_diff, clear_scene, export_glb, finish, join_objects, link, make_mat, mesh_cube, mesh_cylinder, mesh_torus, set_active

import math

import bpy
import bmesh
from mathutils import Euler, Matrix, Vector

OUT_DIR = REPO_ROOT / "assets" / "factory_blocks" / "wire"
OUT_GLB = OUT_DIR / "model.glb"

CELL = 0.5

# 端面圆形接口：略外凸，游戏里才能看见金属环（过薄会像橙色平面）
PORT_HOLE_R, PORT_HOLE_D = 0.085, 0.045
PORT_GOLD_MAJOR, PORT_GOLD_MINOR = 0.105, 0.018
PORT_OUTER_MAJOR, PORT_OUTER_MINOR = 0.122, 0.014
PORT_SCREW_R, PORT_SCREW_D = 0.011, 0.014
PORT_SCREW_OFF = 0.088

# 橙臂截面 ≈0.30；臂身从格心贴到格面，两根对接成一根棍
ARM_W = 0.30
ARM_Z0 = 0.0  # 接到格心，避免对向臂中间留缝
ARM_Z1 = CELL  # 0.50 贴齐面，无外凸

# 通电指示条：嵌在四侧纵槽里
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


def build_arm_along_pos_z(
    mat_orange: bpy.types.Material,
    mat_metal: bpy.types.Material,
    mat_gold: bpy.types.Material,
    mat_dark: bpy.types.Material,
    mat_power: bpy.types.Material,
) -> tuple[list[bpy.types.Object], list[bpy.types.Object]]:
    """沿 Blender +Z 的单臂：橙身贴齐格面；返回 (外观零件, 通电凹槽条)。"""
    parts: list[bpy.types.Object] = []
    power_parts: list[bpy.types.Object] = []
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

        # 通电白条：坐在槽底，默认由游戏按通电显隐
        px = math.cos(rad) * (ARM_W * 0.5 - groove_d * 0.55)
        py = math.sin(rad) * (ARM_W * 0.5 - groove_d * 0.55)
        glow = mesh_cube(
            f"Power_{ang}",
            Vector((POWER_D, POWER_W, arm_h * POWER_LEN_RATIO)),
            Vector((px, py, arm_z)),
        )
        glow.rotation_euler = Euler((0, 0, rad))
        power_parts.append(finish(glow, mat_power))

    # 端面浅凹，给接口留出嵌面
    boolean_diff(
        arm,
        mesh_cylinder(
            "FaceRecess",
            PORT_OUTER_MAJOR + 0.010,
            0.05,
            Vector((0, 0, ARM_Z1 - 0.005)),
        ),
    )
    parts.append(arm)

    # 端面接口：略探出橙面，避免游戏里只剩橙色平面
    z_face = ARM_Z1 - 0.008
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
                Vector((0, 0, z_face + 0.012)),
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
                Vector((0, 0, z_face + 0.010)),
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
                    Vector((sx, sy, z_face + 0.014)),
                    verts=10,
                ),
                mat_dark,
            )
        )
    return parts, power_parts


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
    mat_power = make_mat("Power", (1.0, 1.0, 1.0, 1.0), metallic=0.0, roughness=0.35)
    # 自发光：Blender Principled Emission
    nt = mat_power.node_tree
    bsdf = next(n for n in nt.nodes if n.type == "BSDF_PRINCIPLED")
    if "Emission Color" in bsdf.inputs:
        bsdf.inputs["Emission Color"].default_value = (1.0, 1.0, 1.0, 1.0)
        bsdf.inputs["Emission Strength"].default_value = 4.0
    elif "Emission" in bsdf.inputs:
        bsdf.inputs["Emission"].default_value = (1.0, 1.0, 1.0, 1.0)

    for name, rot in FACE_ORIENTATIONS:
        print(f"building {name}…", file=sys.stderr)
        parts, power_parts = build_arm_along_pos_z(
            mat_orange, mat_metal, mat_gold, mat_dark, mat_power
        )
        rotate_objects(parts, rot)
        rotate_objects(power_parts, rot)
        join_objects(name, parts)
        if power_parts:
            join_objects(f"{name}_Power", power_parts)

    export_glb(OUT_GLB)
    print(f"Wrote {OUT_GLB}", file=sys.stderr)


if __name__ == "__main__":
    main()
