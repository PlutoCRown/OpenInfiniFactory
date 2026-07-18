"""用 Blender 生成活塞 (Pusher) / 拦截器 (Blocker) 外观 GLB。

二者几何大体相同；拦截器：头面板橙色，活塞柄为单根粗方臂（两节）。

Blender Z-up；export_yup 后前进方向 → 游戏局部 -Z（Blender +Y）：
  - Body：沿前进轴厚度 1-HEAD_T；前橙环属本体；五面供电口
  - Stage：粗根节，平移 offset/2
  - Head：田字格面板 + 细尖节，平移 offset

一次按 offset 导出收起/伸出；Pusher 与 Blocker 各一套：
  assets/factory_blocks/pusher/{model,extended}.glb
  assets/factory_blocks/blocker/{model,extended}.glb

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python scripts/factory_blocks/generate_pusher_glb.py
"""

from __future__ import annotations

import math
import sys
from pathlib import Path

import bpy
import bmesh
from mathutils import Euler, Vector

ROOT = Path(__file__).resolve().parents[2]
OUT_PUSHER = ROOT / "assets" / "factory_blocks" / "pusher"
OUT_BLOCKER = ROOT / "assets" / "factory_blocks" / "blocker"

CELL = 0.5
# 供电口：与 generate_detector_glb.add_port 一致
PORT_PLATE = 0.40
PORT_PLATE_T = 0.03
PORT_HOLE_R, PORT_HOLE_D = 0.095, 0.035
PORT_GOLD_MAJOR, PORT_GOLD_MINOR = 0.118, 0.009
PORT_OUTER_MAJOR, PORT_OUTER_MINOR = 0.145, 0.007
PORT_SCREW_R, PORT_SCREW_D = 0.013, 0.013
PORT_SCREW_OFF = 0.145
PORT_PAD = 0.016

ORANGE_T = 0.08
HEAD_T = 0.08
# 活塞体沿前进轴厚度 = 1 - 活塞面厚度；橙环是体前脸一圈
BODY_Y0 = -CELL
BODY_Y1 = CELL - HEAD_T  # 0.42；收起时头面板占满剩余 HEAD_T
# 四臂 2×2；两节伸缩——根节粗、只移 offset/2；尖节细、随头移 offset
ARM_OFF = 0.22
ROOT_R = 0.11  # 活塞：根部粗圆节
TIP_R = 0.075  # 活塞：尖节
SEG_LEN = 0.58  # 每节长度；半伸出时仍重叠咬合
# 拦截器：四根合成一根粗方臂（几乎贴满橙框开口 0.78）
BLOCK_ROOT_W = 0.72  # 根节截面边长
BLOCK_TIP_W = 0.60  # 尖节截面边长

# (offset, 文件名)
OFFSET_EXPORTS = (
    (0.0, "model.glb"),
    (1.0, "extended.glb"),
)
# (输出目录, 头面板是否橙色)
VARIANTS = (
    (OUT_PUSHER, False),
    (OUT_BLOCKER, True),
)


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


def finish(obj: bpy.types.Object, mat: bpy.types.Material) -> bpy.types.Object:
    apply_mat(obj, mat)
    apply_transforms(obj)
    return obj


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


