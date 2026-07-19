"""材料方块资源生成脚本。

| 脚本 | 作用 |
|------|------|
| `generate_pixel_textures.py` | basic/iron/copper/… 32×32 像素贴图（对话定稿） |
| `generate_gypsum_texture.py` | 石膏贴图 + 斜坡 GLB 换皮 |
| `generate_face_textures.py` | material_1~4 面板风 texture+normal（无 PIL） |
| `generate_aluminum_glb.py` | material_5 Blender 倒角立方体 |

图标一律：
  `./scripts/bake_scene_icons.sh --materials-only [--only ID]`

对话里更早的 PIL 草稿见 `scripts/_history/pil_texture_24h/`。
"""
