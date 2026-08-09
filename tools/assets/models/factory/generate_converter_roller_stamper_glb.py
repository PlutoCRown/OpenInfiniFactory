"""用 Blender 生成转换器 / 滚刷 / 印花机外观 GLB。

印花机对齐参考操作台造型（蓝灰机箱 + 橙框面板）。
Blender Z-up；export_yup 后前进方向 → 游戏局部 -Z（Blender +Y）。

产出：
  assets/factory_blocks/converter/model.glb
  assets/factory_blocks/roller/model.glb
  assets/factory_blocks/stamper/model.glb

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/factory/generate_converter_roller_stamper_glb.py
  # 只重建印花机：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/factory/generate_converter_roller_stamper_glb.py -- --only stamper
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
    export_factory_glb,
    join_by_material,
    make_mat,
    mesh_cube,
    mesh_cylinder,
    set_active,
)

import math

import bpy
import bmesh
from mathutils import Euler, Vector

OUT_ROOT = REPO_ROOT / "assets" / "factory_blocks"


def bevel_object(obj: bpy.types.Object, width: float = 0.03, segments: int = 2) -> None:
    """全体外棱轻倒角。"""
    apply_transforms(obj)
    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bm = bmesh.from_edit_mesh(obj.data)
    for e in bm.edges:
        e.select = True
    bmesh.update_edit_mesh(obj.data)
    bpy.ops.mesh.bevel(offset=width, segments=segments, affect="EDGES")
    bpy.ops.object.mode_set(mode="OBJECT")


def finish_cube(
    name: str,
    size: Vector,
    loc: Vector,
    mat: bpy.types.Material,
    *,
    bevel: float = 0.028,
) -> bpy.types.Object:
    obj = mesh_cube(name, size, loc)
    apply_mat(obj, mat)
    bevel_object(obj, width=bevel)
    apply_mat(obj, mat)
    return obj


def export_current(subdir: str) -> None:
    join_by_material()
    out_dir = OUT_ROOT / subdir
    out_dir.mkdir(parents=True, exist_ok=True)
    # 印花机面板贴机头，AO 过强会在接缝处烤出假缝
    if subdir == "stamper":
        export_factory_glb(
            out_dir / "model.glb",
            ao_max_ray_distance=0.12,
            ao_strength=0.22,
        )
    else:
        export_factory_glb(out_dir / "model.glb")


def build_converter() -> None:
    """中部机箱 + 左右进出料口（青入 / 橙出）+ 顶桥。"""
    mat_body = make_mat("Body", (0.16, 0.18, 0.20, 1.0), metallic=0.28, roughness=0.46)
    mat_in = make_mat(
        "In",
        (0.18, 0.62, 1.0, 1.0),
        metallic=0.08,
        roughness=0.35,
        emission=(0.10, 0.35, 0.85),
        emission_strength=1.6,
    )
    mat_out = make_mat(
        "Out",
        (1.0, 0.54, 0.18, 1.0),
        metallic=0.08,
        roughness=0.35,
        emission=(0.85, 0.28, 0.04),
        emission_strength=1.6,
    )
    mat_accent = make_mat(
        "Accent", (0.42, 0.46, 0.50, 1.0), metallic=0.45, roughness=0.38
    )

    finish_cube("Body", Vector((0.78, 0.70, 0.62)), Vector((0.0, -0.02, 0.0)), mat_body)
    finish_cube(
        "InPort",
        Vector((0.22, 0.22, 0.22)),
        Vector((-0.30, 0.0, 0.22)),
        mat_in,
        bevel=0.02,
    )
    finish_cube(
        "OutPort",
        Vector((0.22, 0.22, 0.22)),
        Vector((0.30, 0.0, 0.22)),
        mat_out,
        bevel=0.02,
    )
    finish_cube(
        "Bridge",
        Vector((0.62, 0.14, 0.12)),
        Vector((0.0, 0.0, 0.22)),
        mat_accent,
        bevel=0.018,
    )
    finish_cube(
        "BaseSkirt",
        Vector((0.92, 0.92, 0.10)),
        Vector((0.0, 0.0, -0.36)),
        mat_accent,
        bevel=0.02,
    )


def build_roller() -> None:
    """宽高 1、厚 0.7 蓝灰机箱；正面两侧竖槽内伸出蓝轴，接前方滚筒。

    沿工作轴（Blender +Y）：背面 -0.5 → 机身 0.7 → 前脸 0.2；滚筒在前脸外、不穿模。
    竖槽留上下行程，蓝轴端嵌在槽内向前伸出接圆柱。
    """
    mat_body = make_mat("Body", (0.30, 0.38, 0.46, 1.0), metallic=0.18, roughness=0.52)
    mat_signal = make_mat(
        "Signal",
        (0.20, 0.78, 1.0, 1.0),
        metallic=0.12,
        roughness=0.32,
        emission=(0.05, 0.35, 0.70),
        emission_strength=1.4,
    )
    mat_brush = make_mat(
        "Brush", (0.78, 0.80, 0.72, 1.0), metallic=0.05, roughness=0.72
    )
    mat_metal = make_mat(
        "Metal", (0.55, 0.58, 0.60, 1.0), metallic=0.55, roughness=0.34
    )

    body_depth = 0.7
    body_y_min = -0.5
    body_y_max = body_y_min + body_depth  # 0.2
    body_y = (body_y_min + body_y_max) * 0.5

    # 机身前脸 y=0.2，格前界 y=0.5；滚筒轴心取中点则半径最大
    # r_max = (0.5 - 0.2) / 2 = 0.15；两侧各留 0.008 防贴面
    clear = 0.008
    axle_y = (body_y_max + 0.5) * 0.5  # 0.35
    drum_r = (0.5 - body_y_max) * 0.5 - clear  # 0.142
    drum_len = 0.58

    # 竖槽 / 蓝杆收细：槽略宽于杆，深度刚好埋住杆后端
    groove_w = 0.08
    groove_d = 0.10
    groove_h = 0.68
    groove_x = 0.38
    hub = 0.055
    groove_back = body_y_max - groove_d

    body = mesh_cube(
        "Body",
        Vector((1.0, body_depth, 1.0)),
        Vector((0.0, body_y, 0.0)),
    )
    apply_mat(body, mat_body)
    apply_transforms(body)
    for side in (-1.0, 1.0):
        boolean_diff(
            body,
            mesh_cube(
                f"GrooveCut_{side}",
                Vector((groove_w, groove_d + 0.04, groove_h)),
                Vector(
                    (
                        side * groove_x,
                        body_y_max - groove_d * 0.5 + 0.02,
                        0.0,
                    )
                ),
            ),
        )
    clean_mesh(body)
    apply_mat(body, mat_body)

    # 蓝轴端：后部落在竖槽内，前伸到滚筒轴心，不超出机身左右外轮廓
    hub_rear = groove_back + 0.015
    hub_front = axle_y + hub * 0.3
    hub_depth = hub_front - hub_rear
    hub_y = (hub_rear + hub_front) * 0.5
    for side in (-1.0, 1.0):
        finish_cube(
            f"Hub_{side}",
            Vector((hub, hub_depth, hub)),
            Vector((side * groove_x, hub_y, 0.0)),
            mat_signal,
            bevel=0.008,
        )

    # 横轴：连两侧蓝轴，穿过滚筒
    finish_cube(
        "Axle",
        Vector((2.0 * groove_x - hub * 0.3, 0.038, 0.038)),
        Vector((0.0, axle_y, 0.0)),
        mat_metal,
        bevel=0.006,
    )

    drum = mesh_cylinder(
        "Drum",
        drum_r,
        drum_len,
        Vector((0.0, axle_y, 0.0)),
        rot=Euler((0.0, math.pi / 2.0, 0.0)),
        verts=28,
    )
    apply_mat(drum, mat_brush)
    apply_transforms(drum)


def clean_mesh(obj: bpy.types.Object) -> None:
    """合并重叠点、去掉退化面、统一法线（布尔后消 Z 闪）。"""
    apply_transforms(obj)
    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.mesh.remove_doubles(threshold=1e-4)
    bpy.ops.mesh.dissolve_degenerate(threshold=1e-4)
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.object.mode_set(mode="OBJECT")


def build_stamper() -> None:
    """印花机：机身厚 0.8（正方体 ∪ 八棱柱，布尔合并）+ 橙面板厚 0.1 + 正面空隙 0.1。

    沿工作轴（Blender +Y）：背面 -0.5 → 机身 0.8 → 面板 0.1 → 空隙 0.1 → 格前 0.5。
    橙色 = 底板 + 前框（浅凹槽），黑面 StampFace 略凹；暂不做倒角。
    """
    mat_body = make_mat("Body", (0.62, 0.68, 0.74, 1.0), metallic=0.16, roughness=0.48)
    mat_frame = make_mat(
        "Frame", (0.95, 0.42, 0.08, 1.0), metallic=0.12, roughness=0.38
    )
    mat_stamp = make_mat("StampFace", (0.0, 0.0, 0.0, 1.0), metallic=0.0, roughness=0.9)

    core = 0.8
    oct_r = 0.1
    body_depth = 0.8
    panel_t = 0.1
    front_gap = 0.1
    recess = 0.018  # 黑面相对橙框正面略凹
    stamp_t = 0.012
    inner = 0.86

    body_y_min = -0.5
    body_y_max = body_y_min + body_depth  # 0.3
    # 机身前端略嵌入橙板背面，避免共面 Z 闪
    body_embed = 0.004
    body_y_max += body_embed
    body_depth_eff = body_y_max - body_y_min
    body_y = (body_y_min + body_y_max) * 0.5
    panel_y_min = body_y_min + body_depth  # 仍贴在名义 0.3
    panel_y_max = panel_y_min + panel_t  # 0.4
    assert abs(panel_y_max + front_gap - 0.5) < 1e-6
    panel_y = (panel_y_min + panel_y_max) * 0.5

    body = mesh_cube(
        "BodyCore",
        Vector((core, body_depth_eff, core)),
        Vector((0.0, body_y, 0.0)),
    )
    apply_mat(body, mat_body)
    apply_transforms(body)

    for sx in (-1.0, 1.0):
        for sz in (-1.0, 1.0):
            octagon = mesh_cylinder(
                f"Corner_{sx}_{sz}",
                oct_r,
                body_depth_eff,
                Vector((sx * core * 0.5, body_y, sz * core * 0.5)),
                rot=Euler((math.pi / 2.0, math.pi / 8.0, 0.0)),
                verts=8,
            )
            apply_mat(octagon, mat_body)
            apply_transforms(octagon)
            boolean_union(body, octagon)

    body.name = "Body"
    if body.data:
        body.data.name = "Mesh_Body"
    clean_mesh(body)
    apply_mat(body, mat_body)

    # 橙色整板 + 正面浅槽（保留底板，侧面/后面看不到黑面）
    orange = mesh_cube(
        "OrangePanel",
        Vector((1.0, panel_t, 1.0)),
        Vector((0.0, panel_y, 0.0)),
    )
    apply_mat(orange, mat_frame)
    apply_transforms(orange)
    pocket_depth = recess + 0.004
    pocket_y = panel_y_max - pocket_depth * 0.5
    boolean_diff(
        orange,
        mesh_cube(
            "FrontPocket",
            Vector((inner, pocket_depth + 0.02, inner)),
            Vector((0.0, pocket_y + 0.01, 0.0)),
        ),
    )
    clean_mesh(orange)
    apply_mat(orange, mat_frame)

    # 黑面贴在凹槽底，正面仍略低于橙框
    stamp_front = panel_y_max - recess
    stamp_y = stamp_front - stamp_t * 0.5
    stamp = mesh_cube(
        "StampFace",
        Vector((inner - 0.02, stamp_t, inner - 0.02)),
        Vector((0.0, stamp_y, 0.0)),
    )
    apply_mat(stamp, mat_stamp)
    apply_transforms(stamp)


def main() -> None:
    import sys as _sys

    only = None
    if "--only" in _sys.argv:
        i = _sys.argv.index("--only")
        if i + 1 < len(_sys.argv):
            only = _sys.argv[i + 1]

    targets = [
        ("converter", build_converter),
        ("roller", build_roller),
        ("stamper", build_stamper),
    ]
    for name, build in targets:
        if only is not None and only != name:
            continue
        clear_scene()
        build()
        export_current(name)

    print("wrote", only or "converter / roller / stamper", "model.glb")


if __name__ == "__main__":
    main()
