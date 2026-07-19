"""生成 short_grass（交叉面片）与 glass（透明边框立方体）场景块。

用法：
  python3 scripts/scene_blocks/generate_short_grass_and_glass.py
图标：
  ./scripts/bake_scene_icons.sh --scene-only --only <id>
"""

from __future__ import annotations

import json
import math
import random
import struct
from io import BytesIO
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2] / "assets" / "scene_blocks"

NEAREST, REPEAT = 9728, 10497


def write_png(img):
    buf = BytesIO()
    img.save(buf, format="PNG", optimize=True)
    return buf.getvalue()


def f32_list(vals):
    return b"".join(struct.pack("<f", float(v)) for v in vals)


def u16_list(vals):
    return b"".join(struct.pack("<H", int(v)) for v in vals)


def write_glb(
    path,
    name,
    positions,
    normals,
    uvs,
    indices,
    png,
    *,
    roughness=0.9,
    metallic=0.0,
    alpha_mode="OPAQUE",
    alpha_cutoff=0.5,
    double_sided=False,
):
    pos_b = f32_list([c for p in positions for c in p])
    nor_b = f32_list([c for n in normals for c in n])
    uv_b = f32_list([c for u in uvs for c in u])
    blob = bytearray()
    views = []

    def add_view(data, target=None):
        while len(blob) % 4:
            blob.append(0)
        off = len(blob)
        blob.extend(data)
        while len(blob) % 4:
            blob.append(0)
        v = {"buffer": 0, "byteOffset": off, "byteLength": len(data)}
        if target is not None:
            v["target"] = target
        views.append(v)
        return len(views) - 1

    v_idx = add_view(u16_list(indices), 34963)
    v_pos = add_view(pos_b, 34962)
    v_nor = add_view(nor_b, 34962)
    v_uv = add_view(uv_b, 34962)
    v_img = add_view(png)
    pos_min = [min(p[i] for p in positions) for i in range(3)]
    pos_max = [max(p[i] for p in positions) for i in range(3)]
    mat = {
        "name": name,
        "pbrMetallicRoughness": {
            "baseColorFactor": [1, 1, 1, 1],
            "baseColorTexture": {"index": 0},
            "metallicFactor": metallic,
            "roughnessFactor": roughness,
        },
        "alphaMode": alpha_mode,
    }
    if alpha_mode == "MASK":
        mat["alphaCutoff"] = alpha_cutoff
    if double_sided:
        mat["doubleSided"] = True
    gltf = {
        "asset": {"version": "2.0", "generator": "oif-scene-block"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0, "name": name}],
        "meshes": [
            {
                "name": name,
                "primitives": [
                    {
                        "attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2},
                        "indices": 3,
                        "material": 0,
                    }
                ],
            }
        ],
        "materials": [mat],
        "textures": [{"sampler": 0, "source": 0}],
        "samplers": [
            {
                "magFilter": NEAREST,
                "minFilter": NEAREST,
                "wrapS": REPEAT,
                "wrapT": REPEAT,
            }
        ],
        "images": [{"bufferView": v_img, "mimeType": "image/png"}],
        "accessors": [
            {
                "bufferView": v_pos,
                "componentType": 5126,
                "count": len(positions),
                "type": "VEC3",
                "max": pos_max,
                "min": pos_min,
            },
            {
                "bufferView": v_nor,
                "componentType": 5126,
                "count": len(normals),
                "type": "VEC3",
            },
            {
                "bufferView": v_uv,
                "componentType": 5126,
                "count": len(uvs),
                "type": "VEC2",
            },
            {
                "bufferView": v_idx,
                "componentType": 5123,
                "count": len(indices),
                "type": "SCALAR",
            },
        ],
        "bufferViews": views,
        "buffers": [{"byteLength": len(blob)}],
    }

    def pad4_space(b):
        return b + b" " * ((4 - len(b) % 4) % 4)

    def pad4(b):
        return b + b"\x00" * ((4 - len(b) % 4) % 4)

    json_bytes = pad4_space(json.dumps(gltf, separators=(",", ":")).encode())
    bin_chunk = pad4(bytes(blob))
    total = 12 + 8 + len(json_bytes) + 8 + len(bin_chunk)
    out = bytearray()
    out += b"glTF" + struct.pack("<II", 2, total)
    out += struct.pack("<I", len(json_bytes)) + b"JSON" + json_bytes
    out += struct.pack("<I", len(bin_chunk)) + b"BIN\x00" + bin_chunk
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(out)
    print("wrote", path, "verts", len(positions), "alpha", alpha_mode)


def add_quad(positions, normals, uvs, indices, verts, normal, uv_quad):
    base = len(positions)
    for i in range(4):
        positions.append(verts[i])
        normals.append(normal)
        uvs.append(uv_quad[i])
    indices.extend([base, base + 1, base + 2, base, base + 2, base + 3])


def write_meta(path, block_id, connectable, collision=True, directional=False):
    meta = {
        "$schema": "../../../schemas/scene_block.meta.schema.json",
        "id": block_id,
        "collision": collision,
        "connectable": connectable,
    }
    if directional:
        meta["directional"] = True
    path.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    print("wrote", path)


