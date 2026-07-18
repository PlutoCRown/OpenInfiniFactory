"""用 Blender 生成方块传感器 (Detector) 外观 GLB。

坐标系（Blender Z-up；export_yup 后工作面 → 游戏局部 -Z）：

  格：[-0.5, 0.5]³（1×1×1）
  机体：1×0.9×1，不居中——
    · 背面 / ±X / ±Z 五面贴齐格边
    · 工作面（+Y）凹进 0.1，落在 Y=+0.4
  供电口：按 1×1×1 格面中心放置（Y=0 等），不是按机体中心
  倒角：机体工作面与背面各一圈棱

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python scripts/factory_blocks/generate_detector_glb.py
"""

from __future__ import annotations

import math
import sys
from pathlib import Path

import bpy
import bmesh
from mathutils import Euler, Vector

ROOT = Path(__file__).resolve().parents[2]
OUT_DIR = ROOT / "assets" / "factory_blocks" / "detector"
OUT_GLB = OUT_DIR / "model.glb"

# 1×1×1 格
CELL = 0.5
# 机体：横截面 1×1，工作轴厚 0.9；贴齐背面，工作面凹进
BODY_XZ = 1.0
BODY_Y = 0.9
BODY_Y_MIN = -CELL  # -0.5 贴齐背面
BODY_Y_MAX = BODY_Y_MIN + BODY_Y  # +0.4 工作面
BODY_Y_CENTER = (BODY_Y_MIN + BODY_Y_MAX) * 0.5  # -0.05


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


def make_mat(
    name: str,
    color: tuple[float, float, float, float],
    *,
    metallic: float = 0.0,
    roughness: float = 0.55,
    emissive: tuple[float, float, float] | None = None,
    emissive_strength: float = 0.0,
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
    if emissive is not None:
        if "Emission Color" in bsdf.inputs:
            bsdf.inputs["Emission Color"].default_value = (*emissive, 1.0)
            bsdf.inputs["Emission Strength"].default_value = emissive_strength
        elif "Emission" in bsdf.inputs:
            bsdf.inputs["Emission"].default_value = (*emissive, 1.0)
    nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    return mat


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
    verts: int = 28,
) -> bpy.types.Object:
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


def bevel_front_and_back(obj: bpy.types.Object, width: float = 0.04) -> None:
    """倒机体工作面与背面各一圈棱（±Y）。"""
    apply_transforms(obj)
    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bm = bmesh.from_edit_mesh(obj.data)
    for e in bm.edges:
        e.select = False
    for e in bm.edges:
        ys = [v.co.y for v in e.verts]
        on_front = all(abs(y - BODY_Y_MAX) < 0.002 for y in ys)
        on_back = all(abs(y - BODY_Y_MIN) < 0.002 for y in ys)
        if on_front or on_back:
            e.select = True
    bmesh.update_edit_mesh(obj.data)
    bpy.ops.mesh.bevel(offset=width, segments=2, affect="EDGES")
    bpy.ops.object.mode_set(mode="OBJECT")


def build_body(mat: bpy.types.Material) -> bpy.types.Object:
    body = mesh_cube(
        "Body",
        Vector((BODY_XZ, BODY_Y, BODY_XZ)),
        Vector((0.0, BODY_Y_CENTER, 0.0)),
    )
    apply_mat(body, mat)
    bevel_front_and_back(body, width=0.045)
    return body


def add_port(
    prefix: str,
    center: Vector,
    axis: str,
    mat_metal: bpy.types.Material,
    mat_gold: bpy.types.Material,
    mat_dark: bpy.types.Material,
) -> None:
    plate = 0.40
    plate_t = 0.03
    hole_r, hole_d = 0.095, 0.035
    gold_major, gold_minor = 0.118, 0.009
    outer_major, outer_minor = 0.145, 0.007
    screw_r, screw_d = 0.013, 0.013
    screw_off = 0.145

    def finish(o: bpy.types.Object, mat: bpy.types.Material) -> None:
        apply_mat(o, mat)
        apply_transforms(o)

    if axis in ("+Z", "-Z"):
        sign = 1.0 if axis == "+Z" else -1.0
        finish(
            mesh_cube(f"{prefix}_Plate", Vector((plate, plate, plate_t)), center),
            mat_metal,
        )
        finish(
            mesh_cylinder(
                f"{prefix}_Hole", hole_r, hole_d, center + Vector((0, 0, sign * 0.002))
            ),
            mat_dark,
        )
        finish(
            mesh_torus(
                f"{prefix}_Gold",
                gold_major,
                gold_minor,
                center + Vector((0, 0, sign * 0.012)),
            ),
            mat_gold,
        )
        finish(
            mesh_torus(
                f"{prefix}_Outer",
                outer_major,
                outer_minor,
                center + Vector((0, 0, sign * 0.01)),
            ),
            mat_metal,
        )
        for i, (sx, sy) in enumerate(
            (
                (-screw_off, -screw_off),
                (screw_off, -screw_off),
                (-screw_off, screw_off),
                (screw_off, screw_off),
            )
        ):
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

    elif axis in ("+X", "-X"):
        sign = 1.0 if axis == "+X" else -1.0
        rot = Euler((0, math.radians(90), 0))
        finish(
            mesh_cube(f"{prefix}_Plate", Vector((plate_t, plate, plate)), center),
            mat_metal,
        )
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
        for i, (sy, sz) in enumerate(
            (
                (-screw_off, -screw_off),
                (screw_off, -screw_off),
                (-screw_off, screw_off),
                (screw_off, screw_off),
            )
        ):
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

    else:  # -Y 背面
        rot = Euler((math.radians(90), 0, 0))
        finish(
            mesh_cube(f"{prefix}_Plate", Vector((plate, plate_t, plate)), center),
            mat_metal,
        )
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
        for i, (sx, sz) in enumerate(
            (
                (-screw_off, -screw_off),
                (screw_off, -screw_off),
                (-screw_off, screw_off),
                (screw_off, screw_off),
            )
        ):
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


