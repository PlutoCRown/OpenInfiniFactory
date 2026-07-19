"""为 assets/stamp_materials/<id>/ 生成 model.glb 薄板。

约定（与告示牌 / 运行时 facing.yaw 一致）：
  - 局部 +Z 朝宿主，板心 z=+0.45
  - 尺寸 0.78×0.72×0.1 → 贴宿主面外凸 0.1（不内嵌）
  - 贴图用同目录 texture.png（嵌入 GLB）

用法：
  python3 scripts/stamp_materials/generate_stamp_glb.py
"""

from __future__ import annotations

import json
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2] / "assets" / "stamp_materials"
NEAREST, REPEAT = 9728, 10497

# 板尺寸与偏置（局部 +Z 朝宿主）
SIZE = (0.78, 0.72, 0.1)
CENTER_Z = 0.45


def f32_list(vals):
    return b"".join(struct.pack("<f", float(v)) for v in vals)


def u16_list(vals):
    return b"".join(struct.pack("<H", int(v)) for v in vals)


def plate_mesh():
    hx, hy, hz = SIZE[0] * 0.5, SIZE[1] * 0.5, SIZE[2] * 0.5
    z0, z1 = CENTER_Z - hz, CENTER_Z + hz
    # 六面，每面 4 顶点（共 24）；引擎对 Stamp 不走 AO 立方体路径
    faces = [
        # +Z 朝宿主（贴宿主侧）
        (
            [(-hx, -hy, z1), (hx, -hy, z1), (hx, hy, z1), (-hx, hy, z1)],
            (0.0, 0.0, 1.0),
        ),
        # -Z 朝外（可见主面）
        (
            [(hx, -hy, z0), (-hx, -hy, z0), (-hx, hy, z0), (hx, hy, z0)],
            (0.0, 0.0, -1.0),
        ),
        # +Y
        (
            [(-hx, hy, z0), (hx, hy, z0), (hx, hy, z1), (-hx, hy, z1)],
            (0.0, 1.0, 0.0),
        ),
        # -Y
        (
            [(-hx, -hy, z1), (hx, -hy, z1), (hx, -hy, z0), (-hx, -hy, z0)],
            (0.0, -1.0, 0.0),
        ),
        # +X
        (
            [(hx, -hy, z0), (hx, hy, z0), (hx, hy, z1), (hx, -hy, z1)],
            (1.0, 0.0, 0.0),
        ),
        # -X
        (
            [(-hx, hy, z0), (-hx, -hy, z0), (-hx, -hy, z1), (-hx, hy, z1)],
            (-1.0, 0.0, 0.0),
        ),
    ]
    uvs_face = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
    positions, normals, uvs, indices = [], [], [], []
    for corners, normal in faces:
        base = len(positions)
        for p, uv in zip(corners, uvs_face):
            positions.append(p)
            normals.append(normal)
            uvs.append(uv)
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3])
    return positions, normals, uvs, indices


def write_glb(path: Path, name: str, png: bytes) -> None:
    positions, normals, uvs, indices = plate_mesh()
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
    gltf = {
        "asset": {"version": "2.0", "generator": "oif-stamp-plate"},
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
        "materials": [
            {
                "name": name,
                "pbrMetallicRoughness": {
                    "baseColorFactor": [1, 1, 1, 1],
                    "baseColorTexture": {"index": 0},
                    "metallicFactor": 0.0,
                    "roughnessFactor": 0.92,
                },
                "doubleSided": True,
            }
        ],
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
    path.write_bytes(out)
    print("wrote", path, "verts", len(positions))


def main() -> None:
    if not ROOT.is_dir():
        raise SystemExit(f"missing {ROOT}")
    for pack in sorted(p for p in ROOT.iterdir() if p.is_dir()):
        tex = pack / "texture.png"
        if not tex.is_file():
            print("skip (no texture.png):", pack.name)
            continue
        write_glb(pack / "model.glb", pack.name, tex.read_bytes())


if __name__ == "__main__":
    main()
