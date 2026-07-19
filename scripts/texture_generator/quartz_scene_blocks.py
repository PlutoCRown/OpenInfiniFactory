"""生成石英场景方块：共享大理石贴图 + 三款 model.glb + meta.json"""
import json, struct, zlib, math
from pathlib import Path
from io import BytesIO
from PIL import Image, ImageDraw, ImageFilter
import random

ROOT = Path('assets/scene_blocks')
NEAREST = 9728
REPEAT = 10497

def write_png(img: Image.Image) -> bytes:
    buf = BytesIO()
    img.save(buf, format='PNG', optimize=True)
    return buf.getvalue()

def make_marble_texture(size=32) -> Image.Image:
    """浅色大理石：底色偏乳白 + 柔和灰脉 + 细微噪点（低分辨率、nearest 友好）"""
    rng = random.Random(42)
    img = Image.new('RGBA', (size, size))
    px = img.load()
    # base cream-white
    for y in range(size):
        for x in range(size):
            n = (rng.random() - 0.5) * 10
            r = int(max(0, min(255, 236 + n)))
            g = int(max(0, min(255, 234 + n * 0.9)))
            b = int(max(0, min(255, 228 + n * 0.7)))
            px[x, y] = (r, g, b, 255)

    # soft vein strokes (draw on higher-res then downsample? keep pixel for MC feel)
    # draw veins as thin darker pixel runs
    for _ in range(5):
        x = rng.uniform(0, size)
        y = rng.uniform(0, size)
        angle = rng.uniform(-0.6, 0.6) + (math.pi * 0.15 if rng.random() < 0.5 else -0.2)
        length = rng.uniform(size * 0.6, size * 1.4)
        thickness = rng.choice([1, 1, 2])
        shade = rng.randint(185, 210)
        steps = int(length * 3)
        for i in range(steps):
            t = i / max(1, steps - 1)
            cx = x + math.cos(angle) * length * t + math.sin(t * 6) * 0.8
            cy = y + math.sin(angle) * length * t + math.cos(t * 5) * 0.6
            for dy in range(-thickness, thickness + 1):
                for dx in range(-thickness, thickness + 1):
                    if dx*dx + dy*dy > thickness*thickness:
                        continue
                    ix, iy = int(cx + dx), int(cy + dy)
                    if 0 <= ix < size and 0 <= iy < size:
                        r, g, b, a = px[ix, iy]
                        # blend toward cooler grey
                        f = 0.35 if (dx == 0 and dy == 0) else 0.18
                        px[ix, iy] = (
                            int(r * (1 - f) + shade * f),
                            int(g * (1 - f) + (shade - 2) * f),
                            int(b * (1 - f) + (shade - 4) * f),
                            255,
                        )
    # subtle brighter flecks
    for _ in range(size * 2):
        x, y = rng.randrange(size), rng.randrange(size)
        r, g, b, a = px[x, y]
        px[x, y] = (min(255, r + 12), min(255, g + 12), min(255, b + 10), 255)
    return img

def align4(n):
    return (n + 3) & ~3

def pad4(b: bytes) -> bytes:
    return b + b'\x00' * ((4 - len(b) % 4) % 4)

def pad4_space(b: bytes) -> bytes:
    return b + b' ' * ((4 - len(b) % 4) % 4)

def f32_list(vals):
    return b''.join(struct.pack('<f', float(v)) for v in vals)

def u16_list(vals):
    return b''.join(struct.pack('<H', int(v)) for v in vals)

