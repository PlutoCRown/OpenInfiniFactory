"""Regenerate grass textures (MC-style jagged side) + bake model.glb with glTF UV (V=0 = top)."""
import json, struct, math, random
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    import subprocess, sys
    subprocess.check_call([sys.executable, '-m', 'pip', 'install', 'pillow', '-q'])
    from PIL import Image

SIZE = 64
OUT = Path('assets/scene_blocks/grass')

def clamp(v, lo=0, hi=255):
    return max(lo, min(hi, int(v)))

def noise2(x, y, seed):
    n = (x * 374761393 + y * 668265263 + seed * 982451653) & 0xFFFFFFFF
    n = (n ^ (n >> 13)) * 1274126177 & 0xFFFFFFFF
    return (n & 255)

def shade(base, n, amount):
    d = (n - 128) * amount / 128
    return tuple(clamp(c + d) for c in base)

def gen_top():
    # grass top: green with flecks
    im = Image.new('RGB', (SIZE, SIZE))
    px = im.load()
    for y in range(SIZE):
        for x in range(SIZE):
            n = noise2(x, y, 197)
            fleck = ((x * 7 + y * 19 + n) % 31) < 4
            base = (93, 157, 58) if not fleck else (66, 128, 45)
            px[x, y] = shade(base, n, 28)
    return im

def gen_bottom():
    # dirt
    im = Image.new('RGB', (SIZE, SIZE))
    px = im.load()
    for y in range(SIZE):
        for x in range(SIZE):
            n = noise2(x, y, 211)
            fleck = ((x * 11 + y * 17 + n) % 29) < 3
            base = (134, 96, 67) if not fleck else (110, 78, 52)
            px[x, y] = shade(base, n, 22)
    return im

def gen_side(top_im, bottom_im):
    # MC-like: per-column jagged grass/dirt boundary near mid height
    im = Image.new('RGB', (SIZE, SIZE))
    px = im.load()
    top_px = top_im.load()
    bot_px = bottom_im.load()
    rng = random.Random(42)
    # boundary height from top of image (grass grows down from top)
    # base around SIZE*0.42 from top, with jagged variation
    heights = []
    h = SIZE * 0.40
    for x in range(SIZE):
        h += rng.uniform(-1.8, 1.8)
        h += 0.35 * math.sin(x * 0.7) + 0.55 * math.sin(x * 1.9 + 1.2)
        h = max(SIZE * 0.28, min(SIZE * 0.55, h))
        heights.append(h)
    # smooth a bit
    for _ in range(2):
        heights = [heights[0]] + [
            0.25 * heights[i-1] + 0.5 * heights[i] + 0.25 * heights[i+1]
            for i in range(1, SIZE-1)
        ] + [heights[-1]]
    for y in range(SIZE):
        for x in range(SIZE):
            # y=0 top of side texture = grass edge of cube face top
            if y < heights[x] - 0.5:
                # grass: sample top texture with slight dirt tint near boundary
                c = top_px[x, y % SIZE]
                if y > heights[x] - 3:
                    # blend toward dirt near edge
                    t = (y - (heights[x] - 3)) / 3
                    d = bot_px[x, y % SIZE]
                    c = tuple(int(c[i] * (1-t) + d[i] * t) for i in range(3))
                px[x, y] = c
            else:
                # dirt below jagged line
                c = bot_px[x, y % SIZE]
                n = noise2(x, y, 233)
                px[x, y] = shade(c, n, 10)
            # occasional grass blade hanging below boundary
            if abs(y - heights[x]) < 1.2 and noise2(x, y, 99) > 200:
                px[x, y] = top_px[x, (y * 3) % SIZE]
    return im

top = gen_top()
bottom = gen_bottom()
side = gen_side(top, bottom)

# atlas: top, side, bottom (PNG top → bottom) for glTF V=0=image top
atlas = Image.new('RGB', (SIZE, SIZE * 3))
atlas.paste(top, (0, 0))
atlas.paste(side, (0, SIZE))
atlas.paste(bottom, (0, SIZE * 2))
atlas_path = Path('/tmp/grass_atlas_new.png')
atlas.save(atlas_path)
print('atlas', atlas.size)

# --- bake GLB ---
MIN, MAX = -0.5, 0.5

def cube_faces_uv(face_index, local):
    # glTF: V=0 at TOP of image. Atlas strips: top[0,1/3), side[1/3,2/3), bottom[2/3,1]
    u = local[0]
    lv = local[1]  # 0=bottom of face, 1=top of face
    if face_index == 4:  # +Y top → top strip
        v0, v1 = 0.0, 1.0 / 3.0
        return [u, v0 + lv * (v1 - v0)]
    if face_index == 5:  # -Y bottom → bottom strip
        v0, v1 = 2.0 / 3.0, 1.0
        return [u, v0 + lv * (v1 - v0)]
    # sides: face top (lv=1) → grass = upper part of side strip = lower V
    # face bottom (lv=0) → dirt = lower part of side strip = higher V
    v_grass, v_dirt = 1.0 / 3.0, 2.0 / 3.0
    return [u, v_dirt + lv * (v_grass - v_dirt)]

