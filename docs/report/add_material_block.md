# 新增材料 / 印花 / 滚刷资源流程

材料相关种类不写死在 Rust 里，只扫描资源包并注册到 catalog（对齐场景方块）。  
它们**不是颜色枚举**：印花机 / 滚刷机选的是资源包 id。

| 类型 | 目录 | Schema | 运行时角色 |
|------|------|--------|------------|
| 普通材料 | [`assets/material_blocks/<id>/`](../../assets/material_blocks/) | [`schemas/material_block.meta.schema.json`](../../schemas/material_block.meta.schema.json) | `BlockKind::Material(id)`；生成器 / 验收 / 转换器可选 |
| 印花材料 | [`assets/stamp_materials/<id>/`](../../assets/stamp_materials/) | [`schemas/stamp_material.meta.schema.json`](../../schemas/stamp_material.meta.schema.json) | `BlockKind::Stamp(id)`；印花机生成，占格附着 |
| 滚刷材料 | [`assets/paint_materials/<id>/`](../../assets/paint_materials/) | [`schemas/paint_material.meta.schema.json`](../../schemas/paint_material.meta.schema.json) | `PaintMaterialId`；滚刷写到材料面，不占格 |

id 规则：`^[a-z][a-z0-9_]*$`。  
普通材料与场景方块共用 `block.<id>` 名空间，**全局勿撞名**（场景已有 `glass` / `stone`，材料用 `glass_material` / `granite`）。

---

## A. 普通材料 `material_blocks`

参与焊接、重力、生成器 / 验收的可加工方块。

### A1. 建目录与文件

```
assets/material_blocks/<id>/
  meta.json      # 必填：逻辑元数据
  model.glb      # 优先：自定义网格 + 内嵌贴图
  texture.png    # 无 model.glb 时必填：贴到标准单位立方体
  icon.png       # 必填：UI 图标（用 bake 工具生成）
```

| 文件 | 必填？ | 定义 |
|------|--------|------|
| `meta.json` | 是 | id、脆弱、朝向、六面 connectable |
| `model.glb` | 二选一（与 texture） | 外观网格；有则优先于 `texture.png` |
| `texture.png` | 二选一（与 model） | 无 GLB 时贴到单位立方体；建议 32×32 像素风，NEAREST |
| `icon.png` | 是 | 128×128 预烘焙图标；启动不再离屏渲 |

加载规则：有 `model.glb` → GLB；否则有 `texture.png` → 立方体贴图；两者都无 → 报错跳过。

### A2. 写 `meta.json`

```json
{
  "$schema": "../../../schemas/material_block.meta.schema.json",
  "id": "example",
  "fragile": false,
  "directional": false,
  "connectable": [true, true, true, true, true, true]
}
```

| 字段 | 含义 |
|------|------|
| `id` | 字符串 ID；须与文件夹名一致；i18n 走 `block.<id>` |
| `fragile` | `true`：运动冲突时碎裂（如玻璃、水晶），而非挡住对方 |
| `directional` | `true`：可 R 旋转，存档写 facing（当前种子均 `false`） |
| `connectable` | 局部六面是否可**焊接 / 滚刷 / 印花 / 告示贴材料面**，顺序 **+X -X +Y -Y +Z -Z**（相对 `facing=North`） |

Connectable 规则（重要）：

- 焊接：两端相对面都为 `true` 才成功；任一面 `false` 则不焊
- 滚刷 / 印花：目标材料朝向机器的那一面必须为 `true`
- 告示贴到材料面时同样检查该面

### A3. 外观

- **有 `model.glb`**：坐标系与场景方块相同（块心原点，约 `[-0.5, 0.5]^3`）；贴图嵌进 GLB；像素风用 NEAREST
- **仅 `texture.png`**：引擎用标准单位立方体 + 该贴图（六面同图）；采样 Nearest + Repeat

### A4. 烘焙 `icon.png`

与场景方块**同一套**离屏相机（勿用手写等距图）：

```bash
./scripts/bake_scene_icons.sh --materials-only
./scripts/bake_scene_icons.sh --only <id>
```

### A5. 补 i18n

在 `assets/i18n/zh-CN.json` 与 `en.json`：

| Key | 用途 |
|-----|------|
| `block.<id>` | 正式名 |
| `short.<id>` | 槽位无图时的短名 |
| `desc.<id>` | 悬停描述 |

### A6. 验证清单

1. 启动后生成器 / 验收 / 转换器可选到新材料，图标正常
2. 放置外观与 texture / GLB 一致
3. connectable 为 `false` 的面：不能焊、不能刷、不能印
4. `fragile: true` 时被推挤会碎
5. 未翻译时 UI 显示 key 字符串

---

## B. 印花材料 `stamp_materials`

印花机印出的占格附着块。有厚度，**不是纯色面片**。不进入生成器 / 验收列表。

### B1. 建目录与文件

