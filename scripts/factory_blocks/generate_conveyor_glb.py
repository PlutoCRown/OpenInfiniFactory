"""用 Blender 生成传送带 (Conveyor) 外观 GLB。

Blender Z-up；export_yup 后：
  - 传送方向（游戏 forward / -Z）← Blender +Y
  - 顶面 ← Blender +Z
  - 左右侧板 ← Blender ±X

侧面轮廓对应：
  width:100; height:98; border-radius:20px 20px 1px 1px
  → 橙主体高 0.98（顶面 z=0.48），皮带顶面齐格顶 z=0.50
  → 上圆角 0.20，下圆角 0.01

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python scripts/factory_blocks/generate_conveyor_glb.py
"""

from __future__ import annotations

import math
import sys
from pathlib import Path

import bpy
import bmesh
from mathutils import Euler, Vector

ROOT = Path(__file__).resolve().parents[2]
OUT_DIR = ROOT / "assets" / "factory_blocks" / "conveyor"
OUT_GLB = OUT_DIR / "model.glb"
OUT_BELT_TEX = OUT_DIR / "belt_chevron.png"

CELL = 0.5
# 橙主体：宽/深 1.0，高 0.98，底贴齐 z=-0.5，顶在 z=0.48
ORANGE_H = 0.98
ORANGE_Z_MIN = -CELL
ORANGE_Z_MAX = ORANGE_Z_MIN + ORANGE_H  # 0.48
ORANGE_Z_CENTER = (ORANGE_Z_MIN + ORANGE_Z_MAX) * 0.5

WALL_T = 0.08
BELT_WIDTH = 1.0 - 2 * WALL_T
# 上圆角 / 滚筒半径（20/100）
CORNER_R = 0.20
ROLLER_R = CORNER_R
BOTTOM_BEVEL = 0.01  # 1/100

# 皮带顶齐格顶 0.50；胶囊外半径略大于滚筒
BELT_TOP = CELL  # 0.50
BELT_OUTER_R = ROLLER_R + 0.02  # 0.22
ROLLER_Z = BELT_TOP - BELT_OUTER_R  # 0.28
ROLLER_YS = (-0.28, 0.0, 0.28)
BELT_HALF_LEN = 0.28

# 箭头凹槽：浅挖，杆比三角底细；整体包围盒居中
ARROW_DEPTH = 0.045
ARROW_SHAFT_H = 0.10
ARROW_HEAD_SIDE = 0.26  # 等边三角边长（=底宽）
ARROW_SHAFT_LEN = 0.30


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
    verts: int = 32,
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


def boolean_union(target: bpy.types.Object, other: bpy.types.Object) -> None:
    apply_transforms(other)
    set_active(target)
    mod = target.modifiers.new("BoolUnion", "BOOLEAN")
    mod.operation = "UNION"
    mod.solver = "EXACT"
    mod.object = other
    bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.data.objects.remove(other, do_unlink=True)


def make_mat(
    name: str,
    color: tuple[float, float, float, float],
    *,
    metallic: float = 0.0,
    roughness: float = 0.55,
    texture: bpy.types.Image | None = None,
) -> bpy.types.Material:
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    nt = mat.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
    bsdf.location = (200, 0)
    out.location = (450, 0)
    if texture is not None:
        tex = nt.nodes.new("ShaderNodeTexImage")
        tex.image = texture
        tex.location = (-200, 0)
        tex.interpolation = "Closest"
        nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    else:
        bsdf.inputs["Base Color"].default_value = color
    bsdf.inputs["Metallic"].default_value = metallic
    bsdf.inputs["Roughness"].default_value = roughness
    # 双面，避免凹槽翻面发白
    if "Alpha" in bsdf.inputs:
        pass
    mat.use_backface_culling = False
    nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    return mat


