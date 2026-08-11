# 装饰器系统

以「面能力」为核：不占格附着（漆 / 印花 / 灯面板）与占格附着（告示）。按 L0–L5 分层落地。

## 面门禁

| 宿主 | 允许条件 |
|------|----------|
| 材料 | catalog `connectable[面]`（`MaterialProps`） |
| 工厂 | 非 `non_connection_face` |
| 场景 | catalog `connectable[面]` |
| 电线 | 仅灯面板可贴 |

## 材料属性 `MaterialProps`

- 来自 `material_blocks` / `stamp_materials` 资源包：`directional` / `fragile` / `is_stamp` / `connectable[6]`
- 焊接：两端相对面皆 Connectable（`weld_materials`）
- 滚刷 / 印花：目标面 Connectable
- 工厂可贴面：`BlockKind::face_attachable`

印花与滚刷**不是颜色**：分别为 `StampMaterialId` / `PaintMaterialId`（见 `docs/report/add_material_block.md`）。

## 分层

| 层 | 内容 | 状态 |
|----|------|------|
| L0 | 面能力核；删 `MaterialFaceMark` 占位 | 已完成 |
| L1 | 脆弱碎裂回合；`Glass` 材料 | 已完成 |
| L2 | 滚刷机 / 印花机作为普通工厂方块参与碰撞 | 已完成 |
| L3 | 装饰漆 + 灯面板隔断 | 已完成 |
| L4 | 印花面附着与凸出碰撞 | 已完成 |
| L5 | 告示牌 | 已完成 |

## 滚刷机 / 印花机层级

- `Stamper` / `Roller` 是普通 Factory，写入 `blocks`，因此玩家和材料都会正常碰撞
- `Converter` 保留为无碰撞 System，因为它必须与待转换材料同格

## L3 细节

### 装饰漆（`material_paints`）

- 键：`MaterialFace { block: BlockId, normal }` → `PaintMaterialId`
- 回合后处理：滚刷机朝向材料且该面 Connectable 时写入
- 外观：`paint_materials/<id>/texture.png` 贴到面片
- 宿主方块删除时清除；焊接保留；旋转时法线随结构绕 Y 旋转

### 灯面板（`wire_face_panels`）

- 键：`MaterialFace`（电线 BlockId + 法线），`HashSet` 存有无
- 玩法背包工具 `InventoryItem::LightPanel`：对准电线面放置 / 删除（不占邻格）
- 外观：`factory_blocks/light_panel/model.glb`（齐格面）；通电切换白自发光材质；贴板面的电线臂缩到 0.8
- 信号 BFS：A→邻格时若 A 在 `offset` 有面板或邻格在 `-offset` 有面板则不通
- 方案存档：`blocks.bin` v2 增加按格坐标+法线的面板段（加载时映射到 BlockId）

## L4 细节

### 印花材料（`BlockKind::Stamp(StampMaterialId)`）

- 资源包：`assets/stamp_materials/<id>/`（有厚度模型或 texture 立方体 fallback）
- `MaterialProps`：`is_stamp`、不可 Connectable
- 附着：`material_stamps`；外观由 Stamp 资源包决定（不再使用色片面片表）

### 附着（`material_stamps`）

- 键：`MaterialFace { parent: BlockId, normal }` → `StampMaterialId`
- 印花只作为宿主面的附着，不写入 `blocks`，也不占用印花所在格
- 宿主销毁时清除；不参与 `material_structure` BFS
- 旋转时法线随宿主结构绕 Y 转
- 结构移动时，额外检查印花沿法线凸出的一格；撞到非脆弱方块则阻止移动，脆弱印花则碎裂后允许移动

### 印花机阶段

- 面前宿主材料且该面（朝向机身）Connectable
- 印花生成在宿主的工作面上；宿主面邻格只是印花的视觉凸出位置，不是实际占格
- 该面已有印花：非脆弱 → 跳过；脆弱 → 碎旧换新

详见 `simulation_turn_phases.md`。

## L5 细节

### 告示牌（`BlockKind::Sign`）

- 玩法工厂方块，占宿主面邻格（侧贴 / 顶立）
- **玩家碰撞** `has_collision=false`：可穿行
- **模拟占用** blocks 层有块即占格（`is_occupied`）：挡材料下落/推动；非脆弱、不会被挤占覆盖
- 面门禁：场景任意面；工厂 `face_attachable`；材料 `material_face_connectable`
- `factory_attachments`：子工厂 BlockId → `{ parent, parent_face_normal }`；宿主销毁级联删子；结构移动时并入附着子格
- **有向附着物通用规则**（`attaches_to_factory_face && is_directional`）：朝向由贴面法线决定（侧贴映射 N/E/S/W，顶/底固定 North），不跟玩家放置朝向，不可 R 手转
- 设置：`BlockSettings::Sign`（文字与材料图标互斥）；面板可配；瞄准 billboard / 不可破坏为后续

### 后续

- 瞄准告示时的 billboard / 状态栏文字
- 不可破坏（unbreakable）层
- 告示牌选用印花材料图标（`SignDisplay::Stamp`）
