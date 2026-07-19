"""Improve grass side: MC-like per-column pixel jagged fringe."""
import json, struct, math, random
from pathlib import Path
from PIL import Image

SIZE = 64
OUT = Path('assets/scene_blocks/grass')

def clamp(v, lo=0, hi=255):
    return max(lo, min(hi, int(v)))

def noise2(x, y, seed):
    n = (x * 374761393 + y * 668265263 + seed * 982451653) & 0xFFFFFFFF
    n = (n ^ (n >> 13)) * 1274126177 & 0xFFFFFFFF
    return n & 255

def shade(base, n, amount):
    d = (n - 128) * amount / 128
    return tuple(clamp(c + d) for c in base)

def gen_top():
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
    """MC grass side: dirt below, grass above, jagged 1-pixel fringe."""
    im = Image.new('RGB', (SIZE, SIZE))
    px = im.load()
    top_px = top_im.load()
    bot_px = bottom_im.load()
    rng = random.Random(7)
    # Per-column grass depth from top (in pixels). MC-ish ~1/3–1/2 of face.
    base = SIZE // 2 - 2
    heights = []
    h = base
    for x in range(SIZE):
        # random walk + occasional spikes (hanging grass / dirt notches)
        step = rng.choice([-2, -1, -1, 0, 0, 0, 1, 1, 2])
        if rng.random() < 0.12:
            step += rng.choice([-3, 3])
        h = max(SIZE // 3, min(SIZE * 2 // 3, h + step))
        heights.append(h)
    # slight neighbor blend so it isn't pure noise static
    for i in range(1, SIZE - 1):
        heights[i] = int(round(0.2 * heights[i-1] + 0.6 * heights[i] + 0.2 * heights[i+1]))

    for y in range(SIZE):
        for x in range(SIZE):
            n = noise2(x, y, 40)
            if y < heights[x]:
                # grass band — tint slightly darker near edge
                c = list(top_px[x, (y * 2) % SIZE])
                if heights[x] - y <= 2:
                    c = [clamp(c[i] * 0.85) for i in range(3)]
                px[x, y] = shade(tuple(c), n, 18)
            else:
                c = bot_px[x, y % SIZE]
                px[x, y] = shade(c, n, 14)
            # scattered grass pixels hanging 1–3 below boundary (MC fringe)
            if y >= heights[x] and y < heights[x] + 3 and noise2(x, y, 91) > 210:
                px[x, y] = shade(top_px[x, (x + y) % SIZE], n, 20)
            # dirt notches eating into grass
            if y < heights[x] and y > heights[x] - 2 and noise2(x, y, 55) > 230:
                px[x, y] = shade(bot_px[x, y % SIZE], n, 12)
    return im

top = gen_top()
bottom = gen_bottom()
side = gen_side(top, bottom)
atlas = Image.new('RGB', (SIZE, SIZE * 3))
atlas.paste(top, (0, 0))
atlas.paste(side, (0, SIZE))
atlas.paste(bottom, (0, SIZE * 2))
atlas_path = Path('/tmp/grass_atlas_new.png')
atlas.save(atlas_path)
side.save('/tmp/grass_side_jagged.png')

MIN, MAX = -0.5, 0.5

def cube_faces_uv(face_index, local):
    u, lv = local[0], local[1]
    if face_index == 4:
        v0, v1 = 0.0, 1.0 / 3.0
        return [u, v0 + lv * (v1 - v0)]
    if face_index == 5:
        v0, v1 = 2.0 / 3.0, 1.0
        return [u, v0 + lv * (v1 - v0)]
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
    local = [[0,0],[1,0],[1,1],[0,1]]
    pos=[]; nor=[]; uvs=[]; idx=[]
    for fi,(corners,n) in enumerate(faces):
        b=len(pos)
        for i,p in enumerate(corners):
            pos.append(p); nor.append(n); uvs.append(cube_faces_uv(fi, local[i]))
        idx.extend([b,b+1,b+2,b,b+2,b+3])
    return pos, nor, uvs, idx

def align4(n): return (4-(n%4))%4
def pack_f32(vals): return b''.join(struct.pack('<f', float(v)) for v in vals)

positions, normals, uvs, indices = build_cube()
pos_bin = pack_f32([c for p in positions for c in p])
nor_bin = pack_f32([c for n in normals for c in n])
uv_bin = pack_f32([c for uv in uvs for c in uv])
idx_bin = b''.join(struct.pack('<H', i) for i in indices)
png_bytes = atlas_path.read_bytes()
parts=[]; offsets={}; cursor=0
for name,data in [('pos',pos_bin),('nor',nor_bin),('uv',uv_bin),('idx',idx_bin),('img',png_bytes)]:
    pad=align4(len(data)); offsets[name]=(cursor,len(data)); parts.append(data+b'\x00'*pad); cursor+=len(data)+pad
bin_blob=b''.join(parts)
def bv(name,t):
    o,l=offsets[name]; return {"buffer":0,"byteOffset":o,"byteLength":l,"target":t}
gltf={
 "asset":{"version":"2.0","generator":"oif-bake-grass"},
 "buffers":[{"byteLength":len(bin_blob)}],
 "bufferViews":[bv('pos',34962),bv('nor',34962),bv('uv',34962),bv('idx',34963),
  {"buffer":0,"byteOffset":offsets['img'][0],"byteLength":offsets['img'][1]}],
 "accessors":[
  {"bufferView":0,"componentType":5126,"count":len(positions),"type":"VEC3","max":[MAX,MAX,MAX],"min":[MIN,MIN,MIN]},
  {"bufferView":1,"componentType":5126,"count":len(normals),"type":"VEC3"},
  {"bufferView":2,"componentType":5126,"count":len(uvs),"type":"VEC2"},
  {"bufferView":3,"componentType":5123,"count":len(indices),"type":"SCALAR"}],
 "images":[{"bufferView":4,"mimeType":"image/png"}],
 "samplers":[{"magFilter":9729,"minFilter":9729,"wrapS":10497,"wrapT":10497}],
 "textures":[{"sampler":0,"source":0}],
 "materials":[{"name":"grass","pbrMetallicRoughness":{"baseColorFactor":[1,1,1,1],"baseColorTexture":{"index":0},"metallicFactor":0,"roughnessFactor":0.96}}],
 "meshes":[{"name":"grass","primitives":[{"attributes":{"POSITION":0,"NORMAL":1,"TEXCOORD_0":2},"indices":3,"material":0}]}],
 "nodes":[{"mesh":0,"name":"grass"}],"scenes":[{"nodes":[0]}],"scene":0
}
jb=json.dumps(gltf,separators=(',',':')).encode(); jp=align4(len(jb)); jc=jb+b' '*jp
bp=align4(len(bin_blob)); bc=bin_blob+b'\x00'*bp
total=12+8+len(jc)+8+len(bc)
out=bytearray(); out+=struct.pack('<4sII',b'glTF',2,total); out+=struct.pack('<I4s',len(jc),b'JSON'); out+=jc
out+=struct.pack('<I4s',len(bc),b'BIN\x00'); out+=bc
(OUT/'model.glb').write_bytes(out)
print('ok', OUT/'model.glb', len(out))