def write_chevron_texture(path: Path, size: int = 256) -> bpy.types.Image:
    """清晰单向人字：每排一个 ^ 尖朝 +V（传送方向），不相交成网。"""
    path.parent.mkdir(parents=True, exist_ok=True)
    bg = (0.07, 0.07, 0.08)
    fg = (0.55, 0.55, 0.58)
    img = bpy.data.images.new("BeltChevron", width=size, height=size, alpha=False)
    pixels = list(bg) + [1.0]
    pixels = pixels * (size * size)

    def set_px(x: int, y: int, rgb: tuple[float, float, float]) -> None:
        if 0 <= x < size and 0 <= y < size:
            i = (y * size + x) * 4
            pixels[i], pixels[i + 1], pixels[i + 2] = rgb

    def thick_line(x0: float, y0: float, x1: float, y1: float, w: int = 7) -> None:
        steps = max(int(math.hypot(x1 - x0, y1 - y0)), 1)
        for s in range(steps + 1):
            t = s / steps
            cx = x0 + (x1 - x0) * t
            cy = y0 + (y1 - y0) * t
            for dx in range(-w, w + 1):
                for dy in range(-w, w + 1):
                    if dx * dx + dy * dy <= w * w:
                        set_px(int(cx) + dx, int(cy) + dy, fg)

    # 4 排独立 V，尖朝 +V（传送 +Y），左右臂不相交成菱形网
    for band in range(4):
        base_y = size * (0.10 + band * 0.22)
        tip_y = base_y + size * 0.14
        mid = size * 0.5
        half = size * 0.32
        thick_line(mid, tip_y, mid - half, base_y, w=6)
        thick_line(mid, tip_y, mid + half, base_y, w=6)

    img.pixels = pixels
    img.pack()
    img.filepath_raw = str(path)
    img.file_format = "PNG"
    img.save()
    return img


def make_arrow_prism(
    name: str,
    *,
    axis: str,
    center: Vector,
    depth: float,
) -> bpy.types.Object:
    """箭头棱柱：细杆 + 更宽的等边三角头，尖朝 +Y；整体包围盒居中于 center。"""
    shaft_len = ARROW_SHAFT_LEN
    shaft_h = ARROW_SHAFT_H
    head_side = ARROW_HEAD_SIDE
    tri_h = head_side * math.sqrt(3) * 0.5
    total_len = shaft_len + tri_h

    bm = bmesh.new()
    # 杆 + 头整体在 Y 上居中
    y0 = -total_len * 0.5
    y1 = y0 + shaft_len
    w0 = -shaft_h * 0.5
    w1 = shaft_h * 0.5
    base_y = y1
    tip_y = base_y + tri_h
    hb = head_side * 0.5

    profile = [
        (y0, w0),
        (y0, w1),
        (base_y, w1),
        (base_y, hb),
        (tip_y, 0.0),
        (base_y, -hb),
        (base_y, w0),
    ]

    d = depth * 0.5
    v_front = []
    v_back = []
    for y, w in profile:
        if axis == "x":
            v_front.append(bm.verts.new((d, y, w)))
            v_back.append(bm.verts.new((-d, y, w)))
        else:
            v_front.append(bm.verts.new((w, y, d)))
            v_back.append(bm.verts.new((w, y, -d)))

    n = len(profile)
    bm.faces.new(v_front)
    bm.faces.new(list(reversed(v_back)))
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new([v_front[i], v_front[j], v_back[j], v_back[i]])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)

    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    link(obj)
    obj.location = center
    return obj


def select_edges(obj: bpy.types.Object, pred) -> int:
    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bm = bmesh.from_edit_mesh(obj.data)
    count = 0
    for e in bm.edges:
        e.select = bool(pred(e))
        if e.select:
            count += 1
    bmesh.update_edit_mesh(obj.data)
    return count


def bevel_selected_edges(obj: bpy.types.Object, width: float, segments: int) -> None:
    # 假定已在 EDIT 且已选边
    bpy.ops.mesh.bevel(offset=width, segments=segments, affect="EDGES")
    bpy.ops.object.mode_set(mode="OBJECT")


