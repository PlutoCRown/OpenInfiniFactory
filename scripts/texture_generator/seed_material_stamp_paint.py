from pathlib import Path
from PIL import Image
import json

ROOT = Path(".")

materials = {
    "basic": ((210, 188, 118), False),
    "iron": ((158, 166, 170), False),
    "copper": ((201, 112, 58), False),
    "glass": ((168, 210, 224), True),
}
stamp_paint_colors = {
    "red": (242, 31, 26),
    "green": (51, 209, 71),
    "blue": (46, 107, 242),
    "yellow": (255, 214, 46),
}


def solid_png(path: Path, rgb, size=32):
    Image.new("RGB", (size, size), rgb).save(path, "PNG")


# material_blocks
for mid, (rgb, fragile) in materials.items():
    d = ROOT / "assets" / "material_blocks" / mid
    d.mkdir(parents=True, exist_ok=True)
    meta = {
        "$schema": "../../../schemas/material_block.meta.schema.json",
        "id": mid,
        "fragile": fragile,
        "directional": False,
        "connectable": [True, True, True, True, True, True],
    }
    (d / "meta.json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    solid_png(d / "texture.png", rgb)
    solid_png(d / "icon.png", rgb)

# stamp_materials
for sid, rgb in stamp_paint_colors.items():
    d = ROOT / "assets" / "stamp_materials" / sid
    d.mkdir(parents=True, exist_ok=True)
    meta = {
        "$schema": "../../../schemas/stamp_material.meta.schema.json",
        "id": sid,
        "fragile": False,
    }
    (d / "meta.json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    solid_png(d / "texture.png", rgb)
    solid_png(d / "icon.png", rgb)

# paint_materials
for pid, rgb in stamp_paint_colors.items():
    d = ROOT / "assets" / "paint_materials" / pid
    d.mkdir(parents=True, exist_ok=True)
    meta = {
        "$schema": "../../../schemas/paint_material.meta.schema.json",
        "id": pid,
    }
    (d / "meta.json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    solid_png(d / "texture.png", rgb)

print("assets created")
for p in sorted((ROOT / "assets").rglob("meta.json")):
    if any(
        x in str(p) for x in ("material_blocks", "stamp_materials", "paint_materials")
    ):
        print(p)
