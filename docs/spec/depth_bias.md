# 渲染 `depth_bias` 约定

共面或几乎共面的网格会 z-fighting（闪烁）。本项目用 **`depth_bias` 分层**，**不要**再靠把 mesh 放大一圈 / 沿法线浮出表面来「躲开」深度冲突（瞄准面、选区、结构线框等已按此改掉几何偏移）。

常量定义：`src/game/world/rendering/depth_bias.rs`  
改数值时先改常量与本表，再改调用处。

## 分层（`StandardMaterial` / 自定义 `Material::depth_bias`）

| 档位 | 值 | 用途 |
|------|----|------|
| （默认 / 本体） | `0` | 普通方块；`StandardMaterial` 默认，代码里一般不写常量 |
| `GOAL_GHOST` | `-1` | 验收器游玩态目标材料虚影（`GoalGhostMaterial`） |
| `PAINT` | `1` | 滚刷漆等零厚度面贴片（`face_mark_*`）；灯面板用默认 `0` |
| `OVERLAY` | `2` | 瞄准面高亮、选区/删除包围盒、结构悬停线框等叠层 |

数值越大越「压」在共面上（相对更靠前绘制）；虚影用负数，避免盖住同格真材料观感。

## Gizmo（结构悬停白框）

Bevy `GizmoConfig::depth_bias` **不是**同一套刻度：取值 `[-1, 1]`，**越负越靠前**（`-1` 总在最前，`0` 无偏移）。

| 常量 | 值 | 用途 |
|------|----|------|
| `GIZMO_OVERLAY` | `-0.0001` | 悬停结构 AABB 线框，语义对齐 `OVERLAY` |

几何上用**标准 1×1×1 格**（`max - min + 1`），**不要**再 `+ 0.06` 放大。

## 漆 / 灯面板几何

- **漆**：零厚度平面（`Plane3d`，法线 +Y，spawn 时旋转到附着法线），贴在格面（`0.5 + outset`），靠 `PAINT` bias 压过本体，**不要**做成有厚度 Cuboid（预览里会像穿心十字）。
- **灯面板**：`assets/factory_blocks/light_panel/model.glb`（`1×1×0.1`，局部 +Y 朝外、板心 `y=+0.45` 齐格面不外凸）。spawn 只旋 +Y→法线（`light_panel_transform`）；`depth_bias = 0`。未通电黑、通电白自发光。贴灯板的电线臂沿法线缩到 `0.8`。

生成器/验收器预览与世界印花均使用 `stamp_materials/<id>/model.glb`（厚 `0.1`、局部 +Z 朝宿主、板心 `+0.45`，外凸 `0.1`）。勿再程序化生成印花薄板。

## 维护检查清单

新增会与方块共面的装饰 / UI 叠层时：

1. 选上表一层，写入对应常量，勿魔法数。
2. 优先 `depth_bias`，避免法线方向「浮出」或整体 scale > 1 当主手段。
3. 自定义 `Material` 须实现 / 覆盖 `fn depth_bias(&self) -> f32`。
4. 若走 Gizmo，用 `GIZMO_OVERLAY`（或文档化新的 gizmo 档位），并注明与 `StandardMaterial` 刻度不同。
5. 漆用平面网格；灯面板用 `factory_blocks/light_panel/model.glb`；印花用 `stamp_materials/*/model.glb`，预览与世界共用。