def build_chassis(mat_orange: bpy.types.Material) -> bpy.types.Object:
    body = mesh_cube(
        "Chassis",
        Vector((1.0, 1.0, ORANGE_H)),
        Vector((0, 0, ORANGE_Z_CENTER)),
    )
    apply_mat(body, mat_orange)
    apply_transforms(body)

    # —— 先在完整方盒上倒角（布尔前拓扑干净）——
    def is_top_end_edge(e) -> bool:
        zs = [v.co.z for v in e.verts]
        ys = [v.co.y for v in e.verts]
        xs = [v.co.x for v in e.verts]
        on_top = all(abs(z - ORANGE_Z_MAX) < 0.002 for z in zs)
        along_x = abs(ys[0] - ys[1]) < 0.002 and abs(xs[0] - xs[1]) > 0.5
        near_end = abs(ys[0]) > CELL - 0.002
        return on_top and along_x and near_end

    n = select_edges(body, is_top_end_edge)
    print(f"  top corner edges: {n}", file=sys.stderr)
    if n:
        bevel_selected_edges(body, CORNER_R, segments=8)
    else:
        bpy.ops.object.mode_set(mode="OBJECT")

    def is_bottom_edge(e) -> bool:
        zs = [v.co.z for v in e.verts]
        return all(abs(z - ORANGE_Z_MIN) < 0.002 for z in zs)

    n = select_edges(body, is_bottom_edge)
    print(f"  bottom edges: {n}", file=sys.stderr)
    if n:
        bevel_selected_edges(body, BOTTOM_BEVEL, segments=1)
    else:
        bpy.ops.object.mode_set(mode="OBJECT")

    apply_mat(body, mat_orange)

    # 中间皮带槽（侧墙保留 WALL_T）
    boolean_diff(
        body,
        mesh_cube(
            "BeltTrench",
            Vector((BELT_WIDTH, 1.05, 0.70)),
            Vector((0, 0, ORANGE_Z_MAX - 0.15)),
        ),
    )
    apply_mat(body, mat_orange)

    # 左右侧面浅挖箭头：落在侧板视觉中心（Y/Z 都居中）
    for x_sign in (-1.0, 1.0):
        cutter = make_arrow_prism(
            f"ArrowSide_{x_sign}",
            axis="x",
            center=Vector((x_sign * (CELL - ARROW_DEPTH * 0.5), 0.0, ORANGE_Z_CENTER)),
            depth=ARROW_DEPTH + 0.01,
        )
        boolean_diff(body, cutter)
    apply_mat(body, mat_orange)

    # 底面浅挖箭头（XY 居中）
    bottom_cutter = make_arrow_prism(
        "ArrowBottom",
        axis="z",
        center=Vector((0.0, 0.0, ORANGE_Z_MIN + ARROW_DEPTH * 0.5)),
        depth=ARROW_DEPTH + 0.01,
    )
    boolean_diff(body, bottom_cutter)
    apply_mat(body, mat_orange)
    return body


def belt_loop_s(y: float, z: float) -> float:
    """胶囊外环弧长：顶(-L→+L) → +Y半圆 → 底(+L→-L) → -Y半圆。"""
    L = BELT_HALF_LEN
    R = BELT_OUTER_R
    cz = ROLLER_Z

    if y >= L:
        ang = math.atan2(z - cz, y - L)
        ang = max(-math.pi / 2, min(math.pi / 2, ang))
        return 2 * L + R * (math.pi / 2 - ang)
    if y <= -L:
        vx, vz = y + L, z - cz
        # 自底边顺时针经 -Y 到顶：0 → π
        ang = math.atan2(-vx, -vz)
        if ang < 0:
            ang += 2 * math.pi
        ang = max(0.0, min(math.pi, ang))
        return 4 * L + math.pi * R + R * ang
    if z >= cz:
        return y + L
    return 2 * L + math.pi * R + (L - y)