def add_port(
    prefix: str,
    center: Vector,
    axis: str,
    mat_metal: bpy.types.Material,
    mat_gold: bpy.types.Material,
    mat_dark: bpy.types.Material,
    parts: list[bpy.types.Object],
) -> None:
    """方块传感器同款供电口。"""
    plate = PORT_PLATE
    plate_t = PORT_PLATE_T
    hole_r, hole_d = PORT_HOLE_R, PORT_HOLE_D
    gold_major, gold_minor = PORT_GOLD_MAJOR, PORT_GOLD_MINOR
    outer_major, outer_minor = PORT_OUTER_MAJOR, PORT_OUTER_MINOR
    screw_r, screw_d = PORT_SCREW_R, PORT_SCREW_D
    screw_off = PORT_SCREW_OFF

    if axis in ("+Z", "-Z"):
        sign = 1.0 if axis == "+Z" else -1.0
        parts.append(
            finish(
                mesh_cube(f"{prefix}_Plate", Vector((plate, plate, plate_t)), center),
                mat_metal,
            )
        )
        parts.append(
            finish(
                mesh_cylinder(
                    f"{prefix}_Hole",
                    hole_r,
                    hole_d,
                    center + Vector((0, 0, sign * 0.002)),
                ),
                mat_dark,
            )
        )
        parts.append(
            finish(
                mesh_torus(
                    f"{prefix}_Gold",
                    gold_major,
                    gold_minor,
                    center + Vector((0, 0, sign * 0.012)),
                ),
                mat_gold,
            )
        )
        parts.append(
            finish(
                mesh_torus(
                    f"{prefix}_Outer",
                    outer_major,
                    outer_minor,
                    center + Vector((0, 0, sign * 0.01)),
                ),
                mat_metal,
            )
        )
        for i, (sx, sy) in enumerate(
            (
                (-screw_off, -screw_off),
                (screw_off, -screw_off),
                (-screw_off, screw_off),
                (screw_off, screw_off),
            )
        ):
            parts.append(
                finish(
                    mesh_cylinder(
                        f"{prefix}_S{i}",
                        screw_r,
                        screw_d,
                        center + Vector((sx, sy, sign * 0.014)),
                        verts=10,
                    ),
                    mat_dark,
                )
            )

    elif axis in ("+X", "-X"):
        sign = 1.0 if axis == "+X" else -1.0
        rot = Euler((0, math.radians(90), 0))
        parts.append(
            finish(
                mesh_cube(f"{prefix}_Plate", Vector((plate_t, plate, plate)), center),
                mat_metal,
            )
        )
        parts.append(
            finish(
                mesh_cylinder(
                    f"{prefix}_Hole",
                    hole_r,
                    hole_d,
                    center + Vector((sign * 0.002, 0, 0)),
                    rot=rot,
                ),
                mat_dark,
            )
        )
        parts.append(
            finish(
                mesh_torus(
                    f"{prefix}_Gold",
                    gold_major,
                    gold_minor,
                    center + Vector((sign * 0.012, 0, 0)),
                    rot=rot,
                ),
                mat_gold,
            )
        )
        parts.append(
            finish(
                mesh_torus(
                    f"{prefix}_Outer",
                    outer_major,
                    outer_minor,
                    center + Vector((sign * 0.01, 0, 0)),
                    rot=rot,
                ),
                mat_metal,
            )
        )
        for i, (sy, sz) in enumerate(
            (
                (-screw_off, -screw_off),
                (screw_off, -screw_off),
                (-screw_off, screw_off),
                (screw_off, screw_off),
            )
        ):
            parts.append(
                finish(
                    mesh_cylinder(
                        f"{prefix}_S{i}",
                        screw_r,
                        screw_d,
                        center + Vector((sign * 0.014, sy, sz)),
                        rot=rot,
                        verts=10,
                    ),
                    mat_dark,
                )
            )

    else:  # -Y 背面
        rot = Euler((math.radians(90), 0, 0))
        parts.append(
            finish(
                mesh_cube(f"{prefix}_Plate", Vector((plate, plate_t, plate)), center),
                mat_metal,
            )
        )
        parts.append(
            finish(
                mesh_cylinder(
                    f"{prefix}_Hole",
                    hole_r,
                    hole_d,
                    center + Vector((0, -0.002, 0)),
                    rot=rot,
                ),
                mat_dark,
            )
        )
        parts.append(
            finish(
                mesh_torus(
                    f"{prefix}_Gold",
                    gold_major,
                    gold_minor,
                    center + Vector((0, -0.012, 0)),
                    rot=rot,
                ),
                mat_gold,
            )
        )
        parts.append(
            finish(
                mesh_torus(
                    f"{prefix}_Outer",
                    outer_major,
                    outer_minor,
                    center + Vector((0, -0.01, 0)),
                    rot=rot,
                ),
                mat_metal,
            )
        )
        for i, (sx, sz) in enumerate(
            (
                (-screw_off, -screw_off),
                (screw_off, -screw_off),
                (-screw_off, screw_off),
                (screw_off, screw_off),
            )
        ):
            parts.append(
                finish(
                    mesh_cylinder(
                        f"{prefix}_S{i}",
                        screw_r,
                        screw_d,
                        center + Vector((sx, -0.014, sz)),
                        rot=rot,
                        verts=10,
                    ),
                    mat_dark,
                )
            )


