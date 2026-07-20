"""Generate gypsum textures + icons; rebuild gypsum packs without copying quartz look."""

from pathlib import Path
from PIL import Image, ImageDraw
import struct, json, hashlib, zlib

ROOT = Path("./assets/material_blocks")


def gypsum_texture(seed: int = 1) -> Image.Image:
    """32x32 soft plaster: warm off-white with pores and faint veins — not quartz-blue."""
    rng = __import__("random").Random(seed)
    img = Image.new("RGB", (32, 32))
    px = img.load()
    # base cream plaster
    for y in range(32):
        for x in range(32):
            # soft mottling
            n = hashlib.md5(bytes([x, y, seed])).digest()[0] / 255.0
            # warm gypsum: #f2ebe0-ish with variation
            r = int(228 + n * 18 + rng.uniform(-4, 4))
            g = int(220 + n * 14 + rng.uniform(-4, 4))
            b = int(205 + n * 12 + rng.uniform(-5, 3))
            # faint horizontal plaster layers
            if y % 8 < 1:
                r -= 8
                g -= 7
                b -= 6
            px[x, y] = (max(0, min(255, r)), max(0, min(255, g)), max(0, min(255, b)))
    # scattered pores / grit
    for _ in range(48):
        x, y = rng.randrange(32), rng.randrange(32)
        shade = rng.randint(-28, -10)
        r, g, b = px[x, y]
        px[x, y] = (max(0, r + shade), max(0, g + shade), max(0, b + shade))
    # a few brighter flecks (chalk)
    for _ in range(20):
        x, y = rng.randrange(32), rng.randrange(32)
        r, g, b = px[x, y]
        px[x, y] = (min(255, r + 22), min(255, g + 20), min(255, b + 16))
    return img


