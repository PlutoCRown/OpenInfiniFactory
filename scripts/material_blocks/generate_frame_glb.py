"""用 Blender 生成材料方块粗立方体框架 GLB。

1×1×1 居中；六面镂空，直角无倒角。
凹槽效果用法线贴图（共享 frame_normal.png），不用几何倒角。

颜色作变量，一次导出两种预览：
  assets/material_blocks/preview_red/model.glb
  assets/material_blocks/preview_blue/model.glb

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python scripts/material_blocks/generate_frame_glb.py
"""

from __future__ import annotations

import math
import struct
import sys
import zlib
from pathlib import Path

import bpy
import bmesh
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[2]
OUT_ROOT = ROOT / "assets" / "material_blocks"
OUT_NORMAL = OUT_ROOT / "frame_normal.png"

# 棱框厚度（单侧）；开孔 = 1 - 2 * FRAME_T
FRAME_T = 0.22
TEX_SIZE = 256

# (子目录名, 颜色 RGBA, metallic, roughness)
VARIANTS: list[tuple[str, tuple[float, float, float, float], float, float]] = [
    ("preview_red", (0.72, 0.36, 0.20, 1.0), 0.75, 0.38),
    ("preview_blue", (0.42, 0.55, 0.72, 1.0), 0.55, 0.42),
]


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


def groove_profile(dist: float, center: float, half_w: float) -> float:
    """距 center 越近越高（凹槽深度），返回 0..1。"""
    t = abs(dist - center) / half_w
    if t >= 1.0:
        return 0.0
    return (1.0 - t * t) ** 2


def generate_groove_normal(path: Path, size: int = TEX_SIZE) -> Path:
    """立方体面 UV：简单回字凹槽（外沿一圈 + 内开孔一圈），槽较窄。"""
    height = [[0.5] * size for _ in range(size)]
    # 窄槽：中心靠近边，半宽更小
    out_c, in_c = 0.035, 0.035
    half_w = 0.014
    depth = 0.50

    for y in range(size):
        for x in range(size):
            u = (x + 0.5) / size
            v = (y + 0.5) / size
            d_out = min(u, 1.0 - u, v, 1.0 - v)
            d_in = min(
                abs(u - FRAME_T),
                abs(u - (1.0 - FRAME_T)),
                abs(v - FRAME_T),
                abs(v - (1.0 - FRAME_T)),
            )
            g = groove_profile(d_out, out_c, half_w)
            if d_out < FRAME_T + 0.01:
                g = max(g, groove_profile(d_in, in_c, half_w))
            height[y][x] = 0.5 - depth * g

    normal = [0] * (size * size * 4)
    strength = 7.0
    for y in range(size):
        for x in range(size):
            h_l = height[y][(x - 1) % size]
            h_r = height[y][(x + 1) % size]
            h_d = height[(y - 1) % size][x]
            h_u = height[(y + 1) % size][x]
            dx = (h_r - h_l) * strength
            dy = (h_u - h_d) * strength
            nx, ny, nz = -dx, -dy, 1.0
            inv = 1.0 / math.sqrt(nx * nx + ny * ny + nz * nz)
            nx, ny, nz = nx * inv, ny * inv, nz * inv
            i = (y * size + x) * 4
            normal[i] = int((nx * 0.5 + 0.5) * 255)
            normal[i + 1] = int((ny * 0.5 + 0.5) * 255)
            normal[i + 2] = int((nz * 0.5 + 0.5) * 255)
            normal[i + 3] = 255

    write_png_rgba(path, size, size, normal)
    return path


def make_mat(
    name: str,
    color: tuple[float, float, float, float],
    *,
    metallic: float,
    roughness: float,
    normal_img: bpy.types.Image,
) -> bpy.types.Material:
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    nt = mat.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
    bsdf.inputs["Base Color"].default_value = color
    bsdf.inputs["Metallic"].default_value = metallic
    bsdf.inputs["Roughness"].default_value = roughness

    tex_n = nt.nodes.new("ShaderNodeTexImage")
    tex_n.image = normal_img
    tex_n.interpolation = "Smart"
    nmap = nt.nodes.new("ShaderNodeNormalMap")
    nmap.inputs["Strength"].default_value = 1.15
    nt.links.new(tex_n.outputs["Color"], nmap.inputs["Color"])
    nt.links.new(nmap.outputs["Normal"], bsdf.inputs["Normal"])
    nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    return mat