def write_glb(path: Path, name: str, positions, normals, uvs, indices, png: bytes,
              roughness=0.55, metallic=0.0):
    """positions/normals/uvs: list of triples/pairs; indices: list of int"""
    pos_b = f32_list([c for p in positions for c in p])
    nor_b = f32_list([c for n in normals for c in n])
    uv_b = f32_list([c for u in uvs for c in u])
    idx_b = u16_list(indices)
    if len(indices) % 2 == 1:
        idx_b += b'\x00\x00'  # pad to 4-byte for bufferView alignment later

    # Build binary blob with aligned bufferViews
    blob = bytearray()
    views = []

    def add_view(data: bytes, target=None):
        # align to 4
        while len(blob) % 4:
            blob.append(0)
        offset = len(blob)
        blob.extend(data)
        view = {'buffer': 0, 'byteOffset': offset, 'byteLength': len(data)}
        if target is not None:
            view['target'] = target
        views.append(view)
        return len(views) - 1

    # indices may need even count for UINT16 alignment of following — we pad indices bufferView length
    idx_view = add_view(idx_b if len(idx_b) % 4 == 0 else idx_b + b'\x00\x00', 34963)
    # fix: if we padded, byteLength should be actual used for accessor, view can be padded
    # Simpler: rebuild cleanly

    blob = bytearray()
    views = []

    def add_view(data: bytes, target=None, byte_stride=None):
        while len(blob) % 4:
            blob.append(0)
        offset = len(blob)
        blob.extend(data)
        while len(blob) % 4:
            blob.append(0)
        view = {'buffer': 0, 'byteOffset': offset, 'byteLength': len(data)}
        if target is not None:
            view['target'] = target
        if byte_stride is not None:
            view['byteStride'] = byte_stride
        views.append(view)
        return len(views) - 1

    v_idx = add_view(u16_list(indices), 34963)
    v_pos = add_view(pos_b, 34962)
    v_nor = add_view(nor_b, 34962)
    v_uv = add_view(uv_b, 34962)
    v_img = add_view(png)

    pos_min = [min(p[i] for p in positions) for i in range(3)]
    pos_max = [max(p[i] for p in positions) for i in range(3)]

    accessors = [
        {'bufferView': v_pos, 'componentType': 5126, 'count': len(positions), 'type': 'VEC3',
         'max': pos_max, 'min': pos_min},
        {'bufferView': v_nor, 'componentType': 5126, 'count': len(normals), 'type': 'VEC3'},
        {'bufferView': v_uv, 'componentType': 5126, 'count': len(uvs), 'type': 'VEC2'},
        {'bufferView': v_idx, 'componentType': 5123, 'count': len(indices), 'type': 'SCALAR'},
    ]

    gltf = {
        'asset': {'version': '2.0', 'generator': 'oif-scene-block'},
        'scene': 0,
        'scenes': [{'nodes': [0]}],
        'nodes': [{'mesh': 0, 'name': name}],
        'meshes': [{
            'name': name,
            'primitives': [{
                'attributes': {'POSITION': 0, 'NORMAL': 1, 'TEXCOORD_0': 2},
                'indices': 3,
                'material': 0,
            }],
        }],
        'materials': [{
            'name': name,
            'pbrMetallicRoughness': {
                'baseColorFactor': [1, 1, 1, 1],
                'baseColorTexture': {'index': 0},
                'metallicFactor': metallic,
                'roughnessFactor': roughness,
            },
        }],
        'textures': [{'sampler': 0, 'source': 0}],
        'samplers': [{
            'magFilter': NEAREST,
            'minFilter': NEAREST,
            'wrapS': REPEAT,
            'wrapT': REPEAT,
        }],
        'images': [{'bufferView': v_img, 'mimeType': 'image/png'}],
        'accessors': accessors,
        'bufferViews': views,
        'buffers': [{'byteLength': len(blob)}],
    }

    json_bytes = pad4_space(json.dumps(gltf, separators=(',', ':')).encode('utf-8'))
    bin_chunk = pad4(bytes(blob))
    total = 12 + 8 + len(json_bytes) + 8 + len(bin_chunk)
    out = bytearray()
    out += b'glTF'
    out += struct.pack('<II', 2, total)
    out += struct.pack('<I', len(json_bytes)) + b'JSON' + json_bytes
    out += struct.pack('<I', len(bin_chunk)) + b'BIN\x00' + bin_chunk
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(out)
    print(f'wrote {path} ({len(out)} bytes, verts={len(positions)}, tris={len(indices)//3})')

def add_quad(positions, normals, uvs, indices, verts, normal, uv_quad):
    """verts: 4 xyz in CCW when looking along -normal (outward). uv_quad: 4 uv"""
    base = len(positions)
    for i in range(4):
        positions.append(verts[i])
        normals.append(normal)
        uvs.append(uv_quad[i])
    indices.extend([base, base + 1, base + 2, base, base + 2, base + 3])

def add_tri(positions, normals, uvs, indices, verts, normal, uv_tri):
    base = len(positions)
    for i in range(3):
        positions.append(verts[i])
        normals.append(normal)
        uvs.append(uv_tri[i])
    indices.extend([base, base + 1, base + 2])