# --- short_grass texture: MC-like tuft silhouette ---
def make_grass_plant_tex(size=16):
    rng = random.Random(7)
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    px = img.load()
    # several blades from bottom center upward
    cx = size // 2
    for blade in range(7):
        x0 = cx + rng.randint(-5, 5)
        top = rng.randint(1, 4)
        width = 1 if rng.random() < 0.7 else 2
        green = (
            46 + rng.randint(0, 40),
            120 + rng.randint(0, 50),
            40 + rng.randint(0, 30),
            255,
        )
        dark = (
            30 + rng.randint(0, 20),
            80 + rng.randint(0, 30),
            25 + rng.randint(0, 20),
            255,
        )
        x = float(x0)
        for y in range(size - 1, top - 1, -1):
            lean = (size - 1 - y) * rng.uniform(-0.08, 0.08) + (blade - 3) * 0.05
            x += lean
            ix = int(round(x))
            col = green if (y + blade) % 3 else dark
            for dx in range(width):
                xx = ix + dx
                if 0 <= xx < size:
                    px[xx, y] = col
            # tip taper
            if y <= top + 2:
                width = 1
    return img


def short_grass_mesh():
    """两片交叉面片（XY 与 ZY），双面由 material.doubleSided 处理，这里只写正面绕序。"""
    h = 0.5
    positions, normals, uvs, indices = [], [], [], []
    uv = [(0, 1), (1, 1), (1, 0), (0, 0)]
    # plane in XY (facing +Z): bottom-left, bottom-right, top-right, top-left
    add_quad(
        positions,
        normals,
        uvs,
        indices,
        [(-h, -h, 0), (h, -h, 0), (h, h, 0), (-h, h, 0)],
        (0, 0, 1),
        uv,
    )
    # plane in ZY (facing +X)
    add_quad(
        positions,
        normals,
        uvs,
        indices,
        [(0, -h, h), (0, -h, -h), (0, h, -h), (0, h, h)],
        (1, 0, 0),
        uv,
    )
    return positions, normals, uvs, indices


# --- glass texture: light blue border, transparent center ---
def make_glass_tex(size=16, border=2):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    px = img.load()
    # soft cyan-blue glass edge
    edge = (160, 210, 230, 140)
    corner = (190, 230, 245, 180)
    for y in range(size):
        for x in range(size):
            on_edge = (
                x < border or y < border or x >= size - border or y >= size - border
            )
            if not on_edge:
                continue
            # slightly brighter at corners
            near_corner = (
                (x < border and y < border)
                or (x < border and y >= size - border)
                or (x >= size - border and y < border)
                or (x >= size - border and y >= size - border)
            )
            px[x, y] = corner if near_corner else edge
    # inner highlight line (1px inset, more transparent)
    if border >= 1:
        for i in range(border, size - border):
            for x, y in (
                (border, i),
                (size - border - 1, i),
                (i, border),
                (i, size - border - 1),
            ):
                if px[x, y][3] == 0:
                    px[x, y] = (200, 235, 245, 70)
    return img


def cube_mesh():
    h = 0.5
    faces = [
        ([(h, -h, h), (h, -h, -h), (h, h, -h), (h, h, h)], (1, 0, 0)),
        ([(-h, -h, -h), (-h, -h, h), (-h, h, h), (-h, h, -h)], (-1, 0, 0)),
        ([(-h, h, h), (h, h, h), (h, h, -h), (-h, h, -h)], (0, 1, 0)),
        ([(-h, -h, -h), (h, -h, -h), (h, -h, h), (-h, -h, h)], (0, -1, 0)),
        ([(-h, -h, h), (h, -h, h), (h, h, h), (-h, h, h)], (0, 0, 1)),
        ([(h, -h, -h), (-h, -h, -h), (-h, h, -h), (h, h, -h)], (0, 0, -1)),
    ]
    uv_full = [(0, 1), (1, 1), (1, 0), (0, 0)]
    positions, normals, uvs, indices = [], [], [], []
    for verts, n in faces:
        add_quad(positions, normals, uvs, indices, verts, n, uv_full)
    return positions, normals, uvs, indices


# short_grass
pos, nor, uv, idx = short_grass_mesh()
png = write_png(make_grass_plant_tex(16))
write_glb(
    ROOT / "short_grass" / "model.glb",
    "short_grass",
    pos,
    nor,
    uv,
    idx,
    png,
    roughness=0.95,
    alpha_mode="MASK",
    alpha_cutoff=0.1,
    double_sided=True,
)
write_meta(
    ROOT / "short_grass" / "meta.json", "short_grass", [False] * 6, collision=False
)

# glass
pos, nor, uv, idx = cube_mesh()
png = write_png(make_glass_tex(16, border=2))
write_glb(
    ROOT / "glass" / "model.glb",
    "glass",
    pos,
    nor,
    uv,
    idx,
    png,
    roughness=0.05,
    metallic=0.0,
    alpha_mode="BLEND",
    double_sided=False,
)
write_meta(ROOT / "glass" / "meta.json", "glass", [True] * 6, collision=True)

print("done")
