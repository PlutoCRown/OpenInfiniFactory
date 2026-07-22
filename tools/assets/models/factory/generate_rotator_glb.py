"""用 Blender 生成旋转器 / 逆向旋转器外观 GLB。

Blender Z-up；俯视 -Z：
  - 底座蓝灰，竖棱倒角
  - 四侧水平箭头凹槽（方向随变体）
  - 顶部橙环 + 深色网格盘 + 双弧形橙箭头贴图（方向随变体）

一次生成：
  assets/factory_blocks/rotator/          （顺时针）
  assets/factory_blocks/counter_rotator/  （逆时针）

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/factory/generate_rotator_glb.py
"""

from __future__ import annotations

from pathlib import Path
import sys
_TOOLS = Path(__file__).resolve().parents[2]
if str(_TOOLS) not in sys.path:
    sys.path.insert(0, str(_TOOLS))
from common.paths import REPO_ROOT
from common.bpy_util import apply_mat, apply_transforms, boolean_diff, clear_scene, export_glb, join_by_material, link, make_mat, mesh_cube, mesh_cylinder, set_active
from common.png_util import write_png_rgba

import math
import struct
import zlib

import bpy
import bmesh
from mathutils import Matrix, Vector

OUT_ROTATOR = REPO_ROOT / "assets" / "factory_blocks" / "rotator"
OUT_COUNTER = REPO_ROOT / "assets" / "factory_blocks" / "counter_rotator"

CELL = 0.5
BASE_H = 0.72
BASE_Z0 = -CELL
BASE_Z1 = BASE_Z0 + BASE_H  # 0.22
CORNER_CHAMFER = 0.12

ARROW_DEPTH = 0.04
ARROW_SHAFT_H = 0.09
ARROW_HEAD_SIDE = 0.22
ARROW_SHAFT_LEN = 0.28

RING_R = 0.43
RING_INNER = 0.38
RING_H = 0.14
# 黑盘叠进橙环内侧；顶面略低，避免共面闪烁
DISK_R = 0.405
DISK_H = 0.04
DISK_GAP_Z = 0.015


def write_disk_texture(
    path: Path, *, clockwise: bool, size: int = 512
) -> bpy.types.Image:
    """深色网格盘 + 两道橙弧箭头；clockwise 控制箭头旋向。"""
    rgba = [0] * (size * size * 4)
    cx = cy = (size - 1) * 0.5
    bg = (0.12, 0.13, 0.14)
    grid = (0.22, 0.23, 0.24)
    orange = (0.92, 0.40, 0.06)

    def set_px(x: int, y: int, rgb: tuple[float, float, float]) -> None:
        if 0 <= x < size and 0 <= y < size:
            i = (y * size + x) * 4
            rgba[i] = int(rgb[0] * 255)
            rgba[i + 1] = int(rgb[1] * 255)
            rgba[i + 2] = int(rgb[2] * 255)
            rgba[i + 3] = 255

    for y in range(size):
        for x in range(size):
            dx = (x - cx) / (size * 0.5)
            dy = (y - cy) / (size * 0.5)
            if dx * dx + dy * dy > 1.0:
                set_px(x, y, (0, 0, 0))
                continue
            gx = abs((x % 8) - 4) < 1 or abs((y % 8) - 4) < 1
            set_px(x, y, grid if gx else bg)

    def paint_arc(r0: float, r1: float, a0: float, a1: float) -> None:
        steps = 220
        for i in range(steps):
            t = i / (steps - 1)
            a = a0 + (a1 - a0) * t
            for rr in range(int(r0 * size * 0.5), int(r1 * size * 0.5) + 1):
                px = int(cx + math.cos(a) * rr)
                py = int(cy + math.sin(a) * rr)
                set_px(px, py, orange)
                set_px(px + 1, py, orange)
                set_px(px, py + 1, orange)

    def fill_tri(
        p0: tuple[float, float], p1: tuple[float, float], p2: tuple[float, float]
    ) -> None:
        """填充实心三角形。"""
        xs = [p0[0], p1[0], p2[0]]
        ys = [p0[1], p1[1], p2[1]]
        minx, maxx = int(min(xs)), int(max(xs)) + 1
        miny, maxy = int(min(ys)), int(max(ys)) + 1

        def edge(ax, ay, bx, by, px, py):
            return (px - ax) * (by - ay) - (py - ay) * (bx - ax)

        for y in range(miny, maxy + 1):
            for x in range(minx, maxx + 1):
                w0 = edge(p1[0], p1[1], p2[0], p2[1], x, y)
                w1 = edge(p2[0], p2[1], p0[0], p0[1], x, y)
                w2 = edge(p0[0], p0[1], p1[0], p1[1], x, y)
                if (w0 >= 0 and w1 >= 0 and w2 >= 0) or (
                    w0 <= 0 and w1 <= 0 and w2 <= 0
                ):
                    set_px(x, y, orange)

    def paint_triangle_head(a: float, r_inner: float, r_outer: float) -> None:
        """弧末端实心三角；切向随 clockwise 翻转。"""
        scale = size * 0.5
        r_mid = (r_inner + r_outer) * 0.5
        mx = cx + math.cos(a) * r_mid * scale
        my = cy + math.sin(a) * r_mid * scale
        # 贴图与侧面同向时：顺时针模型用数学逆时针切向（UV 镜像效果）
        if clockwise:
            tx, ty = -math.sin(a), math.cos(a)
        else:
            tx, ty = math.sin(a), -math.cos(a)
        tip_len = size * 0.14
        flare = (r_outer - r_inner) * 0.45
        tip = (mx + tx * tip_len, my + ty * tip_len)
        inner = (
            cx + math.cos(a) * (r_inner - flare) * scale,
            cy + math.sin(a) * (r_inner - flare) * scale,
        )
        outer = (
            cx + math.cos(a) * (r_outer + flare) * scale,
            cy + math.sin(a) * (r_outer + flare) * scale,
        )
        fill_tri(tip, inner, outer)

    r_inner, r_outer = 0.46, 0.62
    sweep = math.radians(90) if clockwise else math.radians(-90)
    for base in (math.radians(-20), math.radians(160)):
        a1 = base + sweep
        paint_arc(r_inner, r_outer, base, a1)
        paint_triangle_head(a1, r_inner, r_outer)

    write_png_rgba(path, size, size, rgba)
    return bpy.data.images.load(str(path))


