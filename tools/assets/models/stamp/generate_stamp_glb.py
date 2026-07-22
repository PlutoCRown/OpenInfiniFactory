"""用 Blender 为 stamp_materials/<id>/ 生成薄板 model.glb。

约定（与告示牌 / 运行时 facing.yaw 一致）：
  - 游戏局部 +Z 朝宿主，板心 z=+0.45
  - 尺寸 0.78×0.72×0.1 → 贴宿主面外凸 0.1
  - 贴图用同目录 texture.png（嵌入 GLB）
  - Blender：厚沿 -Y；export_yup 后 → 游戏 +Z

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/stamp/generate_stamp_glb.py
"""

from __future__ import annotations

from pathlib import Path
import sys

_TOOLS = Path(__file__).resolve().parents[2]
if str(_TOOLS) not in sys.path:
    sys.path.insert(0, str(_TOOLS))
from common.paths import STAMP_MATERIALS as ROOT
from common.bpy_util import (
    apply_transforms,
    clear_scene,
    export_glb,
    link,
    make_mat,
    set_active,
)

import bpy
import bmesh
from mathutils import Vector

# 游戏 (x,y,z)=(0.78,0.72,0.1)；Blender 尺寸 (x, z厚度→y, y→z)
SIZE = Vector((0.78, 0.1, 0.72))
# 游戏 z=+0.45 → Blender y=-0.45（export: glTF_z = -Blender_y）
CENTER_Y = -0.45


def build_plate(mat: bpy.types.Material) -> bpy.types.Object:
    """建薄板并烘焙变换。"""
    mesh = bpy.data.meshes.new("Stamp")
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    bmesh.ops.scale(bm, vec=SIZE, verts=bm.verts)
    bm.to_mesh(mesh)
    bm.free()

    obj = bpy.data.objects.new("Stamp", mesh)
    link(obj)
    obj.location = Vector((0.0, CENTER_Y, 0.0))
    obj.data.materials.append(mat)
    apply_transforms(obj)

    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.cube_project(cube_size=1.0)
    bpy.ops.mesh.quads_convert_to_tris(quad_method="BEAUTY", ngon_method="BEAUTY")
    bpy.ops.object.mode_set(mode="OBJECT")
    return obj


def export_one(pack: Path) -> None:
    """清空场景、挂贴图、导出该印花包。"""
    tex_path = pack / "texture.png"
    if not tex_path.is_file():
        print(f"skip (no texture.png): {pack.name}", file=sys.stderr)
        return
    clear_scene()
    img = bpy.data.images.load(str(tex_path))
    img.pack()
    mat = make_mat(
        pack.name,
        (1.0, 1.0, 1.0, 1.0),
        roughness=0.92,
        texture=img,
        backface_culling=False,
    )
    build_plate(mat)
    export_glb(pack / "model.glb")


def main() -> None:
    if not ROOT.is_dir():
        raise SystemExit(f"missing {ROOT}")
    for pack in sorted(p for p in ROOT.iterdir() if p.is_dir()):
        export_one(pack)


if __name__ == "__main__":
    main()
