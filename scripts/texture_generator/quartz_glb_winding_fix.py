"""修复绕序后重写三个 quartz GLB（共用同一贴图）"""
import json, struct, math
from pathlib import Path
from io import BytesIO
from PIL import Image

ROOT = Path('assets/scene_blocks')
NEAREST, REPEAT = 9728, 10497

def write_png(img):
    buf = BytesIO(); img.save(buf, format='PNG', optimize=True); return buf.getvalue()

def make_marble_texture(size=32):
    import random
    rng = random.Random(42)
    img = Image.new('RGBA', (size, size))
    px = img.load()
    for y in range(size):
        for x in range(size):
            n = (rng.random() - 0.5) * 10
            px[x, y] = (
                int(max(0, min(255, 236 + n))),
                int(max(0, min(255, 234 + n * 0.9))),
                int(max(0, min(255, 228 + n * 0.7))),
                255,
            )
    for _ in range(5):
        x = rng.uniform(0, size); y = rng.uniform(0, size)
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
                    if dx*dx + dy*dy > thickness*thickness: continue
                    ix, iy = int(cx + dx), int(cy + dy)
                    if 0 <= ix < size and 0 <= iy < size:
                        r, g, b, a = px[ix, iy]
                        f = 0.35 if (dx == 0 and dy == 0) else 0.18
                        px[ix, iy] = (
                            int(r * (1 - f) + shade * f),
                            int(g * (1 - f) + (shade - 2) * f),
                            int(b * (1 - f) + (shade - 4) * f),
                            255,
                        )
    for _ in range(size * 2):
        x, y = rng.randrange(size), rng.randrange(size)
        r, g, b, a = px[x, y]
        px[x, y] = (min(255, r + 12), min(255, g + 12), min(255, b + 10), 255)
    return img

def f32_list(vals):
    return b''.join(struct.pack('<f', float(v)) for v in vals)
def u16_list(vals):
    return b''.join(struct.pack('<H', int(v)) for v in vals)

def write_glb(path, name, positions, normals, uvs, indices, png, roughness=0.45):
    pos_b = f32_list([c for p in positions for c in p])
    nor_b = f32_list([c for n in normals for c in n])
    uv_b = f32_list([c for u in uvs for c in u])
    blob = bytearray(); views = []
    def add_view(data, target=None):
        while len(blob) % 4: blob.append(0)
        offset = len(blob); blob.extend(data)
        while len(blob) % 4: blob.append(0)
        view = {'buffer': 0, 'byteOffset': offset, 'byteLength': len(data)}
        if target is not None: view['target'] = target
        views.append(view); return len(views) - 1
    v_idx = add_view(u16_list(indices), 34963)
    v_pos = add_view(pos_b, 34962)
    v_nor = add_view(nor_b, 34962)
    v_uv = add_view(uv_b, 34962)
    v_img = add_view(png)
    pos_min = [min(p[i] for p in positions) for i in range(3)]
    pos_max = [max(p[i] for p in positions) for i in range(3)]
    accessors = [
        {'bufferView': v_pos, 'componentType': 5126, 'count': len(positions), 'type': 'VEC3', 'max': pos_max, 'min': pos_min},
        {'bufferView': v_nor, 'componentType': 5126, 'count': len(normals), 'type': 'VEC3'},
        {'bufferView': v_uv, 'componentType': 5126, 'count': len(uvs), 'type': 'VEC2'},
        {'bufferView': v_idx, 'componentType': 5123, 'count': len(indices), 'type': 'SCALAR'},
    ]
    gltf = {
        'asset': {'version': '2.0', 'generator': 'oif-scene-block'},
        'scene': 0, 'scenes': [{'nodes': [0]}], 'nodes': [{'mesh': 0, 'name': name}],
        'meshes': [{'name': name, 'primitives': [{'attributes': {'POSITION': 0, 'NORMAL': 1, 'TEXCOORD_0': 2}, 'indices': 3, 'material': 0}]}],
        'materials': [{'name': name, 'pbrMetallicRoughness': {
            'baseColorFactor': [1,1,1,1], 'baseColorTexture': {'index': 0},
            'metallicFactor': 0.0, 'roughnessFactor': roughness}}],
        'textures': [{'sampler': 0, 'source': 0}],
        'samplers': [{'magFilter': NEAREST, 'minFilter': NEAREST, 'wrapS': REPEAT, 'wrapT': REPEAT}],
        'images': [{'bufferView': v_img, 'mimeType': 'image/png'}],
        'accessors': accessors, 'bufferViews': views, 'buffers': [{'byteLength': len(blob)}],
    }
    def pad4_space(b): return b + b' ' * ((4 - len(b) % 4) % 4)
    def pad4(b): return b + b'\x00' * ((4 - len(b) % 4) % 4)
    json_bytes = pad4_space(json.dumps(gltf, separators=(',', ':')).encode())
    bin_chunk = pad4(bytes(blob))
    total = 12 + 8 + len(json_bytes) + 8 + len(bin_chunk)
    out = bytearray()
    out += b'glTF' + struct.pack('<II', 2, total)
    out += struct.pack('<I', len(json_bytes)) + b'JSON' + json_bytes
    out += struct.pack('<I', len(bin_chunk)) + b'BIN\x00' + bin_chunk
    path.write_bytes(out)
    print('wrote', path, 'verts', len(positions))

