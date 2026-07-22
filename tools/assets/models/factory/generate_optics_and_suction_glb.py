"""用 Blender 生成激光 / 镜子 / 分光镜 / 吸盘外观 GLB。

几何对齐现有游戏零件（render_assets.rs），外观做成工厂块风格。
Blender Z-up；export_yup 后前进方向 → 游戏局部 -Z（Blender +Y）。

产出：
  assets/factory_blocks/laser/model.glb
  assets/factory_blocks/mirror/model.glb
  assets/factory_blocks/vertical_mirror/model.glb
  assets/factory_blocks/splitter/model.glb
  assets/factory_blocks/suction_cup/model.glb

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/factory/generate_optics_and_suction_glb.py
"""

from __future__ import annotations

from pathlib import Path
import sys
_TOOLS = Path(__file__).resolve().parents[2]
if str(_TOOLS) not in sys.path:
    sys.path.insert(0, str(_TOOLS))
from common.paths import REPO_ROOT
from common.bpy_util import apply_mat, apply_transforms, boolean_diff, clear_scene, export_glb, join_by_material, link, make_mat, mesh_cube, set_active

import math

import bpy
import bmesh
from mathutils import Matrix, Vector

OUT_ROOT = REPO_ROOT / "assets" / "factory_blocks"
THICK = 0.06
CELL = 0.5


def mesh_cylinder(
    name: str, radius: float, depth: float, loc: Vector, *, verts: int = 32
) -> bpy.types.Object:
    """圆柱默认沿 +Y（前进方向），与公共 bpy_util 沿 Z 版不同。"""
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
    bmesh.ops.rotate(
        bm,
        cent=(0, 0, 0),
        matrix=Matrix.Rotation(math.radians(90), 3, "X"),
        verts=bm.verts,
    )
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    link(obj)
    obj.location = loc
    return obj


def mesh_from_faces(
    name: str,
    positions: list[Vector],
    faces: list[list[int]],
) -> bpy.types.Object:
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    verts = [bm.verts.new(p) for p in positions]
    for face in faces:
        bm.faces.new([verts[i] for i in face])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    return link(bpy.data.objects.new(name, mesh))


def thick_face(
    name: str,
    front: list[Vector],
    thickness: float = THICK,
) -> bpy.types.Object:
    """双面加厚镜片：front 为正面顶点（绕序使法线朝外）。"""
    n = len(front)
    normal = (front[1] - front[0]).cross(front[2] - front[0]).normalized()
    back = [p - normal * thickness for p in front]
    positions = list(front) + back
    faces: list[list[int]] = []
    for i in range(1, n - 1):
        faces.append([0, i, i + 1])
    for i in range(1, n - 1):
        faces.append([n, n + i + 1, n + i])
    for i in range(n):
        j = (i + 1) % n
        faces.append([i, j, n + j, n + i])
    return mesh_from_faces(name, positions, faces)


def game_to_blender(gx: float, gy: float, gz: float) -> Vector:
    """游戏 Y-up → Blender Z-up（export_yup 可还原）。"""
    return Vector((gx, -gz, gy))


# ——— 激光 ———


def build_laser() -> None:
    """暗红炮管朝 +Y，深灰机身 + 顶供电板。"""
    mat_body = make_mat("Body", (0.14, 0.15, 0.17, 1.0), metallic=0.25, roughness=0.48)
    mat_laser = make_mat(
        "Laser",
        (0.85, 0.08, 0.16, 1.0),
        metallic=0.1,
        roughness=0.35,
        emission=(1.0, 0.10, 0.22),
        emission_strength=2.2,
    )
    mat_lens = make_mat(
        "Lens",
        (1.0, 0.35, 0.40, 1.0),
        roughness=0.2,
        emission=(1.0, 0.25, 0.30),
        emission_strength=4.0,
    )
    mat_power = make_mat("Power", (0.55, 0.58, 0.62, 1.0), metallic=0.4, roughness=0.4)
    mat_orange = make_mat(
        "Orange", (0.92, 0.40, 0.06, 1.0), metallic=0.08, roughness=0.42
    )

    body = mesh_cube("Body", Vector((0.72, 0.55, 0.55)), Vector((0.0, -0.05, 0.05)))
    apply_mat(body, mat_body)
    apply_transforms(body)

    # 侧散热槽
    for side in (-1.0, 1.0):
        for i in range(4):
            z = -0.12 + i * 0.08
            boolean_diff(
                body,
                mesh_cube(
                    f"Groove_{side}_{i}",
                    Vector((0.05, 0.28, 0.03)),
                    Vector((side * 0.36, -0.08, z)),
                ),
            )
    apply_mat(body, mat_body)

    barrel = mesh_cylinder("Barrel", 0.14, 0.42, Vector((0.0, 0.22, 0.05)), verts=28)
    apply_mat(barrel, mat_laser)

    tip = mesh_cylinder("Tip", 0.10, 0.08, Vector((0.0, 0.46, 0.05)), verts=24)
    apply_mat(tip, mat_lens)

    ring = mesh_cylinder("Ring", 0.17, 0.04, Vector((0.0, 0.02, 0.05)), verts=28)
    apply_mat(ring, mat_orange)

    plate = mesh_cube("Power", Vector((0.40, 0.40, 0.05)), Vector((0.0, -0.05, 0.36)))
    apply_mat(plate, mat_power)


def boolean_intersect(target: bpy.types.Object, other: bpy.types.Object) -> None:
    apply_transforms(other)
    set_active(target)
    mod = target.modifiers.new("Bool", "BOOLEAN")
    mod.operation = "INTERSECT"
    mod.solver = "EXACT"
    mod.object = other
    bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.data.objects.remove(other, do_unlink=True)