def build_cube():
    faces = [
        ([[MIN,MIN,MAX],[MAX,MIN,MAX],[MAX,MAX,MAX],[MIN,MAX,MAX]], [0,0,1]),
        ([[MAX,MIN,MIN],[MIN,MIN,MIN],[MIN,MAX,MIN],[MAX,MAX,MIN]], [0,0,-1]),
        ([[MAX,MIN,MAX],[MAX,MIN,MIN],[MAX,MAX,MIN],[MAX,MAX,MAX]], [1,0,0]),
        ([[MIN,MIN,MIN],[MIN,MIN,MAX],[MIN,MAX,MAX],[MIN,MAX,MIN]], [-1,0,0]),
        ([[MIN,MAX,MAX],[MAX,MAX,MAX],[MAX,MAX,MIN],[MIN,MAX,MIN]], [0,1,0]),
        ([[MIN,MIN,MIN],[MAX,MIN,MIN],[MAX,MIN,MAX],[MIN,MIN,MAX]], [0,-1,0]),
    ]
    local_uvs = [[0,0],[1,0],[1,1],[0,1]]
    positions=[]; normals=[]; uvs=[]; indices=[]
    for fi, (corners, n) in enumerate(faces):
        base = len(positions)
        for i, p in enumerate(corners):
            positions.append(p)
            normals.append(n)
            uvs.append(cube_faces_uv(fi, local_uvs[i]))
        indices.extend([base, base+1, base+2, base, base+2, base+3])
    return positions, normals, uvs, indices

def align4(n):
    return (4 - (n % 4)) % 4

def pack_f32(vals):
    return b''.join(struct.pack('<f', float(v)) for v in vals)

def write_glb(out_path, png_bytes):
    positions, normals, uvs, indices = build_cube()
    pos_bin = pack_f32([c for p in positions for c in p])
    nor_bin = pack_f32([c for n in normals for c in n])
    uv_bin = pack_f32([c for uv in uvs for c in uv])
    idx_bin = b''.join(struct.pack('<H', i) for i in indices)
    parts=[]; offsets={}; cursor=0
    for name, data in [('pos', pos_bin), ('nor', nor_bin), ('uv', uv_bin), ('idx', idx_bin), ('img', png_bytes)]:
        pad = align4(len(data))
        offsets[name] = (cursor, len(data))
        parts.append(data + b'\x00'*pad)
        cursor += len(data) + pad
    bin_blob = b''.join(parts)
    def bv(name, target):
        off, length = offsets[name]
        return {"buffer": 0, "byteOffset": off, "byteLength": length, "target": target}
    gltf = {
        "asset": {"version": "2.0", "generator": "oif-bake-grass"},
        "buffers": [{"byteLength": len(bin_blob)}],
        "bufferViews": [
            bv('pos', 34962), bv('nor', 34962), bv('uv', 34962), bv('idx', 34963),
            {"buffer": 0, "byteOffset": offsets['img'][0], "byteLength": offsets['img'][1]},
        ],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": len(positions), "type": "VEC3",
             "max": [MAX,MAX,MAX], "min": [MIN,MIN,MIN]},
            {"bufferView": 1, "componentType": 5126, "count": len(normals), "type": "VEC3"},
            {"bufferView": 2, "componentType": 5126, "count": len(uvs), "type": "VEC2"},
            {"bufferView": 3, "componentType": 5123, "count": len(indices), "type": "SCALAR"},
        ],
        "images": [{"bufferView": 4, "mimeType": "image/png"}],
        "samplers": [{"magFilter": 9729, "minFilter": 9729, "wrapS": 10497, "wrapT": 10497}],
        "textures": [{"sampler": 0, "source": 0}],
        "materials": [{
            "name": "grass",
            "pbrMetallicRoughness": {
                "baseColorFactor": [1,1,1,1],
                "baseColorTexture": {"index": 0},
                "metallicFactor": 0.0,
                "roughnessFactor": 0.96,
            },
        }],
        "meshes": [{"name": "grass", "primitives": [{
            "attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2},
            "indices": 3, "material": 0,
        }]}],
        "nodes": [{"mesh": 0, "name": "grass"}],
        "scenes": [{"nodes": [0]}],
        "scene": 0,
    }
    json_bytes = json.dumps(gltf, separators=(',', ':')).encode()
    json_pad = align4(len(json_bytes))
    json_chunk = json_bytes + b' ' * json_pad
    bin_pad = align4(len(bin_blob))
    bin_chunk = bin_blob + b'\x00' * bin_pad
    total = 12 + 8 + len(json_chunk) + 8 + len(bin_chunk)
    out = bytearray()
    out += struct.pack('<4sII', b'glTF', 2, total)
    out += struct.pack('<I4s', len(json_chunk), b'JSON')
    out += json_chunk
    out += struct.pack('<I4s', len(bin_chunk), b'BIN\x00')
    out += bin_chunk
    out_path.write_bytes(out)
    print('wrote', out_path, len(out))

png_bytes = atlas_path.read_bytes()
write_glb(OUT / 'model.glb', png_bytes)

# also save preview strips
side.save('/tmp/grass_side_jagged.png')
print('side jagged preview /tmp/grass_side_jagged.png')
