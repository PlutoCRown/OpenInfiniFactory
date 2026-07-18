# 新增场景方块流程

场景方块（Scene Block）是无工厂逻辑的装饰/地形块。游戏不写死种类名，只扫描资源包并注册到 catalog。外观以 `model.glb` 为准，不要在代码里再画贴图。

参考目录：`assets/scene_blocks/<id>/`  
Schema：`schemas/scene_block.meta.schema.json`

---

## 1. 建资源目录

在 `assets/scene_blocks/` 下新建以 **id** 命名的文件夹（id 规则：`^[a-z][a-z0-9_]*$`，全局唯一）。

最少需要：

```
assets/scene_blocks/<id>/
  meta.json      # 必填
  model.glb      # 必填：网格 + 材质 + 内嵌贴图
  icon.png       # 必填（用 bake 工具生成，见第 4 步）
  collision.glb  # 可选：非整格碰撞时用
```

启动时会扫描全局 `assets/scene_blocks/`；puzzle 也可自带本地包合并（重复 id 会跳过并警告）。

---

## 2. 写 `meta.json`

可挂 schema 方便编辑器提示：

```json
{
  "$schema": "../../../schemas/scene_block.meta.schema.json",
  "id": "example",
  "collision": true,
  "directional": false,
  "connectable": [true, true, true, true, true, true]
}
```

| 字段 | 含义 |
|------|------|
| `id` | 字符串 ID；UI 名走 i18n `block.<id>` |
| `collision` | `false`：整格无碰撞（忽略 `collision.glb`）。`true` 且无 `collision.glb`：单位正方体；有 `collision.glb`：玩家按该网格碰撞 |
| `directional` | `true`：可 R 旋转，存档写 facing |
| `connectable` | 局部六面是否可贴告示/装饰，顺序 **+X -X +Y -Y +Z -Z**（相对 `facing=North`）。不是焊接、不是连续贴图 |

示例约定：

- 普通实心块：六面 `true`，`collision: true`
- 面片草一类：六面 `false`，`collision: false`
- 斜坡：只在完整面/三角侧面开 connectable，并设 `directional: true`

---

## 3. 做 `model.glb`

### 坐标系

- 块中心在原点，约占 `[-0.5, 0.5]^3`（与世界格心 `grid_to_world` 对齐）
- UV：glTF 惯例，**V=0 在贴图上方**
- 若是标准六面立方体且恰好 24 个顶点 UV，加载器会抽出 `face_uvs`，世界里走带 AO 的立方体网格；非立方体（斜坡、柱、交叉面片）用 GLB 原网格

### 贴图与采样

- 贴图嵌进 GLB（不要另挂外部 PNG 作为运行时外观源）
- 像素风：sampler 的 `magFilter` / `minFilter` 设为 **NEAREST（9728）**，不要 LINEAR
- 透明：
  - 剪影草等：`alphaMode: "MASK"`（可加 `alphaCutoff`），需要双面时设 `doubleSided: true`
  - 玻璃等：`alphaMode: "BLEND"`，贴图像素自带 alpha

### 可用工具

- **Blender / 其它 DCC**：建模 → 导出 GLB，检查采样为 Closest/Nearest，透明模式正确
- **脚本生成**：用 Python 等直接写顶点与 Pillow 画小图再打包进 GLB（当前仓库里部分方块是这样生成的）

### 可选 `collision.glb`

- 仅位置网格即可（可无贴图）
- **局部坐标必须与 `model.glb` 一致**
- 有朝向的块会按 `facing` 的 Yaw 旋转碰撞网格
- 例：石英斜坡的楔形碰撞

---

## 4. 烘焙 `icon.png`

改完 `model.glb` 后执行：

```bash
./scripts/bake_scene_icons.sh --only <id>
# 或全部：
./scripts/bake_scene_icons.sh
```

- 默认输出同目录 `icon.png`，128×128
- 游戏启动**不会**再离屏渲场景图标；缺 `icon.png` 时热键栏会告警且无图
- 物品栏槽位有少量内边距，图标会居中显示；bake 时取景已尽量让方块铺满，不必在 PNG 里再留大空白

---

## 5. 补 i18n

在 `assets/i18n/zh-CN.json` 与 `assets/i18n/en.json` 增加：

| Key | 用途 |
|-----|------|
| `block.<id>` | 正式名 |
| `short.<id>` | 槽位无图时的短名（有图标时一般不显示） |
| `desc.<id>` | 悬停描述 |

未翻译时 UI 会直接显示 key 字符串。

---

## 6. 验证清单

1. 启动游戏，背包/热键栏出现新块且图标正常、不溢出
2. 放置外观与 GLB 一致（朝向块试 R）
3. 碰撞：无碰撞可穿过；有 `collision.glb` 的外形与视觉一致
4. 告示/装饰：只在 `connectable` 为 true 的面上能贴
5. 透明块：MASK/BLEND 与双面是否符合预期
6. （可选）puzzle 本地包：放在 `puzzle/assets/scene_blocks/<id>/` 再进关验证合并

---

## 7. 不需要改的代码

正常情况下 **不必** 改 Rust 注册表或写死种类名。加载路径会：

1. 扫 `meta.json` + `model.glb` → 安装模拟 `SceneBlockCatalog`
2. 读可选 `collision.glb` 三角形、可选 `icon.png`
3. 进世界时从 `model.glb` 加载网格/材质/贴图

只有扩展**通用能力**时才动代码（例如新的 meta 字段、新的碰撞语义）。

---

## 快速对照：现有例子

| id | 要点 |
|----|------|
| `stone` / `dirt` / `quartz` | 实心立方体，六面可连，整格碰撞 |
| `grass` | 草方块（地表） |
| `short_grass` | 交叉面片；无碰撞；六面不可连；MASK + 双面 |
| `glass` | 透明边框贴图；BLEND；正常碰撞与 connectable |
| `quartz_slope` | 有朝向；部分面 connectable；带 `collision.glb` |
| `quartz_pillar` | 非立方体网格；仅上下可连 |
