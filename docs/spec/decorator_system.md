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
| L0 | 面能力核；删 `MaterialFaceMark` 占位 | 进行中 |
| L1 | 脆弱碎裂回合；`Glass` 材料 | 已完成 |
| L2 | `RollerBody` / `StamperBody` | 待做 |
| L3 | 装饰漆 + 灯面板隔断 | 待做 |
| L4 | 印花占格附着 | 待做 |
| L5 | 告示牌 | 待做 |

详见方案讨论与 `simulation_turn_phases.md` 更新。
