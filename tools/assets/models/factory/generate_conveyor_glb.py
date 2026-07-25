"""用 Blender 生成传送带 (Conveyor) 外观 GLB。

Blender Z-up；export_yup 后：
  - 传送方向（游戏 forward / -Z）← Blender +Y
  - 顶面 ← Blender +Z
  - 左右侧板 ← Blender ±X

侧面轮廓对应：
  width:100; height:98; border-radius:15px 15px 1px 1px
  → 橙主体高 0.98（顶面 z=0.48），皮带顶面齐格顶 z=0.50
  → 上圆角随滚筒半径（三轴接近但不相交），下圆角 0.01
  → 侧面沿外缘内缩的圆角凹板 + 凸出切削箭头；±Y 端竖棱斜切

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/factory/generate_conveyor_glb.py
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
    boolean_union,
    clear_scene,
    export_glb,
    join_by_material,
    link,
    make_mat,
    mesh_cube,
    mesh_cylinder,
    set_active,
)

import math

import bpy
import bmesh
from mathutils import Euler, Vector

OUT_DIR = REPO_ROOT / "assets" / "factory_blocks" / "conveyor"
OUT_GLB = OUT_DIR / "model.glb"
OUT_BELT_TEX = OUT_DIR / "belt_chevron.png"
OUT_BELT_NORMAL = OUT_DIR / "belt_chevron_normal.png"

CELL = 0.5
# 橙主体：宽/深 1.0，高 0.98，底贴齐 z=-0.5，顶在 z=0.48
ORANGE_H = 0.98
ORANGE_Z_MIN = -CELL
ORANGE_Z_MAX = ORANGE_Z_MIN + ORANGE_H  # 0.48
ORANGE_Z_CENTER = (ORANGE_Z_MIN + ORANGE_Z_MAX) * 0.5

WALL_T = 0.08
BELT_WIDTH = 1.0 - 2 * WALL_T
# 滚筒接近但不相交；外壳/皮带圆角跟着走
ROLLER_R = 0.15
CORNER_R = ROLLER_R
BOTTOM_BEVEL = 0.01  # 1/100
# ±Y 端竖棱斜切（蓝色标注）
END_CHAMFER = 0.055

# 皮带顶齐格顶 0.50；胶囊外半径略大于滚筒；半长撑满格
BELT_TOP = CELL  # 0.50
BELT_OUTER_R = ROLLER_R + 0.02  # 0.17
BELT_HALF_LEN = CELL - BELT_OUTER_R - 0.02  # 0.31；轴距 0.31，间隙 ≈0.01
ROLLER_Z = BELT_TOP - BELT_OUTER_R  # 0.33
ROLLER_YS = (-BELT_HALF_LEN, 0.0, BELT_HALF_LEN)

# 皮带槽底面抬高贴近皮带底；前后棱 45° 倒角
BELT_BOTTOM_Z = ROLLER_Z - BELT_OUTER_R  # 0.16
TRENCH_CLEARANCE = 0.02
TRENCH_FLOOR_Z = BELT_BOTTOM_Z - TRENCH_CLEARANCE  # 0.14
TRENCH_FLOOR_CHAMFER = 0.07

# 侧面凹板：沿侧轮廓内缩，上两角圆角
PANEL_DEPTH = 0.028
PANEL_MARGIN = 0.07  # 相对侧面外缘内缩
PANEL_CORNER_R = max(CORNER_R - PANEL_MARGIN, 0.04)
PANEL_INNER_W = 1.0 - 2 * PANEL_MARGIN  # Y 向
PANEL_INNER_H = ORANGE_H - 2 * PANEL_MARGIN  # Z 向
# 箭头在凹槽内四边等距；长度保持 ≈0.76，宽度撑满等距后的高度
ARROW_INSET = 0.05
ARROW_TOTAL_LEN = PANEL_INNER_W - 2 * ARROW_INSET  # ≈0.76
ARROW_TOTAL_H = PANEL_INNER_H - 2 * ARROW_INSET  # ≈0.74
ARROW_HEAD_LEN = ARROW_TOTAL_LEN * 0.38
ARROW_SHAFT_LEN = ARROW_TOTAL_LEN - ARROW_HEAD_LEN
ARROW_HEAD_SIDE = ARROW_TOTAL_H  # 三角底宽 = 包围盒高
ARROW_SHAFT_H = ARROW_TOTAL_H * 0.48  # 杆加宽
# 底面仍用浅挖箭头
ARROW_DEPTH = 0.045


def load_image(path: Path, *, is_data: bool = False) -> bpy.types.Image:
    """加载贴图；法线等数据贴图用 Non-Color。"""
    img = bpy.data.images.load(str(path))
    if is_data:
        img.colorspace_settings.name = "Non-Color"
    return img


def make_arrow_prism(
    name: str,
    *,
    axis: str,
    center: Vector,
    depth: float,
) -> bpy.types.Object:
    """箭头棱柱：宽杆 + 三角头，尖朝 +Y；包围盒居中于 center。"""
    shaft_len = ARROW_SHAFT_LEN
    shaft_h = ARROW_SHAFT_H
    head_side = ARROW_HEAD_SIDE
    tri_h = ARROW_HEAD_LEN
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


def make_side_panel_cutter(
    name: str, *, x_sign: float, depth: float
) -> bpy.types.Object:
    """侧面凹槽刀具：YZ 轮廓沿侧板外缘内缩，上两角圆角、下两角近直角。"""
    y0 = -CELL + PANEL_MARGIN
    y1 = CELL - PANEL_MARGIN
    z0 = ORANGE_Z_MIN + PANEL_MARGIN
    z1 = ORANGE_Z_MAX - PANEL_MARGIN
    r = min(PANEL_CORNER_R, (y1 - y0) * 0.45, (z1 - z0) * 0.45)
    segs = 8

    # 逆时针：底左 → 底右 → 右缘 → 右上圆弧 → 顶边 → 左上圆弧 → 左缘
    profile: list[tuple[float, float]] = [
        (y0, z0),
        (y1, z0),
        (y1, z1 - r),
    ]
    # 右上圆角：圆心 (y1-r, z1-r)，角 0→π/2
    cx_r, cz = y1 - r, z1 - r
    for i in range(1, segs + 1):
        ang = (math.pi / 2) * (i / segs)
        profile.append((cx_r + r * math.cos(ang), cz + r * math.sin(ang)))
    # 左上圆角：圆心 (y0+r, z1-r)，角 π/2→π
    cx_l = y0 + r
    for i in range(1, segs + 1):
        ang = math.pi / 2 + (math.pi / 2) * (i / segs)
        profile.append((cx_l + r * math.cos(ang), cz + r * math.sin(ang)))

    bm = bmesh.new()
    d = depth * 0.5
    v_front = []
    v_back = []
    for y, z in profile:
        v_front.append(bm.verts.new((d, y, z)))
        v_back.append(bm.verts.new((-d, y, z)))
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
    # 刀具中心落在侧面向内 depth/2 处，略伸出原表面
    obj.location = Vector((x_sign * (CELL - depth * 0.5 + 0.005), 0.0, 0.0))
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


def chamfer_end_verticals(obj: bpy.types.Object, width: float) -> None:
    """斜切 ±X 侧板在 ±Y 端的竖棱（蓝色标注）。"""

    def is_end_vertical(e) -> bool:
        xs = [v.co.x for v in e.verts]
        ys = [v.co.y for v in e.verts]
        zs = [v.co.z for v in e.verts]
        along_z = abs(zs[0] - zs[1]) > 0.3
        at_x = all(abs(abs(x) - CELL) < 0.002 for x in xs)
        at_y = all(abs(abs(y) - CELL) < 0.002 for y in ys)
        return along_z and at_x and at_y

    n = select_edges(obj, is_end_vertical)
    print(f"  end chamfer edges: {n}", file=sys.stderr)
    if n:
        bevel_selected_edges(obj, width, segments=1)
    else:
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
    chamfer_end_verticals(body, END_CHAMFER)
    apply_mat(body, mat_orange)

    def is_top_end_edge(e) -> bool:
        zs = [v.co.z for v in e.verts]
        ys = [v.co.y for v in e.verts]
        xs = [v.co.x for v in e.verts]
        on_top = all(abs(z - ORANGE_Z_MAX) < 0.002 for z in zs)
        along_x = abs(ys[0] - ys[1]) < 0.002 and abs(xs[0] - xs[1]) > 0.35
        # 斜切后顶边端点略内缩，用稍松的 Y 阈值
        near_end = abs(ys[0]) > CELL - END_CHAMFER - 0.01
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

    # 中间皮带槽：底面抬高贴近皮带，侧墙保留 WALL_T
    trench_z_max = ORANGE_Z_MAX + 0.15
    trench_h = trench_z_max - TRENCH_FLOOR_Z
    trench_cz = (trench_z_max + TRENCH_FLOOR_Z) * 0.5
    boolean_diff(
        body,
        mesh_cube(
            "BeltTrench",
            Vector((BELT_WIDTH, 1.05, trench_h)),
            Vector((0, 0, trench_cz)),
        ),
    )
    apply_mat(body, mat_orange)

    # 槽底前后 45° 斜面：等腰直角三角楔切掉棱角
    for y_sign in (-1.0, 1.0):
        bm = bmesh.new()
        extent = TRENCH_FLOOR_CHAMFER + 0.001
        y_out = y_sign * (CELL + 0.001)
        y_in = y_sign * (CELL - TRENCH_FLOOR_CHAMFER)
        z_hi = TRENCH_FLOOR_Z + 0.001
        z_lo = z_hi - extent  # 与横向跨度等长 → 严格 45°
        profile = [
            (y_out, z_hi),
            (y_out, z_lo),
            (y_in, z_hi),
        ]
        hx = BELT_WIDTH * 0.5  # 与两侧挡板内壁齐平，不切进墙体
        v_a, v_b = [], []
        for y, z in profile:
            v_a.append(bm.verts.new((hx, y, z)))
            v_b.append(bm.verts.new((-hx, y, z)))
        bm.faces.new(v_a)
        bm.faces.new(list(reversed(v_b)))
        for i in range(3):
            j = (i + 1) % 3
            bm.faces.new([v_a[i], v_a[j], v_b[j], v_b[i]])
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        mesh = bpy.data.meshes.new(f"FloorChamfer_{y_sign}")
        bm.to_mesh(mesh)
        bm.free()
        cutter = bpy.data.objects.new(f"FloorChamfer_{y_sign}", mesh)
        link(cutter)
        boolean_diff(body, cutter)
    apply_mat(body, mat_orange)

    # 左右侧面：沿侧轮廓内缩的圆角凹板，再并上与外缘齐平的箭头
    for x_sign in (-1.0, 1.0):
        panel = make_side_panel_cutter(
            f"SidePanel_{x_sign}",
            x_sign=x_sign,
            depth=PANEL_DEPTH + 0.01,
        )
        boolean_diff(body, panel)

        # 箭头外缘与方块侧面齐平，根部落在凹板底
        arrow_cx = x_sign * (CELL - PANEL_DEPTH * 0.5)
        arrow = make_arrow_prism(
            f"ArrowSide_{x_sign}",
            axis="x",
            center=Vector((arrow_cx, 0.0, ORANGE_Z_CENTER)),
            depth=PANEL_DEPTH,
        )
        apply_mat(arrow, mat_orange)
        boolean_union(body, arrow)
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


def build_belt_capsule(mat_belt: bpy.types.Material) -> bpy.types.Object:
    """胶囊棱柱：YZ 跑道外环挤出带宽。

    UV：U=带宽[0,1]；V=弧长/带宽（可 >1，贴图 REPEAT）。
    方图一格对应「宽×宽」的物理块，环向按比例循环，相对宽度不拉伸。
    """
    L = BELT_HALF_LEN
    R = BELT_OUTER_R
    cz = ROLLER_Z
    half_w = (BELT_WIDTH - 0.02) * 0.5
    belt_w = half_w * 2.0
    loop_len = 4.0 * L + 2.0 * math.pi * R
    # 环向要铺几格方图：loop_len / belt_w
    print(
        f"  belt UV tiles V={loop_len / belt_w:.3f} (loop={loop_len:.3f} / width={belt_w:.3f})",
        file=sys.stderr,
    )

    # 直线段与圆弧段取相近弦长，避免 V 向疏密不均
    n_arc = 24
    seg = math.pi * R / n_arc
    n_flat = max(2, round((2.0 * L) / seg))

    # 外环折线 (y, z, s)，s∈[0, loop_len)；闭合边用 s=loop_len
    outline: list[tuple[float, float, float]] = []
    s = 0.0

    # 顶：-L → +L
    for i in range(n_flat + 1):
        y = -L + (2.0 * L) * (i / n_flat)
        if i:
            s += (2.0 * L) / n_flat
        outline.append((y, cz + R, s))

    # +Y 半圆：θ = π/2 → -π/2（不含起点）
    for i in range(1, n_arc + 1):
        theta = math.pi / 2.0 - math.pi * (i / n_arc)
        s += (math.pi * R) / n_arc
        outline.append((L + R * math.cos(theta), cz + R * math.sin(theta), s))

    # 底：+L → -L（不含起点）
    for i in range(1, n_flat + 1):
        y = L - (2.0 * L) * (i / n_flat)
        s += (2.0 * L) / n_flat
        outline.append((y, cz - R, s))

    # -Y 半圆：θ = -π/2 → -3π/2（不含起点与终点，终点=顶起点）
    for i in range(1, n_arc):
        theta = -math.pi / 2.0 - math.pi * (i / n_arc)
        s += (math.pi * R) / n_arc
        outline.append((-L + R * math.cos(theta), cz + R * math.sin(theta), s))

    bm = bmesh.new()
    uv = bm.loops.layers.uv.new()
    n = len(outline)
    v_l = [bm.verts.new((-half_w, y, z)) for y, z, _ in outline]
    v_r = [bm.verts.new((half_w, y, z)) for y, z, _ in outline]

    def set_uv(face: bmesh.types.BMFace, coords: list[tuple[float, float]]) -> None:
        for loop, (uu, vv) in zip(face.loops, coords):
            # 整圈翻转，使人字尖朝传送方向
            loop[uv].uv = (1.0 - uu, 1.0 - vv)

    # 环带侧面：V = 弧长/带宽（方图循环）
    for i in range(n):
        j = (i + 1) % n
        s0 = outline[i][2]
        s1 = outline[j][2] if j else loop_len
        v0 = s0 / belt_w
        v1 = s1 / belt_w
        face = bm.faces.new([v_l[i], v_r[i], v_r[j], v_l[j]])
        set_uv(face, [(0.0, v0), (1.0, v0), (1.0, v1), (0.0, v1)])

    # ±X 端盖（夹在侧墙里，几乎不可见）
    face_l = bm.faces.new(list(reversed(v_l)))
    set_uv(
        face_l,
        [(0.0, outline[i][2] / belt_w) for i in range(n - 1, -1, -1)],
    )
    face_r = bm.faces.new(v_r)
    set_uv(face_r, [(1.0, outline[i][2] / belt_w) for i in range(n)])

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new("Belt")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Belt", mesh)
    link(obj)
    apply_mat(obj, mat_belt)
    apply_transforms(obj)
    return obj


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


def main() -> None:
    clear_scene()

    # 先用贴图脚本生成无缝人字 albedo/normal
    from textures.generate_conveyor_belt_texture import write_belt_textures

    albedo_path, normal_path = write_belt_textures(OUT_DIR)
    chevron_img = load_image(albedo_path)
    chevron_img.pack()
    normal_img = load_image(normal_path, is_data=True)
    normal_img.pack()

    mat_orange = make_mat(
        "Orange", (0.92, 0.42, 0.08, 1.0), metallic=0.06, roughness=0.42
    )
    mat_belt = make_mat(
        "Belt",
        (0.08, 0.08, 0.09, 1.0),
        metallic=0.05,
        roughness=0.68,
        texture=chevron_img,
        normal=normal_img,
        normal_strength=1.15,
    )
    # 细槽用平滑插值，避免 Closest 锯齿
    for node in mat_belt.node_tree.nodes:
        if getattr(node, "type", "") == "TEX_IMAGE":
            node.interpolation = "Smart"
    mat_roller = make_mat(
        "Roller", (0.86, 0.87, 0.88, 1.0), metallic=0.65, roughness=0.28
    )

    print(
        f"orange z=[{ORANGE_Z_MIN:.2f},{ORANGE_Z_MAX:.2f}] belt_top={BELT_TOP} "
        f"corner_r={CORNER_R} roller_r={ROLLER_R} half_len={BELT_HALF_LEN}",
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