def make_arrow_prism(name: str) -> bpy.types.Object:
    """细杆 + 宽三角头，局部尖朝 +Y（与传送带相同）。"""
    shaft_len = ARROW_SHAFT_LEN
    shaft_h = ARROW_SHAFT_H
    head_side = ARROW_HEAD_SIDE
    tri_h = head_side * math.sqrt(3) * 0.5
    total_len = shaft_len + tri_h

    bm = bmesh.new()
    y0 = -total_len * 0.5
    y1 = y0 + shaft_len
    w0, w1 = -shaft_h * 0.5, shaft_h * 0.5
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
    d = ARROW_DEPTH * 0.5 + 0.005
    vf, vb = [], []
    for y, w in profile:
        vf.append(bm.verts.new((d, y, w)))
        vb.append(bm.verts.new((-d, y, w)))
    n = len(profile)
    bm.faces.new(vf)
    bm.faces.new(list(reversed(vb)))
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new([vf[i], vf[j], vb[j], vb[i]])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    link(obj)
    return obj


def chamfer_vertical_corners(obj: bpy.types.Object, width: float) -> None:
    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bm = bmesh.from_edit_mesh(obj.data)
    for e in bm.edges:
        e.select = False
    for e in bm.edges:
        xs = [v.co.x for v in e.verts]
        ys = [v.co.y for v in e.verts]
        zs = [v.co.z for v in e.verts]
        # 竖棱：x,y 近角，z 不同
        along_z = abs(zs[0] - zs[1]) > 0.2
        at_corner = abs(abs(xs[0]) - CELL) < 0.02 and abs(abs(ys[0]) - CELL) < 0.02
        if along_z and at_corner:
            e.select = True
    bmesh.update_edit_mesh(obj.data)
    bpy.ops.mesh.bevel(offset=width, segments=1, affect="EDGES")
    bpy.ops.object.mode_set(mode="OBJECT")


