"""用 Blender 生成选中框 (Selection Box) 外观 GLB。

半透明青色 1×1×1：六面同一张贴图（淡青填充 + 亮青边线/角块），
边线贴齐外沿；Alpha Blend + 背面剔除。

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python scripts/factory_blocks/generate_selection_box_glb.py
"""

from __future__ import annotations

import struct
import sys
import zlib
from pathlib import Path

import bpy
import bmesh
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[2]
OUT_DIR = ROOT / "assets" / "factory_blocks" / "selection_box"
OUT_GLB = OUT_DIR / "model.glb"
OUT_FACE = OUT_DIR / "face_albedo.png"

TEX_SIZE = 256

# 参考图取样：亮边 ~ (180, 240, 225)，面略淡
EDGE_RGB = (180, 240, 225)
CORNER_RGB = (195, 248, 235)
FILL_RGBA = (150, 210, 205, 58)  # 半透明面填充


def clear_scene() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    for coll in (
        bpy.data.meshes,
        bpy.data.materials,
        bpy.data.images,
        bpy.data.cameras,
        bpy.data.lights,
    ):
        for block in list(coll):
            coll.remove(block)


def link(obj: bpy.types.Object) -> bpy.types.Object:
    bpy.context.collection.objects.link(obj)
    return obj


def set_active(obj: bpy.types.Object) -> None:
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj


def apply_transforms(obj: bpy.types.Object) -> None:
    set_active(obj)
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)


def write_png_rgba(path: Path, width: int, height: int, rgba: list[int]) -> None:
    """不依赖 PIL 的最小 PNG 写出。"""

    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    raw = bytearray()
    for y in range(height):
        raw.append(0)
        i = y * width * 4
        raw.extend(rgba[i : i + width * 4])
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n")
        f.write(chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)))
        f.write(chunk(b"IDAT", zlib.compress(bytes(raw), 9)))
        f.write(chunk(b"IEND", b""))


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


def export_glb(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    for obj in bpy.context.scene.objects:
        if obj.type == "MESH":
            obj.select_set(True)
    bpy.ops.export_scene.gltf(
        filepath=str(path),
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_texcoords=True,
        export_normals=True,
        export_materials="EXPORT",
        export_yup=True,
        export_image_format="AUTO",
    )


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