def add_quad(positions, normals, uvs, indices, verts, normal, uv_quad):
    base = len(positions)
    for i in range(4):
        positions.append(verts[i]); normals.append(normal); uvs.append(uv_quad[i])
    indices.extend([base, base+1, base+2, base, base+2, base+3])

def add_tri(positions, normals, uvs, indices, verts, normal, uv_tri):
    base = len(positions)
    for i in range(3):
        positions.append(verts[i]); normals.append(normal); uvs.append(uv_tri[i])
    indices.extend([base, base+1, base+2])

def cube_mesh():
    # 顶点绕序与 stone 一致：从外侧看为 CCW（与法线同向）
    h = 0.5
    faces = [
        ([(h,-h,h),(h,-h,-h),(h,h,-h),(h,h,h)], (1,0,0)),   # +X
        ([(-h,-h,-h),(-h,-h,h),(-h,h,h),(-h,h,-h)], (-1,0,0)), # -X
        ([(-h,h,h),(h,h,h),(h,h,-h),(-h,h,-h)], (0,1,0)),   # +Y
        ([(-h,-h,-h),(h,-h,-h),(h,-h,h),(-h,-h,h)], (0,-1,0)), # -Y
        ([(-h,-h,h),(h,-h,h),(h,h,h),(-h,h,h)], (0,0,1)),   # +Z
        ([(h,-h,-h),(-h,-h,-h),(-h,h,-h),(h,h,-h)], (0,0,-1)), # -Z
    ]
    uv_full = [(0,1),(1,1),(1,0),(0,0)]
    positions, normals, uvs, indices = [], [], [], []
    for verts, n in faces:
        add_quad(positions, normals, uvs, indices, verts, n, uv_full)
    return positions, normals, uvs, indices

def slope_mesh():
    h = 0.5
    a = (-h, -h, -h); b = ( h, -h, -h)
    c = ( h, -h,  h); d = (-h, -h,  h)
    e = (-h,  h, -h); f = ( h,  h, -h)
    positions, normals, uvs, indices = [], [], [], []
    uv4 = [(0,1),(1,1),(1,0),(0,0)]
    uv3 = [(0,1),(1,1),(0.5,0)]
    # bottom -Y: a,b,c,d — (b-a)×(d-a)=(1,0,0)×(0,0,1)=(0,-1,0) but tri a,b,c: (b-a)×(c-a)=(1,0,0)×(1,0,1)=(0,-1,0) ✓
    # wait for CCW from -Y outside: looking along -Y from below... stone -Y was [(-h,-h,-h),(h,-h,-h),(h,-h,h),(-h,-h,h)] = a,b,c,d with a redefined
    add_quad(positions, normals, uvs, indices, [a, b, c, d], (0,-1,0), uv4)
    # back -Z: looking from -Z outside, CCW: b,f,e,a? stone -Z: [(h,-h,-h),(-h,-h,-h),(-h,h,-h),(h,h,-h)] = b,a,e,f
    # (a-b)×(e-b)=(-1,0,0)×(-1,1,0)=(0,0,-1) ✓ with order b,a,e,f
    add_quad(positions, normals, uvs, indices, [b, a, e, f], (0,0,-1), uv4)
    # +X tri: b,f,c — (f-b)×(c-b)=(0,1,0)×(0,0,1)=(1,0,0) ✓
    add_tri(positions, normals, uvs, indices, [b, f, c], (1,0,0), uv3)
    # -X tri: a,d,e — (d-a)×(e-a)=(0,0,1)×(0,1,0)=(-1,0,0) ✓
    add_tri(positions, normals, uvs, indices, [a, d, e], (-1,0,0), uv3)
    # slope: e,d,c,f with (d-e)×(c-d)=(0,1,1)
    sn = (0, 1/math.sqrt(2), 1/math.sqrt(2))
    add_quad(positions, normals, uvs, indices, [e, d, c, f], sn, [(0,0),(0,1),(1,1),(1,0)])
    return positions, normals, uvs, indices