def cube_mesh():
    # unit cube, face order +X -X +Y -Y +Z -Z, 4 verts each — matches stone style for AO path
    h = 0.5
    faces = [
        # +X
        ([(h,-h,-h),(h,-h,h),(h,h,h),(h,h,-h)], (1,0,0)),
        # -X
        ([(-h,-h,h),(-h,-h,-h),(-h,h,-h),(-h,h,h)], (-1,0,0)),
        # +Y
        ([(-h,h,-h),(h,h,-h),(h,h,h),(-h,h,h)], (0,1,0)),
        # -Y
        ([(-h,-h,h),(h,-h,h),(h,-h,-h),(-h,-h,-h)], (0,-1,0)),
        # +Z
        ([(-h,-h,h),(-h,h,h),(h,h,h),(h,-h,h)], (0,0,1)),
        # -Z
        ([(h,-h,-h),(h,h,-h),(-h,h,-h),(-h,-h,-h)], (0,0,-1)),
    ]
    # UV: V=0 top of image (glTF). Map each face to full texture.
    uv_full = [(0,1),(1,1),(1,0),(0,0)]  # BL BR TR TL in image space with V flip → 
    # For stone they used atlas UVs; we use full atlas per face like dirt-ish
    # Standard: bottom-left of face → (0,1), bottom-right (1,1), top-right (1,0), top-left (0,0)
    positions, normals, uvs, indices = [], [], [], []
    for verts, n in faces:
        add_quad(positions, normals, uvs, indices, verts, n, uv_full)
    return positions, normals, uvs, indices

def slope_mesh():
    """楔形斜坡：底面+北立面完整，东西三角侧面，斜面向南下。"""
    h = 0.5
    # corners
    # bottom: y=-h
    a = (-h, -h, -h)  # NW bottom (toward -Z)
    b = ( h, -h, -h)  # NE bottom
    c = ( h, -h,  h)  # SE bottom
    d = (-h, -h,  h)  # SW bottom
    e = (-h,  h, -h)  # NW top
    f = ( h,  h, -h)  # NE top

    positions, normals, uvs, indices = [], [], [], []
    uv4 = [(0,1),(1,1),(1,0),(0,0)]
    uv3 = [(0,1),(1,1),(0.5,0)]

    # bottom -Y
    add_quad(positions, normals, uvs, indices, [a, b, c, d], (0,-1,0), uv4)
    # back -Z (full)
    add_quad(positions, normals, uvs, indices, [b, a, e, f], (0,0,-1), uv4)
    # +X triangle: b,c,f
    nxp = (1, 0, 0)
    add_tri(positions, normals, uvs, indices, [b, c, f], nxp, uv3)
    # -X triangle: a,e,d
    nxm = (-1, 0, 0)
    add_tri(positions, normals, uvs, indices, [a, e, d], nxm, uv3)
    # slope face: e,f,c,d — normal pointing up-south
    # plane from e-f-c: edges f-e = (1,0,0), c-f = (0,-1,1)
    # cross: i(0*1 - 0*(-1)) - j(1*1 - 0*0) + k(1*(-1) - 0*0) = (0, -1, -1)? 
    # (f-e)×(c-f) = (1,0,0)×(0,-1,1) = (0*1-0*(-1), 0*0-1*1, 1*(-1)-0*0) = (0, -1, -1)
    # Wait we want outward = up and toward +Z: (0, 1, 1) normalized
    # Use (e-f)×(d-e) or order e,f,c,d CCW from outside
    # Outside looking from above-south: e -> f -> c -> d
    # (f-e)×(c-f) = (1,0,0)×(0,-1,1) = (0,-1,-1) — points down-north, wrong
    # Flip: e,d,c,f
    # (d-e)×(c-d) = (0,-1,1)×(1,0,0) = (0*0-1*0, 1*1-0*0, 0*0-(-1)*1) = (0,1,1) ✓
    sn = (0, 1/math.sqrt(2), 1/math.sqrt(2))
    add_quad(positions, normals, uvs, indices, [e, d, c, f], sn,
             [(0,0),(0,1),(1,1),(1,0)])
    return positions, normals, uvs, indices

