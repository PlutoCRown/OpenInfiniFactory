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
| L2 | `RollerBody` / `StamperBody` 同格占位（有碰撞、无模型；写入 blocks，可与宿主 System 共存） | 已完成 |
| L3 | 装饰漆 + 灯面板隔断 | 已完成 |
| L4 | 印花占格附着 | 待做 |
| L5 | 告示牌 | 待做 |

## L3 细节

### 装饰漆（`material_paints`）

- 键：`MaterialFace { block: BlockId, normal }` → `StampColor`
- 回合后处理：滚刷机（`MaterialLabeler::Roller`）朝向材料且该面 Connectable 时写入
- 印花机本层不刷漆（L4）
- 宿主方块删除时清除；焊接保留（按 BlockId）；平移无需改写；旋转时法线随结构绕 Y 旋转
- 渲染：材料子实体复用 `face_mark` 网格与 stamp 色材质

### 灯面板（`wire_face_panels`）

- 键：`MaterialFace`（电线 BlockId + 法线），`HashSet` 存有无
- 玩法背包工具 `InventoryItem::LightPanel`：对准电线面放置 / 删除（不占邻格）
- 信号 BFS：A→邻格时若 A 在 `offset` 有面板或邻格在 `-offset` 有面板则不通
- 通电电线上面板用发光材质；视觉连接件同样被面板隔断
- 方案存档：`blocks.bin` v2 增加按格坐标+法线的面板段（加载时映射到 BlockId）

详见方案讨论与 `simulation_turn_phases.md` 更新。