```
assets/stamp_materials/<id>/
  meta.json      # 必填
  model.glb      # 优先：有厚度薄板等
  texture.png    # 无 model.glb 时必填
  icon.png       # 必填（目前可手绘/复制贴图；后续可扩展 bake）
```

| 文件 | 必填？ | 定义 |
|------|--------|------|
| `meta.json` | 是 | id、fragile |
| `model.glb` | 二选一 | 有厚度模型；建议局部 **+Z 朝外**，主体贴靠 **-Z（宿主侧）** |
| `texture.png` | 二选一 | 无 GLB 时用引擎内置**告示牌式薄板**（约 0.78×0.72×0.06，贴靠宿主）+ 该贴图 |
| `icon.png` | 是 | UI 选型图标 |

### B2. 写 `meta.json`

```json
{
  "$schema": "../../../schemas/stamp_material.meta.schema.json",
  "id": "red",
  "fragile": false
}
```

| 字段 | 含义 |
|------|------|
| `id` | 字符串 ID；i18n 走 `stamp.<id>` |
| `fragile` | 可选；宿主面已有印花时，脆弱印花可被替换 |

固有属性（写死在代码，不必进 meta）：

- `is_stamp = true`
- 六面不可 Connectable（不能再当焊 / 刷 / 印的宿主）

### B3. 补 i18n

| Key | 用途 |
|-----|------|
| `stamp.<id>` | 正式名 |
| `short.stamp.<id>` | 短名 |
| `desc.stamp.<id>` | 描述 |

### B4. 验证清单

1. 印花机设置里能选到该包
2. 通电印花后生成占格印花，贴在宿主面上，有厚度
3. 印花本身不能被焊接 / 滚刷 / 再印花

---

## C. 滚刷材料 `paint_materials`

滚刷机刷到材料面上的 **2D 贴图**，不占邻格、不需要模型。

### C1. 建目录与文件

```
assets/paint_materials/<id>/
  meta.json      # 必填（目前主要写 id，预留扩展）
  texture.png    # 必填：刷到材料面上的贴图
```

| 文件 | 必填？ | 定义 |
|------|--------|------|
| `meta.json` | 是 | 目前仅 `id`；以后可加其它字段而不改目录结构 |
| `texture.png` | 是 | 面片贴图；建议像素风；UI 选型可直接用此图作预览 |

**不要**放 `model.glb` / `icon.png`（当前加载器不要求；有也不当外观模型用）。

### C2. 写 `meta.json`

```json
{
  "$schema": "../../../schemas/paint_material.meta.schema.json",
  "id": "red"
}
```

### C3. 补 i18n

| Key | 用途 |
|-----|------|
| `paint.<id>` | 正式名 |
| `short.paint.<id>` | 短名 |
| `desc.paint.<id>` | 描述 |

### C4. 验证清单

1. 滚刷机设置里能选到该包
2. 朝向材料且该面 connectable 时，面上出现贴图
3. 材料删除后面漆消失

---

## 外观 fallback 总表

| 类型 | 有 `model.glb` | 仅 `texture.png` | 都没有 |
|------|----------------|------------------|--------|
| 场景 / 普通材料 | 用 GLB | 单位立方体 + 贴图 | 报错 |
| 印花 | 用 GLB | 告示牌式薄板 + 贴图 | 报错 |
| 滚刷 | （忽略） | 面片贴图 | 报错 |

---

## 不需要改的代码

正常加包只需：资源目录 + i18n（材料再 bake icon）。加载会：

1. 扫三套目录 → 安装对应 Catalog + 表现注册表  
2. `BlockKind::Material(id)` / `Stamp(id)`；`material_paints → PaintMaterialId`  
3. 存档用**字符串 id**（`blocks.bin` v5）

只有扩展通用能力时才动 Rust（新 meta 字段、新碰撞语义等）。

---

## 快速对照：现有种子

### 普通材料

| id | 要点 |
|----|------|
| `basic` | 陶土砖纹 |
| `iron` | 拉丝钢 |
| `copper` | 拉丝铜 + 铜绿 |
| `glass_material` | 玻璃面板；`fragile`；避开场景 `glass` |
| `gold` | 金色拉丝金属 |
| `aluminum` | 冷白拉丝铝 |
| `wood` | 木纹 |
| `granite` | 灰岩；避开场景 `stone` |
| `coal` | 深色煤炭 |
| `crystal` | 紫青水晶；`fragile` |

### 印花 / 滚刷

| 目录 | 当前种子 |
|------|----------|
| `stamp_materials` | `red` / `green` / `blue` / `yellow` |
| `paint_materials` | `red` / `green` / `blue` / `yellow` |

---

## 和场景方块文档的关系

场景方块流程见 [`add_scene_block.md`](add_scene_block.md)。  
三者共用：id 规则、`model.glb` 坐标系约定、`texture.png` 立方体 fallback、同一 bake 相机参数。  
材料特有：`fragile` / 焊接门禁用的 `connectable`、印花薄板 fallback、滚刷纯 2D 贴图。
