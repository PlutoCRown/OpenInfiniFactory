"""用 Blender 生成选中框 (Selection Box) 外观 GLB。

半透明青色 1×1×1：六面同一张贴图（淡青填充 + 亮青边线/角块），
边线贴齐外沿；Alpha Blend + 背面剔除。

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/models/factory/generate_selection_box_glb.py
"""

from __future__ import annotations

from pathlib import Path
import sys
_TOOLS = Path(__file__).resolve().parents[2]
if str(_TOOLS) not in sys.path:
    sys.path.insert(0, str(_TOOLS))
from common.paths import REPO_ROOT
from common.bpy_util import apply_transforms, clear_scene, export_glb, link, set_active
from common.png_util import write_png_rgba

import struct
import zlib

import bpy
import bmesh
from mathutils import Vector

OUT_DIR = REPO_ROOT / "assets" / "factory_blocks" / "selection_box"
OUT_GLB = OUT_DIR / "model.glb"
OUT_FACE = OUT_DIR / "face_albedo.png"

TEX_SIZE = 256

# 参考图取样：亮边 ~ (180, 240, 225)，面略淡
EDGE_RGB = (180, 240, 225)
CORNER_RGB = (195, 248, 235)
FILL_RGBA = (150, 210, 205, 58)  # 半透明面填充


def generate_face_texture(path: Path, size: int = TEX_SIZE) -> Path:
    """淡青半透明填充 + 贴齐外沿的亮青边线与角块。"""
    rgba = [0] * (size * size * 4)

    def set_px(x: int, y: int, color: tuple[int, int, int, int]) -> None:
        if 0 <= x < size and 0 <= y < size:
            i = (y * size + x) * 4
            rgba[i : i + 4] = list(color)

    def fill_rect(
        x0: float, y0: float, x1: float, y1: float, color: tuple[int, int, int, int]
    ) -> None:
        xa, xb = int(min(x0, x1)), int(max(x0, x1)) + 1
        ya, yb = int(min(y0, y1)), int(max(y0, y1)) + 1
        for y in range(ya, yb):
            for x in range(xa, xb):
                set_px(x, y, color)

    # 整面半透明青填充
    fill_rect(0, 0, size - 1, size - 1, FILL_RGBA)

    # 细边线贴齐外沿
    line_w = max(2, int(size * 0.016))
    edge = (*EDGE_RGB, 255)
    fill_rect(0, 0, size - 1, line_w - 1, edge)
    fill_rect(0, size - line_w, size - 1, size - 1, edge)
    fill_rect(0, 0, line_w - 1, size - 1, edge)
    fill_rect(size - line_w, 0, size - 1, size - 1, edge)

    # 角块：实心小方块贴四角（对应参考图顶点亮块）
    corner_s = max(6, int(size * 0.07))
    corner = (*CORNER_RGB, 255)
    for ox, oy in (
        (0, 0),
        (size - corner_s, 0),
        (0, size - corner_s),
        (size - corner_s, size - corner_s),
    ):
        fill_rect(ox, oy, ox + corner_s - 1, oy + corner_s - 1, corner)

    write_png_rgba(path, size, size, rgba)
    return path


def make_face_mat(img: bpy.types.Image) -> bpy.types.Material:
    """半透明青贴图：自发光 + 只渲染正面。"""
    mat = bpy.data.materials.new("SelectionBoxFace")
    mat.use_nodes = True
    mat.blend_method = "BLEND"
    mat.use_backface_culling = True
    if hasattr(mat, "show_transparent_back"):
        mat.show_transparent_back = False
    if hasattr(mat, "surface_render_method"):
        mat.surface_render_method = "BLENDED"

    nt = mat.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = img
    tex.interpolation = "Closest"
    bsdf.inputs["Metallic"].default_value = 0.0
    bsdf.inputs["Roughness"].default_value = 1.0
    if "Specular IOR Level" in bsdf.inputs:
        bsdf.inputs["Specular IOR Level"].default_value = 0.0
    elif "Specular" in bsdf.inputs:
        bsdf.inputs["Specular"].default_value = 0.0
    nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    if "Alpha" in bsdf.inputs:
        nt.links.new(tex.outputs["Alpha"], bsdf.inputs["Alpha"])
    # 轻微自发光，贴近参考图的亮青边
    if "Emission Color" in bsdf.inputs:
        nt.links.new(tex.outputs["Color"], bsdf.inputs["Emission Color"])
        bsdf.inputs["Emission Strength"].default_value = 0.85
    elif "Emission" in bsdf.inputs:
        nt.links.new(tex.outputs["Color"], bsdf.inputs["Emission"])
    nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    return mat


def build_cube(mat: bpy.types.Material) -> bpy.types.Object:
    """居中 1×1×1；每面 UV 0..1。"""
    mesh = bpy.data.meshes.new("SelectionBox")
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    uv = bm.loops.layers.uv.verify()
    for face in bm.faces:
        n = face.normal
        for loop in face.loops:
            p = loop.vert.co
            if abs(n.z) > 0.7:
                u, v = p.x + 0.5, p.y + 0.5
                if n.z < 0:
                    u = 1.0 - u
            elif abs(n.x) > 0.7:
                u, v = p.y + 0.5, p.z + 0.5
                if n.x < 0:
                    u = 1.0 - u
            else:
                u, v = p.x + 0.5, p.z + 0.5
                if n.y > 0:
                    u = 1.0 - u
            loop[uv].uv = (u, v)
    bm.to_mesh(mesh)
    bm.free()

    obj = bpy.data.objects.new("SelectionBox", mesh)
    link(obj)
    obj.location = Vector((0, 0, 0))
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

    print("generating face albedo…", file=sys.stderr)
    face_path = generate_face_texture(OUT_FACE)
    img = bpy.data.images.load(str(face_path))

    print("building selection box…", file=sys.stderr)
    mat = make_face_mat(img)
    build_cube(mat)

    export_glb(OUT_GLB)
    print(f"Wrote {OUT_GLB}", file=sys.stderr)
    print(f"Wrote {OUT_FACE}", file=sys.stderr)


if __name__ == "__main__":
    main()
