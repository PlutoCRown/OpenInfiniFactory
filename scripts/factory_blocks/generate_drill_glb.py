"""用 Blender 生成钻头 (Drill) 外观 GLB。

整体 1×1×2（Blender Z-up；export_yup 后前进方向 → 游戏局部 -Z）：
  - 本体格：中心在原点，Y ∈ [-0.5, +0.5]
  - 钻头格：Y ∈ [+0.5, +1.5]

同一 model.glb 内两个独立节点（方便对 Head 做旋转动画）：
  - Body：蓝尾环 + 深灰散热槽 + 橙框
  - Head：阶梯锥 + 尖齿

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python scripts/factory_blocks/generate_drill_glb.py
"""

from __future__ import annotations

import math
import sys
from pathlib import Path

import bpy
import bmesh
from mathutils import Euler, Vector

ROOT = Path(__file__).resolve().parents[2]
OUT_DIR = ROOT / "assets" / "factory_blocks" / "drill"
OUT_GLB = OUT_DIR / "model.glb"

CELL = 0.5
# 本体：蓝尾环 + 深灰中段 + 橙前框
BLUE_T = 0.10
BODY_Y0 = -CELL
BLUE_Y1 = BODY_Y0 + BLUE_T  # -0.40
BODY_Y1 = 0.40
ORANGE_T = 0.10
ORANGE_Y0 = BODY_Y1
ORANGE_Y1 = ORANGE_Y0 + ORANGE_T  # 0.50 贴齐本体格前脸

# 钻头：从橙框中心伸出，占前进一格
HEAD_Y0 = ORANGE_Y1 - 0.02
HEAD_Y1 = CELL + 1.0  # +1.5


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
    verts: int = 32,
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


def mesh_cone_along_y(
    name: str,
    radius_back: float,
    radius_front: float,
    y0: float,
    y1: float,
    *,
    verts: int = 28,
) -> bpy.types.Object:
    """沿 +Y 的截锥：radius_back 在 y0，radius_front 在 y1。

    create_cone 沿 +Z；绕 X 转 90° 后 local -Z→+Y，故 radius1 对应前端。
    """
    depth = y1 - y0
    return mesh_cylinder(
        name,
        radius_front,
        depth,
        Vector((0.0, (y0 + y1) * 0.5, 0.0)),
        rot=Euler((math.radians(90), 0, 0)),
        verts=verts,
        radius2=radius_back,
    )


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
    """合并为一块，保留多材质槽。"""
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


def build_body(
    mat_dark: bpy.types.Material,
    mat_orange: bpy.types.Material,
    mat_blue: bpy.types.Material,
    mat_recess: bpy.types.Material,
) -> list[bpy.types.Object]:
    """蓝尾环 + 深灰散热槽机身 + 橙框。"""
    # 屁股蓝环
    blue = mesh_cube(
        "BlueRing",
        Vector((1.0, BLUE_T, 1.0)),
        Vector((0.0, (BODY_Y0 + BLUE_Y1) * 0.5, 0.0)),
    )
    apply_mat(blue, mat_blue)
    apply_transforms(blue)

    body_h = BODY_Y1 - BLUE_Y1
    body = mesh_cube(
        "BodyCore",
        Vector((1.0, body_h, 1.0)),
        Vector((0.0, (BLUE_Y1 + BODY_Y1) * 0.5, 0.0)),
    )
    apply_mat(body, mat_dark)
    apply_transforms(body)

    # 顶面与两侧水平散热槽
    groove_h = 0.045
    groove_d = 0.055
    n_grooves = 5
    y_span = BODY_Y1 - BLUE_Y1 - 0.12
    for i in range(n_grooves):
        t = (i + 0.5) / n_grooves
        y = BLUE_Y1 + 0.06 + t * y_span
        boolean_diff(
            body,
            mesh_cube(
                f"GrooveTop_{i}",
                Vector((1.06, groove_h, groove_d)),
                Vector((0.0, y, CELL - groove_d * 0.35)),
            ),
        )
        boolean_diff(
            body,
            mesh_cube(
                f"GrooveBot_{i}",
                Vector((1.06, groove_h, groove_d)),
                Vector((0.0, y, -CELL + groove_d * 0.35)),
            ),
        )
        boolean_diff(
            body,
            mesh_cube(
                f"GroovePosX_{i}",
                Vector((groove_d, groove_h, 1.06)),
                Vector((CELL - groove_d * 0.35, y, 0.0)),
            ),
        )
        boolean_diff(
            body,
            mesh_cube(
                f"GrooveNegX_{i}",
                Vector((groove_d, groove_h, 1.06)),
                Vector((-CELL + groove_d * 0.35, y, 0.0)),
            ),
        )

    frame = mesh_cube(
        "OrangeFrame",
        Vector((1.0, ORANGE_T, 1.0)),
        Vector((0.0, (ORANGE_Y0 + ORANGE_Y1) * 0.5, 0.0)),
    )
    apply_mat(frame, mat_orange)
    apply_transforms(frame)
    boolean_diff(
        frame,
        mesh_cone_along_y(
            "FrameHole", 0.22, 0.22, ORANGE_Y0 - 0.02, ORANGE_Y1 + 0.02, verts=24
        ),
    )

    recess = mesh_cone_along_y(
        "Recess", 0.20, 0.20, ORANGE_Y0 - 0.01, ORANGE_Y1 - 0.02, verts=24
    )
    apply_mat(recess, mat_recess)
    apply_transforms(recess)
    return [blue, body, frame, recess]