def draw_cube_icon(tex: Image.Image, size=128) -> Image.Image:
    """Simple isometric cube icon using the texture colors."""
    out = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(out)
    # sample palette from texture
    colors = list(tex.getdata())
    avg = tuple(sum(c[i] for c in colors) // len(colors) for i in range(3))
    light = tuple(min(255, c + 28) for c in avg)
    mid = avg
    dark = tuple(max(0, c - 35) for c in avg)
    edge = tuple(max(0, c - 55) for c in avg)

    s = size
    # classic isometric cube points (centered)
    cx, cy = s // 2, s // 2 + 6
    w, h = 44, 26  # half-widths
    top = [(cx, cy - h - 18), (cx + w, cy - 18), (cx, cy + h - 18), (cx - w, cy - 18)]
    left = [(cx - w, cy - 18), (cx, cy + h - 18), (cx, cy + h + 22), (cx - w, cy + 22)]
    right = [(cx + w, cy - 18), (cx, cy + h - 18), (cx, cy + h + 22), (cx + w, cy + 22)]
    draw.polygon(top, fill=light + (255,))
    draw.polygon(left, fill=dark + (255,))
    draw.polygon(right, fill=mid + (255,))
    draw.line(top + [top[0]], fill=edge + (255,), width=2)
    draw.line(left + [left[0]], fill=edge + (255,), width=2)
    draw.line(right + [right[0]], fill=edge + (255,), width=2)
    return out


def draw_slope_icon(tex: Image.Image, size=128) -> Image.Image:
    """Isometric wedge/slope icon."""
    out = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(out)
    colors = list(tex.getdata())
    avg = tuple(sum(c[i] for c in colors) // len(colors) for i in range(3))
    light = tuple(min(255, c + 30) for c in avg)
    mid = avg
    dark = tuple(max(0, c - 40) for c in avg)
    edge = tuple(max(0, c - 60) for c in avg)

    cx, cy = size // 2, size // 2 + 8
    # wedge: high back-left, low front-right
    # top slant face
    top = [(cx - 40, cy - 8), (cx + 8, cy - 36), (cx + 42, cy - 6), (cx - 6, cy + 22)]
    # left vertical
    left = [(cx - 40, cy - 8), (cx - 6, cy + 22), (cx - 6, cy + 40), (cx - 40, cy + 10)]
    # front/right
    front = [
        (cx - 6, cy + 22),
        (cx + 42, cy - 6),
        (cx + 42, cy + 12),
        (cx - 6, cy + 40),
    ]
    draw.polygon(top, fill=light + (255,))
    draw.polygon(left, fill=dark + (255,))
    draw.polygon(front, fill=mid + (255,))
    for poly in (top, left, front):
        draw.line(poly + [poly[0]], fill=edge + (255,), width=2)
    return out


def png_bytes(img: Image.Image) -> bytes:
    import io

    buf = io.BytesIO()
    img.save(buf, format="PNG", optimize=True)
    return buf.getvalue()


def replace_glb_texture(glb_path: Path, new_png: bytes, out_path: Path):
    data = glb_path.read_bytes()
    assert data[:4] == b"glTF"
    json_len, json_type = struct.unpack_from("<II", data, 12)
    assert json_type == 0x4E4F534A
    json_start = 20
    js = json.loads(data[json_start : json_start + json_len])
    # find bin chunk
    bin_off = 20 + json_len
    if bin_off % 4:
        bin_off += 4 - (bin_off % 4)
    bin_len, bin_type = struct.unpack_from("<II", data, bin_off)
    assert bin_type == 0x004E4942
    old_bin = bytearray(data[bin_off + 8 : bin_off + 8 + bin_len])

    images = js["images"]
    assert len(images) == 1
    bv_index = images[0]["bufferView"]
    bv = js["bufferViews"][bv_index]
    old_img_off = bv["byteOffset"]
    old_img_len = bv["byteLength"]

    # rebuild binary: keep everything before image, new image, everything after
    before = old_bin[:old_img_off]
    after = old_bin[old_img_off + old_img_len :]
    # pad new png to 4-byte alignment for subsequent views if needed
    new_img = bytearray(new_png)
    # Update this bufferView length; shift later bufferViews
    delta = len(new_img) - old_img_len
    bv["byteLength"] = len(new_img)
    for i, view in enumerate(js["bufferViews"]):
        if i == bv_index:
            continue
        off = view.get("byteOffset", 0)
        if off > old_img_off:
            view["byteOffset"] = off + delta
    new_bin = before + new_img + after
    # pad bin to 4 bytes
    while len(new_bin) % 4:
        new_bin += b"\x00"
    js["buffers"][0]["byteLength"] = len(new_bin)

    json_bytes = json.dumps(js, separators=(",", ":")).encode("utf-8")
    while len(json_bytes) % 4:
        json_bytes += b" "

    out = bytearray()
    out += b"glTF"
    out += struct.pack("<I", 2)  # version
    total = 12 + 8 + len(json_bytes) + 8 + len(new_bin)
    out += struct.pack("<I", total)
    out += struct.pack("<II", len(json_bytes), 0x4E4F534A)
    out += json_bytes
    out += struct.pack("<II", len(new_bin), 0x004E9842 if False else 0x004E4942)
    out += new_bin
    # fix BIN magic - 0x004E4942 is 'BIN\0'
    out_path.write_bytes(bytes(out))
    print("wrote", out_path, "size", len(out), "delta", delta)


# --- gypsum (cube via texture.png) ---
gypsum_dir = ROOT / "gypsum"
tex = gypsum_texture(seed=7)
tex.save(gypsum_dir / "texture.png")
draw_cube_icon(tex).save(gypsum_dir / "icon.png")
model = gypsum_dir / "model.glb"
if model.exists():
    model.unlink()
    print("removed", model)

# --- gypsum_slope (keep mesh, new embedded texture) ---
slope_dir = ROOT / "gypsum_slope"
tex2 = gypsum_texture(seed=11)  # related but different seed
# slightly cooler pores for slope variant
tex2_path_png = png_bytes(tex2)
# source geometry from scene quartz_slope (clean), not the already-copied pack
src_glb = Path("./assets/scene_blocks/quartz_slope/model.glb")
replace_glb_texture(src_glb, tex2_path_png, slope_dir / "model.glb")
# also keep a texture.png for reference/fallback consistency? slope uses model only
draw_slope_icon(tex2).save(slope_dir / "icon.png")
# remove any accidental texture if we don't need it
print("gypsum files", list(p.name for p in gypsum_dir.iterdir()))
print("slope files", list(p.name for p in slope_dir.iterdir()))
