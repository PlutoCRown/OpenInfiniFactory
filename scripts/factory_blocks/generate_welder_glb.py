"""用 Blender 生成焊接器 (Welder) 外观 GLB。

Blender Z-up；export_yup 后工作面 → 游戏局部 -Z（Blender +Y）：
  - 蓝灰外壳，前端上下唇罩、侧面 C 形开口
  - 凹进橙面板 + 银灰十字准星方块
  - 两侧各五道水平散热槽

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python scripts/factory_blocks/generate_welder_glb.py
"""

from __future__ import annotations

import math
import sys
from pathlib import Path

import bpy
import bmesh
from mathutils import Euler, Vector

ROOT = Path(__file__).resolve().parents[2]
OUT_DIR = ROOT / "assets" / "factory_blocks" / "welder"
OUT_GLB = OUT_DIR / "model.glb"

CELL = 0.5


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


def boolean_diff(target: bpy.types.Object, cutter: bpy.types.Object) -> None:
    apply_transforms(cutter)
    set_active(target)
    mod = target.modifiers.new("Bool", "BOOLEAN")
    mod.operation = "DIFFERENCE"
    mod.solver = "EXACT"
    mod.object = cutter
    bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.data.objects.remove(cutter, do_unlink=True)


def boolean_union(objs: list[bpy.types.Object]) -> bpy.types.Object:
    """把多个切削体合成一个。"""
    assert objs
    base = objs[0]
    apply_transforms(base)
    for other in objs[1:]:
        apply_transforms(other)
        set_active(base)
        mod = base.modifiers.new("BoolUnion", "BOOLEAN")
        mod.operation = "UNION"
        mod.solver = "EXACT"
        mod.object = other
        bpy.ops.object.modifier_apply(modifier=mod.name)
        bpy.data.objects.remove(other, do_unlink=True)
    return base


def clean_mesh(obj: bpy.types.Object) -> None:
    """合并重叠点、去掉退化面、融共面、统一法线。"""
    apply_transforms(obj)
    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.mesh.remove_doubles(threshold=1e-4)
    bpy.ops.mesh.dissolve_degenerate(threshold=1e-4)
    bpy.ops.mesh.dissolve_limited(angle_limit=math.radians(1.0))
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.object.mode_set(mode="OBJECT")


def bevel_outer_box_edges(obj: bpy.types.Object, width: float = 0.028) -> None:
    """只倒外壳后半立方体棱，避开前脸开口复杂区域。"""
    apply_transforms(obj)
    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bm = bmesh.from_edit_mesh(obj.data)
    bm.edges.ensure_lookup_table()
    for e in bm.edges:
        e.select = False

    def on_box_skin(v) -> bool:
        return (
            abs(abs(v.co.x) - CELL) < 0.003
            or abs(abs(v.co.y) - CELL) < 0.003
            or abs(abs(v.co.z) - CELL) < 0.003
        )

    for e in bm.edges:
        if not e.is_manifold or len(e.link_faces) != 2:
            continue
        v0, v1 = e.verts
        # 前脸开口附近不倒角，避免和斜切口缠在一起
        if v0.co.y > 0.20 or v1.co.y > 0.20:
            continue
        if not (on_box_skin(v0) and on_box_skin(v1)):
            continue
        axes0, axes1 = [], []
        for axes, v in ((axes0, v0), (axes1, v1)):
            if abs(abs(v.co.x) - CELL) < 0.003:
                axes.append("x")
            if abs(abs(v.co.y) - CELL) < 0.003:
                axes.append("y")
            if abs(abs(v.co.z) - CELL) < 0.003:
                axes.append("z")
        if len(set(axes0) & set(axes1)) >= 1 and (len(axes0) + len(axes1)) >= 3:
            e.select = True

    bmesh.update_edit_mesh(obj.data)
    bpy.ops.mesh.bevel(offset=width, segments=2, affect="EDGES", clamp_overlap=True)
    bpy.ops.object.mode_set(mode="OBJECT")


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


def mesh_loft_x(
    name: str, sections: list[tuple[float, list[tuple[float, float]]]]
) -> bpy.types.Object:
    """沿 X 放样闭合实体；每截面点数相同、绕序一致。"""
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    rings: list[list] = []
    n = len(sections[0][1])
    for x, pts in sections:
        assert len(pts) == n
        rings.append([bm.verts.new((x, y, z)) for y, z in pts])
    # 端盖
    bm.faces.new(rings[0])
    bm.faces.new(list(reversed(rings[-1])))
    # 侧面
    for ri in range(len(rings) - 1):
        a, b = rings[ri], rings[ri + 1]
        for i in range(n):
            j = (i + 1) % n
            bm.faces.new([a[i], a[j], b[j], b[i]])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    link(obj)
    return obj


