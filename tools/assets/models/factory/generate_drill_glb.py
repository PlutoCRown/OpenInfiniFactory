"""用 Blender 生成钻头 (Drill) 外观 GLB。

整体 1×1×2（Blender Z-up；export_yup 后前进方向 → 游戏局部 -Z）：
  - 本体格：中心在原点，Y ∈ [-0.5, +0.5]
  - 钻头格：Y ∈ [+0.5, +1.5]

同一 model.glb 内两个独立节点（方便对 Head 做旋转动画）：
  - Body：蓝尾环 + 深灰散热槽 + 橙框
  - Head：阶梯锥 + 尖齿

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/factory/generate_drill_glb.py
"""

from __future__ import annotations

from pathlib import Path
import sys
_TOOLS = Path(__file__).resolve().parents[2]
if str(_TOOLS) not in sys.path:
    sys.path.insert(0, str(_TOOLS))
from common.paths import REPO_ROOT
from common.bpy_util import apply_mat, apply_transforms, boolean_diff, clear_scene, export_glb, join_objects, link, make_mat, mesh_cube, mesh_cylinder, set_active

import math

import bpy
import bmesh
from mathutils import Euler, Vector

OUT_DIR = REPO_ROOT / "assets" / "factory_blocks" / "drill"
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
