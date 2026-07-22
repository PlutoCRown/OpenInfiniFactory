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
    --python tools/assets/models/factory/generate_detector_glb.py
"""

from __future__ import annotations

from pathlib import Path
import sys
_TOOLS = Path(__file__).resolve().parents[2]
if str(_TOOLS) not in sys.path:
    sys.path.insert(0, str(_TOOLS))
from common.paths import REPO_ROOT
from common.bpy_util import apply_mat, apply_transforms, boolean_diff, clear_scene, export_glb, join_by_material, link, make_mat, mesh_cube, mesh_cylinder, mesh_torus, set_active

import math

import bpy
import bmesh
from mathutils import Euler, Vector

OUT_DIR = REPO_ROOT / "assets" / "factory_blocks" / "detector"
OUT_GLB = OUT_DIR / "model.glb"

# 1×1×1 格
CELL = 0.5
# 机体：横截面 1×1，工作轴厚 0.9；贴齐背面，工作面凹进
BODY_XZ = 1.0
BODY_Y = 0.9
BODY_Y_MIN = -CELL  # -0.5 贴齐背面
BODY_Y_MAX = BODY_Y_MIN + BODY_Y  # +0.4 工作面
BODY_Y_CENTER = (BODY_Y_MIN + BODY_Y_MAX) * 0.5  # -0.05


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
        emission=(0.55, 0.02, 0.03),
        emission_strength=1.4,
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