def pillar_mesh(half=0.42, chamfer=0.08):
    """略瘦八角柱：主面较长、角只切一点。"""
    w = half
    c = chamfer
    # 8 XY points, CCW from top (+Y looking down is CW for outward? 
    # For side faces, walk CCW when viewed from outside)
    # Top view, CCW around +Y:
    ring = [
        (-w + c,  w),      # N edge left
        ( w - c,  w),      # N edge right
        ( w,      w - c),  # E edge north
        ( w,     -w + c),  # E edge south
        ( w - c, -w),      # S edge right
        (-w + c, -w),      # S edge left
        (-w,     -w + c),  # W edge south
        (-w,      w - c),  # W edge north
    ]
    y0, y1 = -0.5, 0.5
    positions, normals, uvs, indices = [], [], [], []

    # side quads
    for i in range(8):
        x0, z0 = ring[i]
        x1, z1 = ring[(i + 1) % 8]
        # outward normal in XZ
        dx, dz = x1 - x0, z1 - z0
        # edge goes CCW; outward is rotate CW in XZ? 
        # CCW ring: outward = (dz, -dx) normalized? For edge along +X on north face (z=w): 
        # from (-w+c,w) to (w-c,w): dx>0,dz=0; outward should be +Z = (0,1) in xz = (dz, -dx)? (0,-dx) wrong
        # rotate edge 90° outward for CCW polygon: (dz, -dx) wait: rotate left (CCW) of edge direction is inward for CCW boundary...
        # For CCW boundary, inward is left of edge, outward is right: (dz, -dx)? edge (dx,dz), right = (dz, -dx)
        # north edge dx>0,dz=0: right=(0,-dx)=(0,-1) = -Z, but north face needs +Z. So for CCW ring viewed from +Y, outward is LEFT = (-dz, dx)
        nx, nz = -dz, dx
        length = math.hypot(nx, nz) or 1.0
        nx, nz = nx / length, nz / length
        # UV: u along perimeter fraction, v from bottom to top (v=1 bottom in glTF image space)
        u0 = i / 8
        u1 = (i + 1) / 8
        verts = [
            (x0, y0, z0),
            (x1, y0, z1),
            (x1, y1, z1),
            (x0, y1, z0),
        ]
        add_quad(positions, normals, uvs, indices, verts, (nx, 0, nz),
                 [(u0, 1), (u1, 1), (u1, 0), (u0, 0)])

    # top +Y fan / triangle fan as quads from center? use triangle fan
    # Better: triangulate from center
    cx, cz = 0.0, 0.0
    # top: ring CCW when viewed from above — for +Y outward, CCW from above is correct for glTF
    top_center_i = None
    # add as triangle fan
    base = len(positions)
    positions.append((cx, y1, cz))
    normals.append((0, 1, 0))
    uvs.append((0.5, 0.5))
    ring_start = len(positions)
    for i, (x, z) in enumerate(ring):
        positions.append((x, y1, z))
        normals.append((0, 1, 0))
        # map xz to uv
        u = (x / w + 1) * 0.5
        v = (z / w + 1) * 0.5  # careful V
        # glTF V=0 top of image; map +Z to top of texture visually: v_img = 1 - (z/w+1)*0.5
        uvs.append((u, 1.0 - (z / w + 1) * 0.5))
    for i in range(8):
        i0 = ring_start + i
        i1 = ring_start + (i + 1) % 8
        indices.extend([base, i0, i1])

    # bottom -Y: CW when viewed from below = CCW from -Y outward
    base = len(positions)
    positions.append((cx, y0, cz))
    normals.append((0, -1, 0))
    uvs.append((0.5, 0.5))
    ring_start = len(positions)
    for i, (x, z) in enumerate(ring):
        positions.append((x, y0, z))
        normals.append((0, -1, 0))
        u = (x / w + 1) * 0.5
        uvs.append((u, 1.0 - (z / w + 1) * 0.5))
    for i in range(8):
        # reverse winding for -Y
        i0 = ring_start + i
        i1 = ring_start + (i + 1) % 8
        indices.extend([base, i1, i0])

    return positions, normals, uvs, indices

def write_meta(path: Path, block_id: str, connectable, directional=False):
    meta = {
        '$schema': '../../../schemas/scene_block.meta.schema.json',
        'id': block_id,
        'collision': True,
        'connectable': connectable,
    }
    if directional:
        meta['directional'] = True
    path.write_text(json.dumps(meta, indent=2) + '\n', encoding='utf-8')
    print('wrote', path)

# --- main ---
tex = make_marble_texture(32)
png = write_png(tex)
# also save preview next to script? keep only in glb; optionally write shared ref
tex.save('/tmp/quartz_marble.png')

# quartz cube
pos, nor, uv, idx = cube_mesh()
write_glb(ROOT / 'quartz' / 'model.glb', 'quartz', pos, nor, uv, idx, png, roughness=0.45)
write_meta(ROOT / 'quartz' / 'meta.json', 'quartz', [True]*6)

# slope
pos, nor, uv, idx = slope_mesh()
write_glb(ROOT / 'quartz_slope' / 'model.glb', 'quartz_slope', pos, nor, uv, idx, png, roughness=0.45)
# +X -X +Y -Y +Z -Z
write_meta(ROOT / 'quartz_slope' / 'meta.json', 'quartz_slope',
           [True, True, False, True, False, True], directional=True)

# pillar
pos, nor, uv, idx = pillar_mesh()
write_glb(ROOT / 'quartz_pillar' / 'model.glb', 'quartz_pillar', pos, nor, uv, idx, png, roughness=0.45)
write_meta(ROOT / 'quartz_pillar' / 'meta.json', 'quartz_pillar',
           [False, False, True, True, False, False])

print('done')
