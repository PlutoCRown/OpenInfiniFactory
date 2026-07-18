"""用 Blender 生成平台方块 (Platform) 外观 GLB。

标准居中 1×1×1；侧面上/下各一条环绕凹槽；顶面可无缝 albedo + normal（相邻块 UV 0..1 能对上）。

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python scripts/factory_blocks/generate_platform_glb.py
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
OUT_DIR = ROOT / "assets" / "factory_blocks" / "platform"
OUT_GLB = OUT_DIR / "model.glb"
OUT_ALBEDO = OUT_DIR / "top_albedo.png"
OUT_NORMAL = OUT_DIR / "top_normal.png"
OUT_HEIGHT = OUT_DIR / "top_height.png"

CELL = 0.5
GROOVE_Z = 0.38  # 距中心，约侧面顶/底 1/10
GROOVE_H = 0.035
GROOVE_DEPTH = 0.03


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


def hash2(ix: int, iy: int) -> float:
    n = ix * 374761393 + iy * 668265263
    n = (n ^ (n >> 13)) * 1274126177
    return ((n ^ (n >> 16)) & 0xFFFFFF) / float(0xFFFFFF)


def value_noise(x: float, y: float) -> float:
    x0, y0 = math.floor(x), math.floor(y)
    fx, fy = x - x0, y - y0
    fx = fx * fx * (3 - 2 * fx)
    fy = fy * fy * (3 - 2 * fy)
    v00 = hash2(x0, y0)
    v10 = hash2(x0 + 1, y0)
    v01 = hash2(x0, y0 + 1)
    v11 = hash2(x0 + 1, y0 + 1)
    return (
        v00 * (1 - fx) * (1 - fy)
        + v10 * fx * (1 - fy)
        + v01 * (1 - fx) * fy
        + v11 * fx * fy
    )


def fbm(x: float, y: float, octaves: int = 4) -> float:
    amp, freq, total, norm = 1.0, 1.0, 0.0, 0.0
    for _ in range(octaves):
        total += value_noise(x * freq, y * freq) * amp
        norm += amp
        amp *= 0.5
        freq *= 2.0
    return total / norm


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


def generate_top_maps(size: int = 256) -> tuple[Path, Path]:
    """可无缝顶面：细微板块 + 噪波；并导出对应法线。"""
    height = [[0.0] * size for _ in range(size)]
    albedo = [0] * (size * size * 4)
    twopi = math.pi * 2.0

    def seamless_n(u: float, v: float, scale: float) -> float:
        # 把 [0,1)² 嵌到圆环，保证对边连续
        return fbm(
            (math.cos(twopi * u) + math.cos(twopi * v)) * scale,
            (math.sin(twopi * u) + math.sin(twopi * v)) * scale,
        )

    base = (0.42, 0.50, 0.56)
    for y in range(size):
        for x in range(size):
            u = x / size
            v = y / size
            n = seamless_n(u, v, 2.2)
            n2 = seamless_n(u + 0.17, v + 0.31, 4.5)
            # 无缝板块线
            line = max(
                0.0,
                1.0 - abs(math.sin(u * math.pi * 4)) * 16.0,
                1.0 - abs(math.sin(v * math.pi * 4)) * 16.0,
            )
            line = max(0.0, min(1.0, line))

            h = 0.55 + 0.12 * (n - 0.5) + 0.06 * (n2 - 0.5) - 0.20 * line
            h = max(0.0, min(1.0, h))
            height[y][x] = h

            # 仅贴边一圈略压暗；中间保持原亮度
            edge = min(u, 1.0 - u, v, 1.0 - v)
            # edge∈[0,0.5]；只在 <0.08 的窄边带生效
            if edge < 0.08:
                t = 1.0 - edge / 0.08
                rim = t * t
            else:
                rim = 0.0
            shade = 0.92 + 0.10 * h - 0.10 * line - 0.14 * rim
            r = int(max(0, min(255, base[0] * shade * 255)))
            g = int(max(0, min(255, base[1] * shade * 255)))
            b = int(max(0, min(255, base[2] * shade * 255)))
            i = (y * size + x) * 4
            albedo[i : i + 4] = [r, g, b, 255]

    # 高度 → 法线（OpenGL，+Y 向上），无缝差分
    normal = [0] * (size * size * 4)
    strength = 4.5
    for y in range(size):
        for x in range(size):
            h_l = height[y][(x - 1) % size]
            h_r = height[y][(x + 1) % size]
            h_d = height[(y - 1) % size][x]
            h_u = height[(y + 1) % size][x]
            dx = (h_r - h_l) * strength
            dy = (h_u - h_d) * strength
            # 法线
            nx, ny, nz = -dx, -dy, 1.0
            inv = 1.0 / math.sqrt(nx * nx + ny * ny + nz * nz)
            nx, ny, nz = nx * inv, ny * inv, nz * inv
            i = (y * size + x) * 4
            normal[i] = int((nx * 0.5 + 0.5) * 255)
            normal[i + 1] = int((ny * 0.5 + 0.5) * 255)
            normal[i + 2] = int((nz * 0.5 + 0.5) * 255)
            normal[i + 3] = 255

    write_png_rgba(OUT_ALBEDO, size, size, albedo)
    write_png_rgba(OUT_NORMAL, size, size, normal)

    # 可选高度图备查
    height_rgba = [0] * (size * size * 4)
    for y in range(size):
        for x in range(size):
            v = int(height[y][x] * 255)
            i = (y * size + x) * 4
            height_rgba[i : i + 4] = [v, v, v, 255]
    write_png_rgba(OUT_HEIGHT, size, size, height_rgba)
    return OUT_ALBEDO, OUT_NORMAL


def load_image(path: Path, *, is_data: bool = False) -> bpy.types.Image:
    img = bpy.data.images.load(str(path))
    if is_data:
        img.colorspace_settings.name = "Non-Color"
    return img


def make_side_mat() -> bpy.types.Material:
    """侧面：仅顶/底很窄的一条边略压暗，中段保持原色。"""
    size_w, size_h = 64, 128
    rgba = [0] * (size_w * size_h * 4)
    base = (0.40, 0.48, 0.54)
    band = 0.10  # 上下各 10% 高度才开始压暗
    for y in range(size_h):
        v = y / (size_h - 1)
        dist = min(v, 1.0 - v)
        if dist < band:
            t = 1.0 - dist / band
            rim = t * t
        else:
            rim = 0.0
        shade = 1.0 - 0.12 * rim
        r = int(base[0] * shade * 255)
        g = int(base[1] * shade * 255)
        b = int(base[2] * shade * 255)
        for x in range(size_w):
            i = (y * size_w + x) * 4
            rgba[i : i + 4] = [r, g, b, 255]
    side_path = OUT_DIR / "side_albedo.png"
    write_png_rgba(side_path, size_w, size_h, rgba)

    img = bpy.data.images.load(str(side_path))
    mat = bpy.data.materials.new("PlatformSide")
    mat.use_nodes = True
    nt = mat.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = img
    tex.interpolation = "Linear"
    bsdf.inputs["Roughness"].default_value = 0.62
    bsdf.inputs["Metallic"].default_value = 0.12
    nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    return mat


def make_top_mat(
    albedo: bpy.types.Image, normal: bpy.types.Image
) -> bpy.types.Material:
    mat = bpy.data.materials.new("PlatformTop")
    mat.use_nodes = True
    nt = mat.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
    tex_a = nt.nodes.new("ShaderNodeTexImage")
    tex_a.image = albedo
    tex_a.interpolation = "Smart"
    tex_n = nt.nodes.new("ShaderNodeTexImage")
    tex_n.image = normal
    tex_n.interpolation = "Smart"
    nmap = nt.nodes.new("ShaderNodeNormalMap")
    nmap.inputs["Strength"].default_value = 0.85
    bsdf.inputs["Roughness"].default_value = 0.58
    bsdf.inputs["Metallic"].default_value = 0.10
    nt.links.new(tex_a.outputs["Color"], bsdf.inputs["Base Color"])
    nt.links.new(tex_n.outputs["Color"], nmap.inputs["Color"])
    nt.links.new(nmap.outputs["Normal"], bsdf.inputs["Normal"])
    nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    return mat


def build_platform(
    mat_side: bpy.types.Material, mat_top: bpy.types.Material
) -> bpy.types.Object:
    body = mesh_cube("Platform", Vector((1.0, 1.0, 1.0)), Vector((0, 0, 0)))
    apply_transforms(body)

    # 直角，不倒角

    # 上/下环绕凹槽：方环刀具一次切四边
    for z in (GROOVE_Z, -GROOVE_Z):
        outer = mesh_cube(
            "RingOuter", Vector((1.02, 1.02, GROOVE_H)), Vector((0, 0, z))
        )
        apply_transforms(outer)
        inner = mesh_cube(
            "RingInner",
            Vector((1.02 - 2 * GROOVE_DEPTH, 1.02 - 2 * GROOVE_DEPTH, GROOVE_H + 0.02)),
            Vector((0, 0, z)),
        )
        # outer - inner = 方环
        set_active(outer)
        mod = outer.modifiers.new("Ring", "BOOLEAN")
        mod.operation = "DIFFERENCE"
        mod.solver = "EXACT"
        mod.object = inner
        apply_transforms(inner)
        set_active(outer)
        bpy.ops.object.modifier_apply(modifier=mod.name)
        bpy.data.objects.remove(inner, do_unlink=True)
        boolean_diff(body, outer)

    # 材质：顶面 / 其余
    body.data.materials.clear()
    body.data.materials.append(mat_side)
    body.data.materials.append(mat_top)

    set_active(body)
    bpy.ops.object.mode_set(mode="EDIT")
    bm = bmesh.from_edit_mesh(body.data)
    uv = bm.loops.layers.uv.verify()
    for face in bm.faces:
        n = face.normal
        if abs(n.z) > 0.7:
            # 顶面与底面共用可无缝贴图（边缘略暗）
            face.material_index = 1
            for loop in face.loops:
                p = loop.vert.co
                loop[uv].uv = (p.x + 0.5, p.y + 0.5)
        else:
            face.material_index = 0
            for loop in face.loops:
                p = loop.vert.co
                # V 映射高度：0=底，1=顶 → 贴图两端压暗
                if abs(n.x) > abs(n.y):
                    loop[uv].uv = (p.y + 0.5, p.z + 0.5)
                else:
                    loop[uv].uv = (p.x + 0.5, p.z + 0.5)
    bmesh.update_edit_mesh(body.data)
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


def main() -> None:
    clear_scene()
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    print("generating seamless top maps…", file=sys.stderr)
    albedo_path, normal_path = generate_top_maps(256)
    albedo = load_image(albedo_path, is_data=False)
    normal = load_image(normal_path, is_data=True)

    mat_side = make_side_mat()
    mat_top = make_top_mat(albedo, normal)

    print("building platform mesh…", file=sys.stderr)
    build_platform(mat_side, mat_top)

    export_glb(OUT_GLB)
    print(f"Wrote {OUT_GLB}", file=sys.stderr)
    print(f"Wrote {OUT_ALBEDO}", file=sys.stderr)
    print(f"Wrote {OUT_NORMAL}", file=sys.stderr)


if __name__ == "__main__":
    main()