def pillar_mesh(half=0.42, chamfer=0.08):
    w, c = half, chamfer
    ring = [
        (-w + c,  w), ( w - c,  w), ( w,  w - c), ( w, -w + c),
        ( w - c, -w), (-w + c, -w), (-w, -w + c), (-w,  w - c),
    ]
    y0, y1 = -0.5, 0.5
    positions, normals, uvs, indices = [], [], [], []
    for i in range(8):
        x0, z0 = ring[i]; x1, z1 = ring[(i + 1) % 8]
        dx, dz = x1 - x0, z1 - z0
        nx, nz = -dz, dx
        L = math.hypot(nx, nz) or 1.0
        nx, nz = nx / L, nz / L
        u0, u1 = i / 8, (i + 1) / 8
        add_quad(positions, normals, uvs, indices,
                 [(x0,y0,z0),(x1,y0,z1),(x1,y1,z1),(x0,y1,z0)], (nx,0,nz),
                 [(u0,1),(u1,1),(u1,0),(u0,0)])
    cx, cz = 0.0, 0.0
    base = len(positions)
    positions.append((cx, y1, cz)); normals.append((0,1,0)); uvs.append((0.5,0.5))
    ring_start = len(positions)
    for x, z in ring:
        positions.append((x, y1, z)); normals.append((0,1,0))
        uvs.append(((x/w+1)*0.5, 1.0 - (z/w+1)*0.5))
    for i in range(8):
        indices.extend([base, ring_start + i, ring_start + (i+1)%8])
    base = len(positions)
    positions.append((cx, y0, cz)); normals.append((0,-1,0)); uvs.append((0.5,0.5))
    ring_start = len(positions)
    for x, z in ring:
        positions.append((x, y0, z)); normals.append((0,-1,0))
        uvs.append(((x/w+1)*0.5, 1.0 - (z/w+1)*0.5))
    for i in range(8):
        indices.extend([base, ring_start + (i+1)%8, ring_start + i])
    return positions, normals, uvs, indices

png = write_png(make_marble_texture(32))
for name, mesh_fn in [('quartz', cube_mesh), ('quartz_slope', slope_mesh), ('quartz_pillar', pillar_mesh)]:
    pos, nor, uv, idx = mesh_fn()
    write_glb(ROOT/name/'model.glb', name, pos, nor, uv, idx, png)

# verify windings
import struct as S
def check(path):
    data=Path(path).read_bytes(); clen,=S.unpack_from('<I', data, 12)
    j=json.loads(data[20:20+clen].decode().rstrip(' '))
    bin_off=20+clen; blen,=S.unpack_from('<I', data, bin_off); blob=data[bin_off+8:bin_off+8+blen]
    def acc(i):
        a=j['accessors'][i]; bv=j['bufferViews'][a['bufferView']]; off=bv.get('byteOffset',0)
        n={'VEC3':3,'VEC2':2,'SCALAR':1}[a['type']]; ctype=a['componentType']
        out=[]; 
        for k in range(a['count']):
            if ctype==5126: out.append(S.unpack_from('<'+'f'*n, blob, off+k*4*n))
            else: out.append(S.unpack_from('<'+'H'*n, blob, off+k*2*n))
        return out
    pos,nor,idx=acc(0),acc(1),[x[0] for x in acc(3)]
    bad=0
    for t in range(len(idx)//3):
        i0,i1,i2=idx[t*3:(t+1)*3]; a,b,c=pos[i0],pos[i1],pos[i2]
        ux,uy,uz=b[0]-a[0],b[1]-a[1],b[2]-a[2]; vx,vy,vz=c[0]-a[0],c[1]-a[1],c[2]-a[2]
        cx,cy,cz=uy*vz-uz*vy, uz*vx-ux*vz, ux*vy-uy*vx
        L=math.sqrt(cx*cx+cy*cy+cz*cz) or 1
        sn=nor[i0]; dot=(cx/L)*sn[0]+(cy/L)*sn[1]+(cz/L)*sn[2]
        if dot < 0.5: bad += 1
    print(path, 'bad tris', bad, '/', len(idx)//3)

for name in ['quartz','quartz_slope','quartz_pillar']:
    check(f'assets/scene_blocks/{name}/model.glb')