def arm_xy() -> list[tuple[float, float]]:
    return [
        (-ARM_OFF, -ARM_OFF),
        (ARM_OFF, -ARM_OFF),
        (-ARM_OFF, ARM_OFF),
        (ARM_OFF, ARM_OFF),
    ]


def build_body(
    mat_body: bpy.types.Material,
    mat_orange: bpy.types.Material,
    mat_metal: bpy.types.Material,
    mat_gold: bpy.types.Material,
    mat_dark: bpy.types.Material,
) -> list[bpy.types.Object]:
    """活塞体：厚度 1-HEAD_T；前橙环属本体；五面供电口。"""
    parts: list[bpy.types.Object] = []
    core_y1 = BODY_Y1 - ORANGE_T
    core_h = core_y1 - BODY_Y0
    body = mesh_cube(
        "BodyCore",
        Vector((1.0, core_h, 1.0)),
        Vector((0.0, (BODY_Y0 + core_y1) * 0.5, 0.0)),
    )
    apply_mat(body, mat_body)
    apply_transforms(body)
    boolean_diff(
        body,
        mesh_cube(
            "Cavity",
            Vector((0.72, core_h - 0.10, 0.72)),
            Vector((0.0, (BODY_Y0 + core_y1) * 0.5 + 0.04, 0.0)),
        ),
    )
    parts.append(body)

    frame = mesh_cube(
        "OrangeRing",
        Vector((1.0, ORANGE_T, 1.0)),
        Vector((0.0, (core_y1 + BODY_Y1) * 0.5, 0.0)),
    )
    apply_mat(frame, mat_orange)
    apply_transforms(frame)
    boolean_diff(
        frame,
        mesh_cube(
            "FrameCut",
            Vector((0.78, ORANGE_T + 0.04, 0.78)),
            Vector((0.0, (core_y1 + BODY_Y1) * 0.5, 0.0)),
        ),
    )
    parts.append(frame)

    pad = PORT_PAD
    add_port(
        "PosZ", Vector((0, 0, CELL + pad)), "+Z", mat_metal, mat_gold, mat_dark, parts
    )
    add_port(
        "NegZ", Vector((0, 0, -CELL - pad)), "-Z", mat_metal, mat_gold, mat_dark, parts
    )
    add_port(
        "PosX", Vector((CELL + pad, 0, 0)), "+X", mat_metal, mat_gold, mat_dark, parts
    )
    add_port(
        "NegX", Vector((-CELL - pad, 0, 0)), "-X", mat_metal, mat_gold, mat_dark, parts
    )
    add_port(
        "NegY", Vector((0, -CELL - pad, 0)), "-Y", mat_metal, mat_gold, mat_dark, parts
    )
    return parts


def build_stage(
    offset: float,
    mat_body: bpy.types.Material,
    *,
    square_arm: bool,
) -> list[bpy.types.Object]:
    """粗根节：只平移 offset/2，收起藏进本体内腔。"""
    parts: list[bpy.types.Object] = []
    dy = offset * 0.5
    y1 = BODY_Y1 + dy
    y0 = y1 - SEG_LEN
    y = (y0 + y1) * 0.5
    if square_arm:
        # 拦截器：单根粗方管
        parts.append(
            finish(
                mesh_cube(
                    "Root",
                    Vector((BLOCK_ROOT_W, SEG_LEN, BLOCK_ROOT_W)),
                    Vector((0.0, y, 0.0)),
                ),
                mat_body,
            )
        )
    else:
        rot = Euler((math.radians(90), 0, 0))
        for i, (x, z) in enumerate(arm_xy()):
            parts.append(
                finish(
                    mesh_cylinder(
                        f"Root_{i}",
                        ROOT_R,
                        SEG_LEN,
                        Vector((x, y, z)),
                        rot=rot,
                        verts=22,
                    ),
                    mat_body,
                )
            )
    return parts