def build_base(mat: bpy.types.Material, *, clockwise: bool) -> bpy.types.Object:
    body = mesh_cube(
        "Base",
        Vector((1.0, 1.0, BASE_H)),
        Vector((0, 0, (BASE_Z0 + BASE_Z1) * 0.5)),
    )
    apply_mat(body, mat)
    apply_transforms(body)
    chamfer_vertical_corners(body, CORNER_CHAMFER)
    apply_mat(body, mat)

    # 四侧箭头：尖沿切向；clockwise=False 时反向
    face_specs = [
        ("PosY", Vector((0, 1, 0)), Vector((1, 0, 0))),
        ("PosX", Vector((1, 0, 0)), Vector((0, -1, 0))),
        ("NegY", Vector((0, -1, 0)), Vector((-1, 0, 0))),
        ("NegX", Vector((-1, 0, 0)), Vector((0, 1, 0))),
    ]
    for name, normal, tip_dir in face_specs:
        if not clockwise:
            tip_dir = -tip_dir
        cutter = make_arrow_prism(f"Arrow_{name}")
        x_axis = normal.normalized()
        y_axis = tip_dir.normalized()
        z_axis = x_axis.cross(y_axis).normalized()
        y_axis = z_axis.cross(x_axis).normalized()
        rot = Matrix((x_axis, y_axis, z_axis)).transposed().to_4x4()
        center = normal * (CELL - ARROW_DEPTH * 0.5) + Vector(
            (0, 0, (BASE_Z0 + BASE_Z1) * 0.5)
        )
        cutter.matrix_world = Matrix.Translation(center) @ rot
        boolean_diff(body, cutter)

    apply_mat(body, mat)
    return body


def build_top(
    mat_orange: bpy.types.Material,
    mat_disk: bpy.types.Material,
    mat_rivet: bpy.types.Material,
) -> None:
    ring_z = BASE_Z1 + RING_H * 0.5
    ring = mesh_cylinder("Ring", RING_R, RING_H, Vector((0, 0, ring_z)), verts=48)
    apply_mat(ring, mat_orange)
    apply_transforms(ring)

    # 内孔留给黑盘叠入
    boolean_diff(
        ring,
        mesh_cylinder(
            "RingCut",
            RING_INNER,
            RING_H + 0.04,
            Vector((0, 0, ring_z)),
            verts=48,
        ),
    )
    apply_mat(ring, mat_orange)

    # 黑盘半径大于内孔，叠在橙环下面；顶面略凹
    disk_top = BASE_Z1 + RING_H - DISK_GAP_Z
    disk = mesh_cylinder(
        "Disk",
        DISK_R,
        DISK_H,
        Vector((0, 0, disk_top - DISK_H * 0.5)),
        verts=48,
    )
    apply_mat(disk, mat_disk)
    apply_transforms(disk)
    set_active(disk)
    bpy.ops.object.mode_set(mode="EDIT")
    bm = bmesh.from_edit_mesh(disk.data)
    uv = bm.loops.layers.uv.verify()
    for face in bm.faces:
        for loop in face.loops:
            p = loop.vert.co
            loop[uv].uv = (p.x / (DISK_R * 2) + 0.5, p.y / (DISK_R * 2) + 0.5)
    bmesh.update_edit_mesh(disk.data)
    bpy.ops.object.mode_set(mode="OBJECT")
    apply_mat(disk, mat_disk)

    n_rivets = 12
    for i in range(n_rivets):
        a = math.pi * 2 * i / n_rivets
        r = (RING_R + RING_INNER) * 0.5
        rivet = mesh_cylinder(
            f"Rivet_{i}",
            0.016,
            0.028,
            Vector((math.cos(a) * r, math.sin(a) * r, BASE_Z1 + RING_H * 0.55)),
            verts=10,
        )
        apply_mat(rivet, mat_rivet)
        apply_transforms(rivet)


def main() -> None:
    OUT_ROTATOR.mkdir(parents=True, exist_ok=True)
    OUT_COUNTER.mkdir(parents=True, exist_ok=True)

    for clockwise, out_dir in ((True, OUT_ROTATOR), (False, OUT_COUNTER)):
        label = "rotator (CW)" if clockwise else "counter_rotator (CCW)"
        clear_scene()
        disk_path = out_dir / "disk_albedo.png"
        glb_path = out_dir / "model.glb"

        disk_img = write_disk_texture(disk_path, clockwise=clockwise)
        mat_base = make_mat(
            "Base", (0.34, 0.42, 0.48, 1.0), metallic=0.15, roughness=0.55
        )
        mat_orange = make_mat(
            "Orange", (0.92, 0.40, 0.06, 1.0), metallic=0.08, roughness=0.40
        )
        mat_disk = make_mat(
            "Disk", (0.12, 0.12, 0.13, 1.0), roughness=0.65, texture=disk_img
        )
        mat_rivet = make_mat(
            "Rivet", (0.15, 0.15, 0.16, 1.0), metallic=0.4, roughness=0.45
        )

        print(f"building {label}…", file=sys.stderr)
        build_base(mat_base, clockwise=clockwise)
        build_top(mat_orange, mat_disk, mat_rivet)
        join_by_material()
        export_glb(glb_path)
        print(f"Wrote {glb_path}", file=sys.stderr)


if __name__ == "__main__":
    main()
