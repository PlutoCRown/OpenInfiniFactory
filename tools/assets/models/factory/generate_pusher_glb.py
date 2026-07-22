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
    --python tools/assets/models/factory/generate_pusher_glb.py
"""

from __future__ import annotations

from pathlib import Path
import sys
_TOOLS = Path(__file__).resolve().parents[2]
if str(_TOOLS) not in sys.path:
    sys.path.insert(0, str(_TOOLS))
from common.paths import REPO_ROOT
from common.bpy_util import apply_mat, apply_transforms, boolean_diff, clear_scene, export_glb, finish, join_objects, link, make_mat, mesh_cube, mesh_cylinder, mesh_torus, set_active

import math

import bpy
import bmesh
from mathutils import Euler, Vector

OUT_PUSHER = REPO_ROOT / "assets" / "factory_blocks" / "pusher"
OUT_BLOCKER = REPO_ROOT / "assets" / "factory_blocks" / "blocker"

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
