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
from common.bpy_util import (
    apply_mat,
    apply_transforms,
    boolean_diff,
    clear_scene,
    export_factory_glb,
    finish,
    join_objects,
    make_mat,
    mesh_cube,
    mesh_cylinder,
)
from common.power_port_geom import add_textured_block_port, make_power_port_material

import math

import bpy
from mathutils import Euler, Vector

OUT_PUSHER = REPO_ROOT / "assets" / "factory_blocks" / "pusher"
OUT_BLOCKER = REPO_ROOT / "assets" / "factory_blocks" / "blocker"

CELL = 0.5
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
    mat_port: bpy.types.Material,
) -> list[bpy.types.Object]:
    """活塞体：厚度 1-HEAD_T；前橙环属本体；五面贴图供电口。"""
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
    # 供电口独立物体：勿 join 进 Body，否则顶点 AO 会乘到 PBR 贴图上
    add_textured_block_port("PosZ", Vector((0, 0, CELL + pad)), "+Z", mat_port)
    add_textured_block_port("NegZ", Vector((0, 0, -CELL - pad)), "-Z", mat_port)
    add_textured_block_port("PosX", Vector((CELL + pad, 0, 0)), "+X", mat_port)
    add_textured_block_port("NegX", Vector((-CELL - pad, 0, 0)), "-X", mat_port)
    add_textured_block_port("NegY", Vector((0, -CELL - pad, 0)), "-Y", mat_port)
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
    mat_dark = make_mat("Dark", (0.04, 0.04, 0.05, 1.0), metallic=0.40, roughness=0.42)
    mat_rod = make_mat("Rod", (0.78, 0.80, 0.82, 1.0), metallic=0.75, roughness=0.30)
    mat_port = make_power_port_material("block")
    mat_head = mat_orange if orange_head else mat_body
    square_arm = orange_head  # 拦截器：单根粗方臂

    kind = "blocker" if orange_head else "pusher"
    print(f"building {kind} body (offset={offset})…", file=sys.stderr)
    body_parts = build_body(mat_body, mat_orange, mat_port)
    print("building stage (offset/2)…", file=sys.stderr)
    stage_parts = build_stage(offset, mat_body, square_arm=square_arm)
    print("building head + tip…", file=sys.stderr)
    head_parts = build_head(offset, mat_head, mat_dark, mat_rod, square_arm=square_arm)

    join_objects("Body", body_parts)
    join_objects("Stage", stage_parts)
    join_objects("Head", head_parts)

    out = out_dir / filename
    export_factory_glb(out, export_tangents=True)
    print(f"Wrote {out}", file=sys.stderr)


def main() -> None:
    for out_dir, orange_head in VARIANTS:
        out_dir.mkdir(parents=True, exist_ok=True)
        for offset, filename in OFFSET_EXPORTS:
            build_and_export(out_dir, offset, filename, orange_head=orange_head)


if __name__ == "__main__":
    main()