def build_body(mat: bpy.types.Material) -> bpy.types.Object:
    body = mesh_cube("Body", Vector((1.0, 1.0, 1.0)), Vector((0, 0, 0)))
    apply_mat(body, mat)
    apply_transforms(body)

    # 前脸开口：单一体放样（中段矩形腔 + 两侧斜切 C），避免多布尔交线错乱
    y_back, y_front = 0.29, 0.52
    # 截面绕序：后下 → 前下 → 前上 → 后上（中段）；侧段同样 4 点梯形
    rect = [
        (y_back, -0.42),
        (y_front, -0.42),
        (y_front, 0.42),
        (y_back, 0.42),
    ]
    trap = [
        (y_back, -0.10),
        (y_front, -0.22),
        (y_front, 0.22),
        (y_back, 0.10),
    ]
    sections = [
        (-0.52, trap),
        (-0.40, rect),
        (0.40, rect),
        (0.52, trap),
    ]
    cutters: list[bpy.types.Object] = [mesh_loft_x("FrontVoid", sections)]

    # 两侧散热槽
    for side in (1.0, -1.0):
        for i in range(5):
            z = -0.18 + i * 0.09
            cutters.append(
                mesh_cube(
                    f"Vent_{side}_{i}",
                    Vector((0.06, 0.28, 0.035)),
                    Vector((side * (CELL - 0.02), -0.12, z)),
                )
            )

    cutter = boolean_union(cutters)
    boolean_diff(body, cutter)
    clean_mesh(body)
    apply_mat(body, mat)
    bevel_outer_box_edges(body, width=0.028)
    clean_mesh(body)
    apply_mat(body, mat)
    return body


def build_front(
    mat_orange: bpy.types.Material,
    mat_silver: bpy.types.Material,
    mat_dark: bpy.types.Material,
) -> None:
    # 橙面板：浅凹，靠近前缘
    orange_y = 0.30
    orange = mesh_cube(
        "Orange",
        Vector((0.78, 0.04, 0.78)),
        Vector((0.0, orange_y, 0.0)),
    )
    apply_mat(orange, mat_orange)
    apply_transforms(orange)

    # 左下角小孔
    hole = mesh_cylinder(
        "Port",
        0.045,
        0.05,
        Vector((-0.26, orange_y + 0.01, -0.26)),
        rot=Euler((math.pi * 0.5, 0, 0)),
        verts=16,
    )
    apply_mat(hole, mat_dark)
    apply_transforms(hole)

    # 银灰准星方块
    silver_y = orange_y + 0.028
    silver = mesh_cube(
        "Silver",
        Vector((0.34, 0.03, 0.34)),
        Vector((0.0, silver_y, 0.0)),
    )
    apply_mat(silver, mat_silver)
    apply_transforms(silver)

    # 对角十字线 + 中心点
    bar_len = 0.30
    bar_w = 0.028
    bar_t = 0.02
    cross_y = silver_y + 0.018
    for i, ang in enumerate((math.radians(45), math.radians(-45))):
        bar = mesh_cube(
            f"Cross_{i}",
            Vector((bar_len, bar_t, bar_w)),
            Vector((0.0, cross_y, 0.0)),
        )
        bar.rotation_euler = Euler((0.0, ang, 0.0))
        apply_mat(bar, mat_dark)
        apply_transforms(bar)

    center = mesh_cube(
        "CrossCenter",
        Vector((0.05, 0.022, 0.05)),
        Vector((0.0, cross_y + 0.002, 0.0)),
    )
    center.rotation_euler = Euler((0.0, math.radians(45), 0.0))
    apply_mat(center, mat_dark)
    apply_transforms(center)


def join_by_material() -> None:
    by_mat: dict[str, list[bpy.types.Object]] = {}
    for obj in list(bpy.context.scene.objects):
        if obj.type != "MESH" or not obj.data.materials:
            continue
        apply_transforms(obj)
        key = obj.data.materials[0].name
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
        export_image_format="AUTO",
    )


def main() -> None:
    clear_scene()
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    mat_body = make_mat("Body", (0.30, 0.38, 0.46, 1.0), metallic=0.18, roughness=0.52)
    mat_orange = make_mat(
        "Orange", (0.92, 0.40, 0.06, 1.0), metallic=0.08, roughness=0.40
    )
    mat_silver = make_mat(
        "Silver", (0.82, 0.84, 0.86, 1.0), metallic=0.35, roughness=0.35
    )
    mat_dark = make_mat("Dark", (0.10, 0.11, 0.12, 1.0), metallic=0.2, roughness=0.55)

    print("building body…", file=sys.stderr)
    build_body(mat_body)
    print("building front…", file=sys.stderr)
    build_front(mat_orange, mat_silver, mat_dark)
    print("joining…", file=sys.stderr)
    join_by_material()
    export_glb(OUT_GLB)
    print(f"Wrote {OUT_GLB}", file=sys.stderr)


if __name__ == "__main__":
    main()