def build_head(
    offset: float,
    mat_head: bpy.types.Material,
    mat_dark: bpy.types.Material,
    mat_rod: bpy.types.Material,
    *,
    square_arm: bool,
) -> list[bpy.types.Object]:
    """头面板 + 尖节：整体平移 offset。"""
    parts: list[bpy.types.Object] = []
    head_y0 = BODY_Y1 + offset
    head_y1 = head_y0 + HEAD_T
    head = mesh_cube(
        "HeadPlate",
        Vector((1.0, HEAD_T, 1.0)),
        Vector((0.0, (head_y0 + head_y1) * 0.5, 0.0)),
    )
    apply_mat(head, mat_head)
    apply_transforms(head)
    groove = 0.04
    boolean_diff(
        head,
        mesh_cube(
            "CrossH",
            Vector((0.92, HEAD_T + 0.04, groove)),
            Vector((0.0, (head_y0 + head_y1) * 0.5, 0.0)),
        ),
    )
    boolean_diff(
        head,
        mesh_cube(
            "CrossV",
            Vector((groove, HEAD_T + 0.04, 0.92)),
            Vector((0.0, (head_y0 + head_y1) * 0.5, 0.0)),
        ),
    )
    parts.append(head)

    parts.append(
        finish(
            mesh_cube(
                "CrossFillH",
                Vector((0.88, 0.02, groove * 0.7)),
                Vector((0.0, head_y1 - 0.015, 0.0)),
            ),
            mat_dark,
        )
    )
    parts.append(
        finish(
            mesh_cube(
                "CrossFillV",
                Vector((groove * 0.7, 0.02, 0.88)),
                Vector((0.0, head_y1 - 0.015, 0.0)),
            ),
            mat_dark,
        )
    )

    tip_y1 = head_y0 + 0.02
    tip_y0 = tip_y1 - SEG_LEN
    tip_y = (tip_y0 + tip_y1) * 0.5
    if square_arm:
        parts.append(
            finish(
                mesh_cube(
                    "Tip",
                    Vector((BLOCK_TIP_W, SEG_LEN, BLOCK_TIP_W)),
                    Vector((0.0, tip_y, 0.0)),
                ),
                mat_rod,
            )
        )
    else:
        rot = Euler((math.radians(90), 0, 0))
        for i, (x, z) in enumerate(arm_xy()):
            parts.append(
                finish(
                    mesh_cylinder(
                        f"Tip_{i}",
                        TIP_R,
                        SEG_LEN,
                        Vector((x, tip_y, z)),
                        rot=rot,
                        verts=20,
                    ),
                    mat_rod,
                )
            )
    return parts


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


def build_and_export(
    out_dir: Path, offset: float, filename: str, *, orange_head: bool
) -> None:
    clear_scene()
    mat_body = make_mat("Body", (0.30, 0.38, 0.46, 1.0), metallic=0.18, roughness=0.52)
    mat_orange = make_mat(
        "Orange", (0.92, 0.40, 0.06, 1.0), metallic=0.08, roughness=0.40
    )
    mat_metal = make_mat(
        "Metal", (0.58, 0.60, 0.62, 1.0), metallic=0.90, roughness=0.24
    )
    mat_gold = make_mat("Gold", (0.88, 0.68, 0.16, 1.0), metallic=0.95, roughness=0.20)
    mat_dark = make_mat("Dark", (0.04, 0.04, 0.05, 1.0), metallic=0.40, roughness=0.42)
    mat_rod = make_mat("Rod", (0.78, 0.80, 0.82, 1.0), metallic=0.75, roughness=0.30)
    mat_head = mat_orange if orange_head else mat_body
    square_arm = orange_head  # 拦截器：单根粗方臂

    kind = "blocker" if orange_head else "pusher"
    print(f"building {kind} body (offset={offset})…", file=sys.stderr)
    body_parts = build_body(mat_body, mat_orange, mat_metal, mat_gold, mat_dark)
    print("building stage (offset/2)…", file=sys.stderr)
    stage_parts = build_stage(offset, mat_body, square_arm=square_arm)
    print("building head + tip…", file=sys.stderr)
    head_parts = build_head(offset, mat_head, mat_dark, mat_rod, square_arm=square_arm)

    join_objects("Body", body_parts)
    join_objects("Stage", stage_parts)
    join_objects("Head", head_parts)

    out = out_dir / filename
    export_glb(out)
    print(f"Wrote {out}", file=sys.stderr)


def main() -> None:
    for out_dir, orange_head in VARIANTS:
        out_dir.mkdir(parents=True, exist_ok=True)
        for offset, filename in OFFSET_EXPORTS:
            build_and_export(out_dir, offset, filename, orange_head=orange_head)


if __name__ == "__main__":
    main()