def add_tooth(
    name: str,
    y: float,
    radius: float,
    angle: float,
    *,
    length: float = 0.07,
    base: float = 0.04,
    mat: bpy.types.Material,
) -> bpy.types.Object:
    """径向尖齿：根在圆柱面，尖朝外。"""
    cx = math.cos(angle) * radius
    cz = math.sin(angle) * radius
    tooth = mesh_cylinder(
        name,
        base * 0.5,
        length,
        Vector((0, 0, 0)),
        verts=6,
        radius2=0.002,
    )
    apply_mat(tooth, mat)
    outward = Vector((cx, 0.0, cz)).normalized()
    tooth.rotation_euler = outward.to_track_quat("Z", "Y").to_euler()
    tooth.location = Vector((cx, y, cz)) + outward * (length * 0.45)
    apply_transforms(tooth)
    return tooth


def build_head(
    mat_metal: bpy.types.Material, mat_tooth: bpy.types.Material
) -> list[bpy.types.Object]:
    """阶梯锥钻头 + 各层尖齿（整体绕 Y 可转）。"""
    parts: list[bpy.types.Object] = []
    tiers = (
        (HEAD_Y0, HEAD_Y0 + 0.32, 0.40, 0.34, 12),
        (HEAD_Y0 + 0.30, HEAD_Y0 + 0.58, 0.32, 0.24, 10),
        (HEAD_Y0 + 0.56, HEAD_Y0 + 0.82, 0.22, 0.14, 8),
    )
    for i, (y0, y1, r0, r1, n_teeth) in enumerate(tiers):
        cone = mesh_cone_along_y(f"Tier_{i}", r0, r1, y0, y1, verts=28)
        apply_mat(cone, mat_metal)
        apply_transforms(cone)
        parts.append(cone)
        y_ring = y0 + (y1 - y0) * 0.55
        r_ring = r0 + (r1 - r0) * 0.55
        for k in range(n_teeth):
            ang = (k / n_teeth) * math.tau + (0.15 if i % 2 else 0.0)
            parts.append(
                add_tooth(
                    f"Tooth_{i}_{k}",
                    y_ring,
                    r_ring,
                    ang,
                    length=0.075 if i == 0 else 0.065,
                    base=0.045 if i == 0 else 0.038,
                    mat=mat_tooth,
                )
            )

    tip = mesh_cone_along_y("Tip", 0.12, 0.01, HEAD_Y0 + 0.80, HEAD_Y1 - 0.02, verts=20)
    apply_mat(tip, mat_metal)
    apply_transforms(tip)
    parts.append(tip)
    return parts


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    clear_scene()

    mat_dark = make_mat("Dark", (0.10, 0.11, 0.12, 1.0), metallic=0.25, roughness=0.55)
    mat_orange = make_mat(
        "Orange", (0.92, 0.38, 0.06, 1.0), metallic=0.08, roughness=0.40
    )
    mat_blue = make_mat("Blue", (0.30, 0.38, 0.46, 1.0), metallic=0.18, roughness=0.52)
    mat_recess = make_mat(
        "Recess", (0.04, 0.04, 0.05, 1.0), metallic=0.35, roughness=0.45
    )
    mat_metal = make_mat(
        "Metal", (0.62, 0.66, 0.68, 1.0), metallic=0.85, roughness=0.28
    )
    mat_tooth = make_mat(
        "Tooth", (0.88, 0.90, 0.90, 1.0), metallic=0.70, roughness=0.32
    )

    print("building body…", file=sys.stderr)
    body_parts = build_body(mat_dark, mat_orange, mat_blue, mat_recess)
    print("building head…", file=sys.stderr)
    head_parts = build_head(mat_metal, mat_tooth)

    print("joining Body / Head…", file=sys.stderr)
    join_objects("Body", body_parts)
    join_objects("Head", head_parts)

    export_glb(OUT_GLB)
    print(f"Wrote {OUT_GLB}", file=sys.stderr)


if __name__ == "__main__":
    main()
