"""用 Blender 生成灯面板 (LightPanel) 外观 GLB。

约定（与 light_panel_transform 一致）：
  - 局部 +Y 朝外（附着法线）
  - 尺寸 1×1×0.1，板心 y=+0.45 → 外表面齐格面（不外凸）
  - 模板沿 Blender +Z 建模；export_yup：Blender +Z → 游戏 +Y
  - 运行时只取 mesh，通电/未通电材质由引擎切换

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/factory/generate_light_panel_glb.py
"""

from __future__ import annotations

from pathlib import Path
import sys
_TOOLS = Path(__file__).resolve().parents[2]
if str(_TOOLS) not in sys.path:
    sys.path.insert(0, str(_TOOLS))
from common.paths import REPO_ROOT
from common.bpy_util import apply_transforms, clear_scene, export_glb, link, make_mat, set_active

import bpy
import bmesh
from mathutils import Vector

OUT_DIR = REPO_ROOT / "assets" / "factory_blocks" / "light_panel"
OUT_GLB = OUT_DIR / "model.glb"

# Blender：厚沿 +Z，板心 z=0.45；export 后游戏厚沿 +Y
SIZE = Vector((1.0, 1.0, 0.1))
CENTER_Z = 0.45


def build_panel(mat: bpy.types.Material) -> bpy.types.Object:
    mesh = bpy.data.meshes.new("LightPanel")
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    bmesh.ops.scale(bm, vec=SIZE, verts=bm.verts)
    bm.to_mesh(mesh)
    bm.free()

    obj = bpy.data.objects.new("LightPanel", mesh)
    link(obj)
    obj.location = Vector((0.0, 0.0, CENTER_Z))
    obj.data.materials.append(mat)
    apply_transforms(obj)

    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.mesh.quads_convert_to_tris(quad_method="BEAUTY", ngon_method="BEAUTY")
    bpy.ops.object.mode_set(mode="OBJECT")
    return obj


def main() -> None:
    clear_scene()
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    mat = make_mat("LightPanel", (0.0, 0.0, 0.0, 1.0), roughness=1.0)
    build_panel(mat)
    export_glb(OUT_GLB)
    print(f"Wrote {OUT_GLB}", file=sys.stderr)


if __name__ == "__main__":
    main()