def build_display(
    mat_orange: bpy.types.Material, mat_screen: bpy.types.Material
) -> None:
    """正方形橙框 + 2×2；坐在凹进的工作面上，伸进 0.1 空隙里。"""
    y0 = BODY_Y_MAX
    frame_depth = 0.08
    y_frame = y0 + frame_depth * 0.5
    outer = 0.56

    frame = mesh_cube(
        "DisplayFrame", Vector((outer, frame_depth, outer)), Vector((0, y_frame, 0))
    )
    apply_mat(frame, mat_orange)
    apply_transforms(frame)
    boolean_diff(
        frame,
        mesh_cube("FrameCut", Vector((0.40, 0.12, 0.40)), Vector((0, y_frame, 0))),
    )
    # 框外沿再轻轻倒一点（仍是工作面一侧）
    set_active(frame)
    bpy.ops.object.mode_set(mode="EDIT")
    bm = bmesh.from_edit_mesh(frame.data)
    for e in bm.edges:
        e.select = False
    y_max = max(v.co.y for v in bm.verts)
    for e in bm.edges:
        if all(abs(v.co.y - y_max) < 0.012 for v in e.verts):
            e.select = True
    bmesh.update_edit_mesh(frame.data)
    bpy.ops.mesh.bevel(offset=0.01, segments=1, affect="EDGES")
    bpy.ops.object.mode_set(mode="OBJECT")

    for x in (-0.31, 0.31):
        ear = mesh_cube(f"Ear_{x}", Vector((0.05, 0.05, 0.10)), Vector((x, y_frame, 0)))
        apply_mat(ear, mat_orange)
        apply_transforms(ear)

    back = mesh_cube(
        "ScreenBack", Vector((0.38, 0.016, 0.38)), Vector((0, y0 + 0.012, 0))
    )
    apply_mat(back, mat_screen)
    apply_transforms(back)

    cell = 0.16
    gap = 0.02
    span = 2 * cell + gap
    y_cell = y0 + frame_depth - 0.018
    for r in range(2):
        for c in range(2):
            x = -span / 2 + cell / 2 + c * (cell + gap)
            z = span / 2 - cell / 2 - r * (cell + gap)
            sq = mesh_cube(
                f"Cell_{r}_{c}", Vector((cell, 0.018, cell)), Vector((x, y_cell, z))
            )
            apply_mat(sq, mat_screen)
            apply_transforms(sq)


def join_by_material() -> None:
    by_mat: dict[str, list[bpy.types.Object]] = {}
    for obj in list(bpy.context.scene.objects):
        if obj.type != "MESH":
            continue
        apply_transforms(obj)
        key = obj.data.materials[0].name if obj.data.materials else "_none"
        by_mat.setdefault(key, []).append(obj)

    for mat_name, group in by_mat.items():
        bpy.ops.object.select_all(action="DESELECT")
        for obj in group:
            obj.select_set(True)
        bpy.context.view_layer.objects.active = group[0]
        if len(group) > 1:
            bpy.ops.object.join()
        joined = bpy.context.active_object
        joined.name = f"Part_{mat_name}"
        if joined.data:
            joined.data.name = f"Mesh_{mat_name}"
        print(f"  {mat_name}: {len(group)} parts", file=sys.stderr)


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

    mat_body = make_mat("Body", (0.28, 0.36, 0.42, 1.0), metallic=0.18, roughness=0.55)
    mat_orange = make_mat(
        "Orange", (0.92, 0.36, 0.05, 1.0), metallic=0.08, roughness=0.38
    )
    mat_screen = make_mat(
        "Screen",
        (0.42, 0.04, 0.05, 1.0),
        roughness=0.28,
        emissive=(0.55, 0.02, 0.03),
        emissive_strength=1.4,
    )
    mat_metal = make_mat(
        "Metal", (0.58, 0.60, 0.62, 1.0), metallic=0.90, roughness=0.24
    )
    mat_gold = make_mat("Gold", (0.88, 0.68, 0.16, 1.0), metallic=0.95, roughness=0.20)
    mat_dark = make_mat("Dark", (0.04, 0.04, 0.05, 1.0), metallic=0.40, roughness=0.42)

    print(
        f"body Y=[{BODY_Y_MIN}, {BODY_Y_MAX}] center={BODY_Y_CENTER} (work recessed 0.1)",
        file=sys.stderr,
    )
    build_body(mat_body)

    # 供电口：贴在 1×1×1 格面中心（不是机体几何中心）
    pad = 0.016
    print("building 5 power ports on cell-face centers…", file=sys.stderr)
    add_port("PosZ", Vector((0, 0, CELL + pad)), "+Z", mat_metal, mat_gold, mat_dark)
    add_port("NegZ", Vector((0, 0, -CELL - pad)), "-Z", mat_metal, mat_gold, mat_dark)
    add_port("PosX", Vector((CELL + pad, 0, 0)), "+X", mat_metal, mat_gold, mat_dark)
    add_port("NegX", Vector((-CELL - pad, 0, 0)), "-X", mat_metal, mat_gold, mat_dark)
    add_port("NegY", Vector((0, -CELL - pad, 0)), "-Y", mat_metal, mat_gold, mat_dark)

    print("building display…", file=sys.stderr)
    build_display(mat_orange, mat_screen)

    print("joining…", file=sys.stderr)
    join_by_material()

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    export_glb(OUT_GLB)
    print(f"Wrote {OUT_GLB}", file=sys.stderr)


if __name__ == "__main__":
    main()