def expand_corners(corners: list[Vector], scale: float) -> list[Vector]:
    """从原点放大，保持平面与法线方向。"""
    return [c * scale for c in corners]


def glass_clipped_to_cell(
    name: str, corners: list[Vector], mat: bpy.types.Material, *, scale: float = 2.2
) -> bpy.types.Object:
    """加厚镜片放大后与 1×1×1 格立方体求交，只保留格内部分。"""
    glass = thick_face(name, expand_corners(corners, scale))
    apply_mat(glass, mat)
    apply_transforms(glass)
    boolean_intersect(
        glass, mesh_cube("CellClip", Vector((1.0, 1.0, 1.0)), Vector((0, 0, 0)))
    )
    apply_mat(glass, mat)
    apply_transforms(glass)
    return glass


# ——— 镜子 / 垂直镜 / 分光镜 ———


def build_mirror() -> None:
    """平面镜：无边框；放大后与格立方体求交。"""
    mat_glass = make_mat(
        "Glass",
        (0.45, 0.88, 1.0, 1.0),
        metallic=0.55,
        roughness=0.18,
        emission=(0.10, 0.22, 0.30),
        emission_strength=0.6,
        alpha=0.72,
    )
    game = [
        (-0.5, -0.5, 0.5),
        (0.5, -0.5, -0.5),
        (0.5, 0.5, -0.5),
        (-0.5, 0.5, 0.5),
    ]
    glass_clipped_to_cell("Glass", [game_to_blender(*p) for p in game], mat_glass)


def build_vertical_mirror() -> None:
    """垂直镜：无边框；放大后与格立方体求交。"""
    mat_glass = make_mat(
        "Glass",
        (0.45, 0.88, 1.0, 1.0),
        metallic=0.55,
        roughness=0.18,
        emission=(0.10, 0.22, 0.30),
        emission_strength=0.6,
        alpha=0.72,
    )
    game = [
        (-0.5, -0.5, -0.5),
        (-0.5, -0.5, 0.5),
        (0.5, 0.5, 0.5),
        (0.5, 0.5, -0.5),
    ]
    glass_clipped_to_cell("Glass", [game_to_blender(*p) for p in game], mat_glass)


def build_splitter() -> None:
    """分光镜：无边框；放大后与格立方体求交。"""
    mat_glass = make_mat(
        "Glass",
        (0.55, 0.92, 1.0, 1.0),
        metallic=0.5,
        roughness=0.2,
        emission=(0.12, 0.28, 0.35),
        emission_strength=0.7,
        alpha=0.68,
    )
    raw = [
        (0.5, -0.5, 0.0),
        (0.5, 0.0, -0.5),
        (0.0, 0.5, -0.5),
        (-0.5, 0.5, 0.0),
        (-0.5, 0.0, 0.5),
        (0.0, -0.5, 0.5),
    ]
    yawed = [(-x, y, -z) for x, y, z in raw]
    glass_clipped_to_cell("Glass", [game_to_blender(*p) for p in yawed], mat_glass)


# ——— 吸盘 ———


def build_suction_cup() -> None:
    """开口四棱锥 + 前唇圈/吸垫错开高度，避免正面共面 z-fighting。"""
    mat_cup = make_mat("Cup", (0.82, 0.84, 0.82, 1.0), metallic=0.05, roughness=0.65)
    mat_lip = make_mat("Lip", (0.55, 0.58, 0.56, 1.0), metallic=0.15, roughness=0.55)
    mat_pad = make_mat("Pad", (0.92, 0.40, 0.06, 1.0), metallic=0.05, roughness=0.5)

    # 底口略收回，正面不封死，避免和唇圈共面
    base = [
        Vector((-0.48, 0.42, -0.48)),
        Vector((0.48, 0.42, -0.48)),
        Vector((0.48, 0.42, 0.48)),
        Vector((-0.48, 0.42, 0.48)),
    ]
    apex = Vector((0, 0, 0))
    positions = list(base) + [apex]
    faces = [
        [0, 1, 4],
        [1, 2, 4],
        [2, 3, 4],
        [3, 0, 4],
    ]
    cup = mesh_from_faces("Cup", positions, faces)
    apply_mat(cup, mat_cup)
    apply_transforms(cup)

    # 唇圈：外框挖孔，顶面贴齐格边
    lip = mesh_cube("Lip", Vector((0.98, 0.08, 0.98)), Vector((0.0, 0.46, 0.0)))
    apply_mat(lip, mat_lip)
    apply_transforms(lip)
    boolean_diff(
        lip,
        mesh_cube("LipCut", Vector((0.72, 0.12, 0.72)), Vector((0.0, 0.46, 0.0))),
    )
    apply_mat(lip, mat_lip)

    # 吸垫沉在唇圈开口里，低于唇圈顶
    pad = mesh_cylinder("Pad", 0.22, 0.035, Vector((0.0, 0.44, 0.0)), verts=24)
    apply_mat(pad, mat_pad)


def run_one(label: str, out_dir: Path, builder) -> None:
    clear_scene()
    out_dir.mkdir(parents=True, exist_ok=True)
    print(f"building {label}…", file=sys.stderr)
    builder()
    join_by_material()
    export_glb(out_dir / "model.glb")


def main() -> None:
    run_one("laser", OUT_ROOT / "laser", build_laser)
    run_one("mirror", OUT_ROOT / "mirror", build_mirror)
    run_one("vertical_mirror", OUT_ROOT / "vertical_mirror", build_vertical_mirror)
    run_one("splitter", OUT_ROOT / "splitter", build_splitter)
    run_one("suction_cup", OUT_ROOT / "suction_cup", build_suction_cup)


if __name__ == "__main__":
    main()
