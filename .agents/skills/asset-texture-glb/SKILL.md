---
name: asset-texture-glb
description: >-
  OpenInfiniFactory 资源外观：用 Python 脚本生成/改写 texture.png、model.glb、
  icon.png（材料/场景/印花/工厂块）。新建或改 GLB 模型默认用 Blender bpy
  （倒角/布尔/多物件）。用户提到 Texture、贴图、PNG、GLB、Icon、图标、
  材质外观、Blender、bpy，或编辑 assets/**/texture.png|model.glb|icon.png
  时使用。禁止手绘 icon；icon 一律 bake 脚本。
---

# 资源 Texture / PNG / GLB / Icon

## 何时启用

- 新建或改 `texture.png` / `model.glb` / `icon.png`
- 用户说 Texture、贴图、PNG、GLB、Icon、图标、材质外观、Blender、bpy
- 目录在 `assets/material_blocks|scene_blocks|stamp_materials|paint_materials|factory_blocks/`

**不要**为「加一种外观」去改 Rust catalog；扫描资源包即可。模拟逻辑问题走 `sim-debug-http`。

---

## 硬规则（必须遵守）

1. **写文件用可复用 Python 脚本**，落到 `scripts/`；禁止只跑一次就丢的 heredoc。
2. **新建 / 大改 `model.glb` → 默认用 Blender `bpy`**（倒角、布尔、多物件合成很常见）。仿写 `scripts/factory_blocks/generate_*.py` 或 `scripts/material_blocks/generate_aluminum_glb.py`。
3. **仅改贴图、网格不动**：可用纯 Python 换 GLB 内嵌 PNG（见 `generate_gypsum_texture.py`），或只写 `texture.png`。
4. **`icon.png` 禁止手绘**。外观就绪后跑 bake。
5. 有视觉：可用 Read 看 PNG。无视觉：用 PIL 查尺寸/像素。
6. `id`：`^[a-z][a-z0-9_]*$`，目录名 = `meta.json` 的 `id`；材料勿与场景撞名（`glass`→`glass_material`）。

---

## 模型怎么做：bpy 优先

| 情况 | 做法 |
|------|------|
| 新网格、倒角、布尔、多物件、多材质 | **`bpy`** + `bpy.ops.export_scene.gltf` |
| 单位立方体 / 只要六面同图 | 只写 `texture.png`，不要假 GLB |
| 已有 GLB，只换皮 | 纯 Python 替换内嵌 PNG，或 bpy 重导 |

### bpy 脚本约定（对齐工厂块）

```text
/Applications/Blender.app/Contents/MacOS/Blender --background \
  --python scripts/<area>/generate_<id>_glb.py
```

- `ROOT = Path(__file__).resolve().parents[2]`
- 清空场景 → 建 mesh/mat →（可选）join → `export_scene.gltf(..., export_format="GLB", export_yup=True, export_apply=True)`
- **坐标系**：Blender Z-up；`export_yup=True` 后与游戏一致。块心原点，约 `[-0.5, 0.5]^3`。工厂块前进方向惯例见各 `generate_*_glb.py` 文件头注释。
- 贴图：嵌入 GLB；有 `model.glb` 后通常**删掉**外部临时 `texture.png` / bake 中间图（铝块脚本即如此）。
- 像素风：导出后确认 sampler 为 NEAREST，或在引擎侧已按 NEAREST 加载（见 `load_scene_glb`）。

模板优先读：

- `scripts/factory_blocks/generate_pusher_glb.py`（多部件 + 导出）
- `scripts/factory_blocks/generate_conveyor_glb.py`（布尔 / 贴图）
- `scripts/material_blocks/generate_aluminum_glb.py`（倒角立方 + 烤贴图进 GLB）

**不要**为「可能有倒角」的新模型去手写 glTF 顶点；历史场景脚本（quartz/grass）是例外遗产，新工作默认 bpy。

---

## 资源包规律

| 类型 | 目录 | 外观优先级 | icon |
|------|------|------------|------|
| 材料 | `assets/material_blocks/<id>/` | `model.glb` ≻ `texture.png` | bake `--materials-only` |
| 场景 | `assets/scene_blocks/<id>/` | 同上；可选 `collision.glb` | bake `--scene-only` |
| 印花 | `assets/stamp_materials/<id>/` | **必须** `model.glb` | bake `--stamps-only` |
| 滚刷 | `assets/paint_materials/<id>/` | **仅** `texture.png` | texture 可作预览 |
| 工厂 | `assets/factory_blocks/<id>/` | `model.glb`（bpy） | 按块惯例 |

- **`texture.png`**：建议 32×32 像素风。
- **`icon.png`**：128×128，bake 生成。
- **`meta.json`**：逻辑字段；新包补 i18n（`docs/report/add_material_block.md` / `add_scene_block.md`）。

加载口诀：**有 GLB 用 GLB；否则 texture 贴单位立方体；两者都无报错跳过。**

---

## 现成脚本

### bpy → GLB（创建模型首选）

`scripts/factory_blocks/generate_*.py`、`scripts/material_blocks/generate_aluminum_glb.py`

### 贴图 / 换皮（PIL 或纯 Python）

| 脚本 | 用途 |
|------|------|
| `scripts/material_blocks/generate_pixel_textures.py` | 多种 32×32 像素贴图 |
| `scripts/material_blocks/generate_gypsum_texture.py` | 立方体贴图 + 斜坡 GLB 换 PNG |
| `scripts/material_blocks/generate_face_textures.py` | material_1~4 面板 texture+normal |

### 遗产：手写 glTF（只维护，新模型勿仿）

`scripts/scene_blocks/generate_{quartz,grass,short_grass_and_glass}.py`、`scripts/stamp_materials/generate_stamp_glb.py`

### Icon

```bash
./scripts/bake_scene_icons.sh --materials-only --only <id>
./scripts/bake_scene_icons.sh --scene-only --only <id>
./scripts/bake_scene_icons.sh --stamps-only --only <id>
```

---

## 工作流

```
1. 定类型与 id → assets/.../<id>/
2. 要网格？ → 仿写 bpy generate_*_glb.py → Blender --background --python …
   只要立方体贴图？ → generate_*_texture / PIL 写 texture.png
3. （新包）meta.json + i18n
4. bake icon
5. 抽查 PNG / 进游戏看一眼
```

### 只改贴图、保留网格

`generate_gypsum_texture.py` 式换内嵌 PNG，或 bpy 改材质后重导。

### 新斜坡 / 异形 / 机器

**bpy** 建模导出；坐标系与现有块一致。

---

## 贴图脚本写法（非 bpy）

```python
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "assets" / "material_blocks" / "<id>"
```

确定性 seed；不要在脚本里画 isometric icon。

---

## 反模式

- 新模型用手写 glTF「省事」（遇到倒角/布尔就崩）
- 手绘 `icon.png`
- 绝对路径、一次性 heredoc 不落盘
- paint 包加 GLB / stamp 无 GLB
- 改完外观不 bake