def unwrap_belt_strip(obj: bpy.types.Object) -> None:
    """整圈展开成一条带：U=宽度，V=环向弧长（可平移做传送动画）。"""
    apply_transforms(obj)
    L = BELT_HALF_LEN
    R = BELT_OUTER_R
    loop_len = 4 * L + 2 * math.pi * R
    half_w = (BELT_WIDTH - 0.02) * 0.5

    mesh = obj.data
    bm = bmesh.new()
    bm.from_mesh(mesh)
    uv_layer = bm.loops.layers.uv.verify()

    for face in bm.faces:
        for loop in face.loops:
            p = loop.vert.co
            u = (p.x / half_w) * 0.5 + 0.5
            v = belt_loop_s(p.y, p.z) / loop_len
            # 顶面箭头方向：整圈 UV 旋转 180°
            loop[uv_layer].uv = (1.0 - u, 1.0 - v)

    bm.to_mesh(mesh)
    bm.free()
    mesh.update()


def build_belt_capsule(mat_belt: bpy.types.Material) -> bpy.types.Object:
    """侧面胶囊：中间长方 + 两端圆柱；UV 展成连续环带。"""
    mid = mesh_cube(
        "BeltMid",
        Vector((BELT_WIDTH - 0.02, BELT_HALF_LEN * 2, BELT_OUTER_R * 2)),
        Vector((0, 0, ROLLER_Z)),
    )
    apply_mat(mid, mat_belt)
    apply_transforms(mid)

    rot = Euler((0, math.radians(90), 0))
    for sign in (-1.0, 1.0):
        cap = mesh_cylinder(
            f"BeltCap_{sign}",
            BELT_OUTER_R,
            BELT_WIDTH - 0.02,
            Vector((0, sign * BELT_HALF_LEN, ROLLER_Z)),
            rot=rot,
            verts=28,
        )
        apply_mat(cap, mat_belt)
        boolean_union(mid, cap)

    unwrap_belt_strip(mid)
    apply_mat(mid, mat_belt)
    return mid


def build_rollers(mat_roller: bpy.types.Material) -> None:
    """圆柱长度接到两侧橙墙内表面（略嵌入）。"""
    rot = Euler((0, math.radians(90), 0))
    # 墙内表面在 ±(0.5 - WALL_T)；再嵌入 0.03
    length = BELT_WIDTH + 0.06
    for i, y in enumerate(ROLLER_YS):
        roller = mesh_cylinder(
            f"Roller_{i}",
            ROLLER_R,
            length,
            Vector((0, y, ROLLER_Z)),
            rot=rot,
            verts=24,
        )
        apply_mat(roller, mat_roller)
        apply_transforms(roller)


def join_by_material() -> None:
    by_mat: dict[str, list[bpy.types.Object]] = {}
    for obj in list(bpy.context.scene.objects):
        if obj.type != "MESH":
            continue
        apply_transforms(obj)
        if not obj.data.materials:
            continue
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

    chevron_img = write_chevron_texture(OUT_BELT_TEX)
    mat_orange = make_mat(
        "Orange", (0.92, 0.42, 0.08, 1.0), metallic=0.06, roughness=0.42
    )
    mat_belt = make_mat(
        "Belt",
        (0.08, 0.08, 0.09, 1.0),
        metallic=0.05,
        roughness=0.68,
        texture=chevron_img,
    )
    mat_roller = make_mat(
        "Roller", (0.86, 0.87, 0.88, 1.0), metallic=0.65, roughness=0.28
    )

    print(
        f"orange z=[{ORANGE_Z_MIN:.2f},{ORANGE_Z_MAX:.2f}] belt_top={BELT_TOP} corner_r={CORNER_R}",
        file=sys.stderr,
    )
    print("building chassis…", file=sys.stderr)
    build_chassis(mat_orange)
    print("building belt…", file=sys.stderr)
    build_belt_capsule(mat_belt)
    print("building rollers…", file=sys.stderr)
    build_rollers(mat_roller)

    print("joining…", file=sys.stderr)
    join_by_material()

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    export_glb(OUT_GLB)
    print(f"Wrote {OUT_GLB}", file=sys.stderr)


if __name__ == "__main__":
    main()
