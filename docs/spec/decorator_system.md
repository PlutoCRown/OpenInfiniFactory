# 装饰器系统

以「面能力」为核：不占格附着（漆 / 灯面板）与占格附着（印花 / 告示）。按 L0–L5 分层落地。

## 面门禁

| 宿主 | 允许条件 |
|------|----------|
| 材料 | `MaterialProps.connectable[面]` |
| 工厂 | 非 `non_connection_face` |
| 场景 | 任意面 |
| 电线 | 仅灯面板可贴 |

## 材料属性 `MaterialProps`

- `directional` / `fragile` / `is_stamp` / `connectable[6]`（局部 +X -X +Y -Y +Z -Z）
- 焊接：两端相对面皆 Connectable（`weld_materials`）
- 工厂可贴面：`BlockKind::face_attachable`

## 分层

| 层 | 内容 | 状态 |
|----|------|------|
| L0 | 面能力核；删 `MaterialFaceMark` 占位 | 已完成 |
| L1 | 脆弱碎裂回合；`Glass` 材料 | 已完成 |
| L2 | `RollerBody` / `StamperBody` 同格占位（有碰撞、无模型） | 已完成（L4 起写入 `machine_bodies`） |
| L3 | 装饰漆 + 灯面板隔断 | 已完成 |
| L4 | 印花占格附着 | 已完成 |
| L5 | 告示牌 | 已完成 |

## L2 / L4 机身占格

- `Stamper` / `Roller` 仍为 System（`no_collision`），可与材料分槽
- `StamperBody` / `RollerBody` 写入 **`machine_bodies`**（非 `blocks`），参与 `is_occupied` / 平台占用
- 因此印花材料可与机身同格共存于 `blocks` + `machine_bodies`

## L3 细节

### 装饰漆（`material_paints`）

- 键：`MaterialFace { block: BlockId, normal }` → `StampColor`
- 回合后处理：滚刷机（`MaterialLabeler::Roller`）朝向材料且该面 Connectable 时写入
- 印花机本层不刷漆（L4 生成占格印花）
- 宿主方块删除时清除；焊接保留（按 BlockId）；平移无需改写；旋转时法线随结构绕 Y 旋转
- 渲染：材料子实体复用 `face_mark` 网格与 stamp 色材质

### 灯面板（`wire_face_panels`）

- 键：`MaterialFace`（电线 BlockId + 法线），`HashSet` 存有无
- 玩法背包工具 `InventoryItem::LightPanel`：对准电线面放置 / 删除（不占邻格）
- 信号 BFS：A→邻格时若 A 在 `offset` 有面板或邻格在 `-offset` 有面板则不通
- 通电电线上面板用发光材质；视觉连接件同样被面板隔断
- 方案存档：`blocks.bin` v2 增加按格坐标+法线的面板段（加载时映射到 BlockId）

## L4 细节

### 印花材料（`MaterialKind::Stamp` / `StampMaterial`）

- `MaterialProps::STAMP`：有向、`is_stamp`、不可 Connectable
- 薄面片：`PartsOnly` 空模型 + `material_paints` 面片着色

### 附着（`material_attachments`）

- 键：子 `BlockId` → `{ parent, parent_face_normal }`（从父指向子）
- 并入 `material_structure`（与焊接一起 BFS）；宿主销毁则子印花一并删除
- 旋转时 `parent_face_normal` 随结构绕 Y 转

### 印花机阶段

- 面前宿主材料且该面（朝向机身）Connectable
- 印花生成在 **机身格**（宿主面邻格 = 印花机格）
- 该面已有印花：非脆弱 → 跳过；脆弱 → 碎旧换新

### StamperBody 透传

- 普通材料 / 滚刷机身：一律实心
- 印花：若附着法线与机身 facing 反向（工作朝向对齐）可进入机身格
- 非对齐：非脆弱阻挡；脆弱走碎裂阶段

详见 `simulation_turn_phases.md`。

## L5 细节

### 告示牌（`BlockKind::Sign`）

- 玩法工厂方块，占宿主面邻格（侧贴 / 顶立）
- 面门禁：场景任意面；工厂 `face_attachable`；材料 `material_face_connectable`
- `factory_attachments`：子工厂 BlockId → `{ parent, parent_face_normal }`；宿主销毁级联删子；结构移动时并入附着子格
- 设置：`BlockSettings::Sign`（文字与材料图标互斥）；面板可配；瞄准 billboard / 不可破坏为后续

### 后续

- 瞄准告示时的 billboard / 状态栏文字
- 不可破坏（unbreakable）层
- 印花色（`StampColor`）图标选择 UI（数据层已支持 `SignDisplay::StampColor`）