def mesh_cube(name: str, size: Vector, loc: Vector) -> bpy.types.Object:
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    bmesh.ops.scale(bm, vec=size, verts=bm.verts)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    link(obj)
    obj.location = loc
    return obj


def boolean_diff(target: bpy.types.Object, cutter: bpy.types.Object) -> None:
    apply_transforms(cutter)
    set_active(target)
    mod = target.modifiers.new("Bool", "BOOLEAN")
    mod.operation = "DIFFERENCE"
    mod.solver = "EXACT"
    mod.object = cutter
    bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.data.objects.remove(cutter, do_unlink=True)


def assign_box_uv(obj: bpy.types.Object) -> None:
    """按面法线立方投影到 0..1，对齐凹槽法线贴图。"""
    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bm = bmesh.from_edit_mesh(obj.data)
    uv = bm.loops.layers.uv.verify()
    for face in bm.faces:
        n = face.normal
        ax = abs(n.x)
        ay = abs(n.y)
        az = abs(n.z)
        for loop in face.loops:
            p = loop.vert.co
            if az >= ax and az >= ay:
                u, v = p.x + 0.5, p.y + 0.5
            elif ax >= ay:
                u, v = p.y + 0.5, p.z + 0.5
            else:
                u, v = p.x + 0.5, p.z + 0.5
            loop[uv].uv = (u, v)
    bmesh.update_edit_mesh(obj.data)
    bpy.ops.object.mode_set(mode="OBJECT")


def build_frame(mat: bpy.types.Material) -> bpy.types.Object:
    """实心立方减去三轴通孔 → 粗框架（直角，无倒角）。"""
    body = mesh_cube("Frame", Vector((1.0, 1.0, 1.0)), Vector((0, 0, 0)))
    body.data.materials.append(mat)
    apply_transforms(body)

    hole = 1.0 - 2.0 * FRAME_T
    boolean_diff(
        body,
        mesh_cube("CutX", Vector((1.12, hole, hole)), Vector((0, 0, 0))),
    )
    boolean_diff(
        body,
        mesh_cube("CutY", Vector((hole, 1.12, hole)), Vector((0, 0, 0))),
    )
    boolean_diff(
        body,
        mesh_cube("CutZ", Vector((hole, hole, 1.12)), Vector((0, 0, 0))),
    )
    assign_box_uv(body)
    set_active(body)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.mesh.quads_convert_to_tris(quad_method="BEAUTY", ngon_method="BEAUTY")
    bpy.ops.object.mode_set(mode="OBJECT")
    return body


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
        export_tangents=True,
        export_materials="EXPORT",
        export_yup=True,
        export_image_format="AUTO",
    )


def build_and_export(
    subdir: str,
    color: tuple[float, float, float, float],
    metallic: float,
    roughness: float,
) -> None:
    clear_scene()
    normal_img = bpy.data.images.load(str(OUT_NORMAL))
    normal_img.colorspace_settings.name = "Non-Color"
    mat = make_mat(
        f"Mat_{subdir}",
        color,
        metallic=metallic,
        roughness=roughness,
        normal_img=normal_img,
    )
    print(f"building frame {subdir}…", file=sys.stderr)
    build_frame(mat)
    out = OUT_ROOT / subdir / "model.glb"
    export_glb(out)
    print(f"Wrote {out}", file=sys.stderr)


def main() -> None:
    OUT_ROOT.mkdir(parents=True, exist_ok=True)
    print("generating groove normal…", file=sys.stderr)
    generate_groove_normal(OUT_NORMAL)
    print(f"Wrote {OUT_NORMAL}", file=sys.stderr)

    for subdir, color, metallic, roughness in VARIANTS:
        build_and_export(subdir, color, metallic, roughness)


if __name__ == "__main__":
    main()
