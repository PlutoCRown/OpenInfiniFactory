"""用 Blender 生成 material_5：十二边单段倒角 + 面板贴图烤进 GLB。

几何：1×1×1 立方体，12 棱 bevel（segments=1），单网格单材质。
细节（板缝 / 屋形 / 灯 / 铜边）全部在 albedo 里。
导出后只保留 model.glb（不留外部 texture/normal.png）。

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python scripts/material_blocks/generate_aluminum_glb.py
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
OUT_DIR = ROOT / "assets" / "material_blocks" / "material_5"
OUT_GLB = OUT_DIR / "model.glb"

CHAMFER = 0.065
TEX_SIZE = 256


def write_png_rgba(path: Path, width: int, height: int, rgba: list[int]) -> None:
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


def clamp01(x: float) -> float:
    return 0.0 if x < 0.0 else 1.0 if x > 1.0 else x


def hash2(ix: int, iy: int) -> float:
    n = (ix * 374761393 + iy * 668265263) & 0x7FFFFFFF
    n = (n ^ (n >> 13)) * 1274126177
    return ((n ^ (n >> 16)) & 0xFFFF) / 65536.0


def value_noise(u: float, v: float, scale: float) -> float:
    x, y = u * scale, v * scale
    x0, y0 = int(math.floor(x)), int(math.floor(y))
    fx, fy = x - x0, y - y0
    fx = fx * fx * (3.0 - 2.0 * fx)
    fy = fy * fy * (3.0 - 2.0 * fy)
    a, b = hash2(x0, y0), hash2(x0 + 1, y0)
    c, d = hash2(x0, y0 + 1), hash2(x0 + 1, y0 + 1)
    return a + (b - a) * fx + (c - a) * fy + (a - b - c + d) * fx * fy


def groove_profile(dist: float, center: float, half_w: float) -> float:
    t = abs(dist - center) / half_w
    if t >= 1.0:
        return 0.0
    return (1.0 - t * t) ** 2


def edge_dist(u: float, v: float) -> float:
    return min(u, 1.0 - u, v, 1.0 - v)


def sd_box(u: float, v: float, x0: float, y0: float, x1: float, y1: float) -> float:
    dx = max(x0 - u, u - x1)
    dy = max(y0 - v, v - y1)
    out = math.sqrt(max(dx, 0.0) ** 2 + max(dy, 0.0) ** 2)
    insides = max(dx, dy)
    return insides if insides < 0.0 else out


def sd_circle(u: float, v: float, cx: float, cy: float, r: float) -> float:
    return math.hypot(u - cx, v - cy) - r


def sd_segment(u: float, v: float, ax: float, ay: float, bx: float, by: float) -> float:
    pax, pay = u - ax, v - ay
    bax, bay = bx - ax, by - ay
    denom = bax * bax + bay * bay
    t = 0.0 if denom < 1e-12 else clamp01((pax * bax + pay * bay) / denom)
    return math.hypot(pax - bax * t, pay - bay * t)


def sd_polyline(u: float, v: float, pts: list[tuple[float, float]]) -> float:
    d = 1e9
    n = len(pts)
    for i in range(n):
        a, b = pts[i], pts[(i + 1) % n]
        d = min(d, sd_segment(u, v, a[0], a[1], b[0], b[1]))
    return d


def soft_step(d: float, w: float) -> float:
    if w <= 1e-8:
        return 1.0 if d <= 0.0 else 0.0
    t = clamp01(1.0 - d / w)
    return t * t * (3.0 - 2.0 * t)


def apply_ao(rgb: tuple[int, int, int], ao: float) -> tuple[int, int, int]:
    return tuple(max(0, min(255, int(c * ao))) for c in rgb)  # type: ignore[return-value]


def generate_albedo(path: Path) -> Path:
    """面板 albedo：板缝/屋形/灯/铜边；凹槽用明暗暗示深度。"""
    size = TEX_SIZE
    body = (214, 210, 200)
    bronze = (148, 104, 68)
    seam = (110, 108, 102)
    pocket = (42, 44, 48)
    led = (255, 138, 42)
    albedo = [0] * (size * size * 4)

    bevel = 0.010
    panel = 0.08
    bronze_w = 0.030
    house = [
        (0.30, 0.70),
        (0.70, 0.70),
        (0.70, 0.44),
        (0.50, 0.26),
        (0.30, 0.44),
    ]
    px0, py0, px1, py1 = 0.68, 0.12, 0.86, 0.26
    led_cx, led_cy, led_r = 0.77, 0.19, 0.026
    strip = (0.45, 0.36, 0.55, 0.74)

    for y in range(size):
        for x in range(size):
            u = (x + 0.5) / size
            v = (y + 0.5) / size
            d = edge_dist(u, v)

            g_border = groove_profile(d, panel, 0.008)
            g_h = groove_profile(v, 0.50, 0.007) if 0.14 < u < 0.86 else 0.0
            g_v = groove_profile(u, 0.50, 0.007) if 0.14 < v < 0.86 else 0.0
            g_house = soft_step(sd_polyline(u, v, house) - 0.0035, 0.009)
            g = max(g_border, g_h * 0.65, g_v * 0.65, g_house * 0.9)

            strip_m = soft_step(sd_box(u, v, *strip), bevel * 0.7)
            pocket_m = soft_step(sd_box(u, v, px0, py0, px1, py1), bevel * 0.6)
            led_m = soft_step(sd_circle(u, v, led_cx, led_cy, led_r), 0.005)

            grain = (value_noise(u * 5.0, v * 5.0, 1.0) - 0.5) * 5.0
            rgb = tuple(max(0, min(255, int(c + grain))) for c in body)

            if d < bronze_w:
                t = clamp01((bronze_w - d) / bronze_w)
                t *= 1.0 - soft_step(d - 0.004, 0.006)
                rgb = tuple(int(rgb[i] * (1.0 - t) + bronze[i] * t) for i in range(3))

            if g > 0.15:
                # 凹槽：暗边 + 中间略亮，模拟一点浮雕
                shade = 1.0 - 0.45 * g
                hilite = 1.0 + 0.08 * g * g
                rgb = tuple(int(clamp01(c / 255.0 * shade * hilite) * 255) for c in rgb)
                rgb = tuple(
                    int(rgb[i] * (1.0 - 0.25 * g) + seam[i] * 0.25 * g)
                    for i in range(3)
                )

            if strip_m > 0.3:
                rgb = apply_ao(rgb, 1.0 - 0.12 * strip_m)
            if pocket_m > 0.25:
                t = clamp01((pocket_m - 0.25) / 0.75)
                rgb = tuple(int(rgb[i] * (1.0 - t) + pocket[i] * t) for i in range(3))
            if led_m > 0.35 and pocket_m > 0.3:
                t = clamp01((led_m - 0.35) / 0.65)
                rgb = tuple(int(rgb[i] * (1.0 - t) + led[i] * t) for i in range(3))

            i = (y * size + x) * 4
            albedo[i : i + 3] = list(rgb)
            albedo[i + 3] = 255

    write_png_rgba(path, size, size, albedo)
    return path


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


def make_mat(albedo_path: Path) -> bpy.types.Material:
    mat = bpy.data.materials.new(name="Aluminum")
    mat.use_nodes = True
    nt = mat.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
    bsdf.inputs["Metallic"].default_value = 0.18
    bsdf.inputs["Roughness"].default_value = 0.48
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = bpy.data.images.load(str(albedo_path))
    tex.interpolation = "Closest"
    nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    return mat


def build_mesh(mat: bpy.types.Material) -> bpy.types.Object:
    mesh = bpy.data.meshes.new("Aluminum")
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Aluminum", mesh)
    link(obj)
    apply_transforms(obj)

    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.mesh.bevel(offset=CHAMFER, segments=1, affect="EDGES")
    bpy.ops.object.mode_set(mode="OBJECT")

    # 主面立方投影；倒角面 UV 落在贴图铜边带
    set_active(obj)
    bpy.ops.object.mode_set(mode="EDIT")
    bm = bmesh.from_edit_mesh(obj.data)
    uv_layer = bm.loops.layers.uv.verify()
    for face in bm.faces:
        n = face.normal
        axis = max(abs(n.x), abs(n.y), abs(n.z))
        for loop in face.loops:
            p = loop.vert.co
            if abs(n.z) >= abs(n.x) and abs(n.z) >= abs(n.y):
                u, v = p.x + 0.5, p.y + 0.5
            elif abs(n.x) >= abs(n.y):
                u, v = p.y + 0.5, p.z + 0.5
            else:
                u, v = p.x + 0.5, p.z + 0.5
            if axis <= 0.92:
                # 倒角：压到贴图最外沿铜边
                u = 0.012 if u < 0.5 else 0.988
                v = 0.012 if v < 0.5 else 0.988
            loop[uv_layer].uv = (u, v)
    bmesh.update_edit_mesh(obj.data)
    bpy.ops.object.mode_set(mode="OBJECT")

    obj.data.materials.clear()
    obj.data.materials.append(mat)
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
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    tmp_tex = OUT_DIR / "_albedo_bake.png"
    print("generating albedo…", file=sys.stderr)
    generate_albedo(tmp_tex)

    clear_scene()
    mat = make_mat(tmp_tex)
    print("building chamfered cube…", file=sys.stderr)
    build_mesh(mat)
    export_glb(OUT_GLB)
    print(f"Wrote {OUT_GLB}", file=sys.stderr)

    # 有 model.glb 就不留外部贴图
    for name in ("texture.png", "normal.png", "_albedo_bake.png"):
        p = OUT_DIR / name
        if p.exists():
            p.unlink()
            print(f"Removed {p}", file=sys.stderr)


if __name__ == "__main__":
    main()
