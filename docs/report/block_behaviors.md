# 方块行为清单

本文对应 `todo.md` 的 S02：按方块维度整理所有方块行为，标明行为散落在哪些文件中，并统计涉及文件数量。

## 行为入口

方块行为声明集中在 `src/game/world/blocks.rs` 的 `Block` / `EditableBlock` trait：

| 行为入口 | 消费位置 | 作用 |
| --- | --- | --- |
| `marker_behavior` | `src/game/simulation/markers.rs` | 生成 WeldPoint、BlockerHead、DrillHead 等派生 marker。 |
| `material_source` | `src/game/simulation/behaviors.rs` | Generator 生成材料。 |
| `material_kind` | `src/game/world/blocks/registry.rs`、`src/game/simulation/behaviors.rs` | 识别材料类型和转换输出。 |
| `default_settings` | `src/game/world/grid.rs`、各编辑 UI | 初始化 Generator、Goal、Labeler、Converter、Teleport 设置。 |
| `movement_rule` | `src/game/simulation/movement.rs`、`src/game/simulation/structures.rs` | 传送带、抬升器、旋转器、活塞类运动。 |
| `material_destroyer` | `src/game/simulation/behaviors.rs` | Drill、DrillHead、Laser 删除材料。 |
| `material_labeler` | `src/game/simulation/behaviors.rs` | Stamper、Roller 标记材料面。 |
| `weld_behavior` | `src/game/simulation/behaviors.rs` | WeldPoint 执行焊接。 |
| `signal_behavior` | `src/game/simulation/signals.rs` | Wire、Detector、PoweredDevice 信号网络。 |
| `render_behavior` | `src/game/world/rendering.rs` | Goal topper、焊接连接、线缆连接渲染。 |
| `alternate` | `src/game/systems/gameplay.rs` | 方块切换形态。 |
| `ui_panel` / `handle_edit_action` | `src/game/systems/gameplay.rs`、`src/game/systems/menus.rs`、`src/game/ui/systems/panels.rs` | 编辑面板和设置写入。 |

## 按方块清单

| 方块 | 类别 | 主要行为 | 散落文件数 | 相关文件 |
| --- | --- | --- | --- | --- |
| Grass | Scene | 基础场景方块；可编辑放置；纹理渲染。 | 3 | `blocks/catalog.rs`, `blocks/registry.rs`, `world/rendering.rs` |
| Stone | Scene | 基础场景方块；可编辑放置；纹理渲染。 | 3 | `blocks/catalog.rs`, `blocks/registry.rs`, `world/rendering.rs` |
| Dirt | Scene | 基础场景方块；可编辑放置；纹理渲染。 | 3 | `blocks/catalog.rs`, `blocks/registry.rs`, `world/rendering.rs` |
| Planks | Scene | 基础场景方块；可编辑放置；纹理渲染。 | 3 | `blocks/catalog.rs`, `blocks/registry.rs`, `world/rendering.rs` |
| Material | Material | 基础材料；可被焊接、移动、销毁、标记、验收、转换；材料结构由 welds 决定。 | 5 | `blocks/catalog.rs`, `simulation/structures.rs`, `simulation/behaviors.rs`, `world/grid.rs`, `world/rendering.rs` |
| IronMaterial | Material | 铁材料；行为同材料，材质类型用于生成、目标、转换和 UI。 | 5 | `blocks/catalog.rs`, `simulation/structures.rs`, `simulation/behaviors.rs`, `world/grid.rs`, `world/rendering.rs` |
| CopperMaterial | Material | 铜材料；行为同材料，材质类型用于生成、目标、转换和 UI。 | 5 | `blocks/catalog.rs`, `simulation/structures.rs`, `simulation/behaviors.rs`, `world/grid.rs`, `world/rendering.rs` |
| Generator | System | 按周期生成材料；有 Generator 设置和 UI；准备 pending preview。 | 6 | `blocks/generator.rs`, `blocks/registry.rs`, `world/grid.rs`, `simulation/behaviors.rs`, `simulation/runtime.rs`, `ui/systems/block_panels.rs` |
| Goal | System | 目标材料设置；目标 topper 渲染；验收相关数据入口。 | 5 | `blocks/goal.rs`, `blocks/registry.rs`, `world/grid.rs`, `world/rendering.rs`, `ui/systems/block_panels.rs` |
| Platform | Factory | 基础工厂结构方块；参与结构连接、移动、重力、渲染。 | 5 | `blocks/platform.rs`, `blocks/registry.rs`, `simulation/factory_activity.rs`, `simulation/structures.rs`, `world/rendering.rs` |
| Welder | Factory | 正面生成 WeldPoint marker；渲染焊接连接；可切换 DownWelder。 | 6 | `blocks/welder.rs`, `simulation/markers.rs`, `simulation/behaviors.rs`, `simulation/factory_activity.rs`, `world/rendering.rs`, `systems/gameplay.rs` |
| DownWelder | Factory | 向下生成 WeldPoint marker；渲染焊接连接；可切换 Welder。 | 6 | `blocks/down_welder.rs`, `simulation/markers.rs`, `simulation/behaviors.rs`, `simulation/factory_activity.rs`, `world/rendering.rs`, `systems/gameplay.rs` |
| Conveyor | Factory | 移动上方结构；参与工厂结构连接阻断规则；可切换 ReverseConveyor；定制模型。 | 6 | `blocks/conveyor.rs`, `simulation/movement.rs`, `simulation/structures.rs`, `simulation/factory_activity.rs`, `world/rendering.rs`, `systems/gameplay.rs` |
| ReverseConveyor | Factory | 移动下方结构；参与工厂结构连接阻断规则；可切换 Conveyor；定制模型。 | 6 | `blocks/reverse_conveyor.rs`, `simulation/movement.rs`, `simulation/structures.rs`, `simulation/factory_activity.rs`, `world/rendering.rs`, `systems/gameplay.rs` |
| Detector | Factory | 检测朝向格材料并供电；信号设备连接；可切换 DownDetector。 | 6 | `blocks/detector.rs`, `simulation/signals.rs`, `simulation/factory_activity.rs`, `world/rendering.rs`, `systems/gameplay.rs`, `blocks/registry.rs` |
| DownDetector | Factory | 检测下方材料并供电；信号设备连接；可切换 Detector。 | 6 | `blocks/down_detector.rs`, `simulation/signals.rs`, `simulation/factory_activity.rs`, `world/rendering.rs`, `systems/gameplay.rs`, `blocks/registry.rs` |
| Wire | Factory | 信号网络连接节点；线缆连接渲染。 | 4 | `blocks/wire.rs`, `simulation/signals.rs`, `world/rendering.rs`, `blocks/registry.rs` |
| Pusher | Factory | 通电伸出推动；收回时可能拉回绑定结构；信号设备连接；可切换 Blocker；定制动画。 | 7 | `blocks/pusher.rs`, `simulation/movement.rs`, `simulation/structures.rs`, `simulation/signals.rs`, `simulation/factory_activity.rs`, `world/rendering.rs`, `systems/gameplay.rs` |
| Blocker | Factory | 反向电控伸缩；生成 BlockerHead marker；信号设备连接；可切换 Pusher。 | 7 | `blocks/blocker.rs`, `simulation/movement.rs`, `simulation/markers.rs`, `simulation/signals.rs`, `simulation/factory_activity.rs`, `world/rendering.rs`, `systems/gameplay.rs` |
| Lifter | Factory | 向上搜索并抬升材料或可推动工厂结构；支持结构免下落。 | 5 | `blocks/lifter.rs`, `simulation/movement.rs`, `simulation/structures.rs`, `simulation/factory_activity.rs`, `world/rendering.rs` |
| Rotator | Factory | 顺时针旋转上方材料结构；可切换 CounterRotator。 | 5 | `blocks/rotator.rs`, `simulation/movement.rs`, `simulation/structures.rs`, `world/rendering.rs`, `systems/gameplay.rs` |
| CounterRotator | Factory | 逆时针旋转上方材料结构；可切换 Rotator。 | 5 | `blocks/counter_rotator.rs`, `simulation/movement.rs`, `simulation/structures.rs`, `world/rendering.rs`, `systems/gameplay.rs` |
| Drill | Factory | 正面生成 DrillHead marker；通电销毁正面材料；信号设备连接；可切换 Laser；定制模型。 | 7 | `blocks/drill.rs`, `simulation/markers.rs`, `simulation/behaviors.rs`, `simulation/signals.rs`, `world/rendering.rs`, `systems/gameplay.rs`, `blocks/drill_head.rs` |
| Laser | Factory | 通电沿朝向远距离销毁材料；信号设备连接；可切换 Drill。 | 5 | `blocks/laser.rs`, `simulation/behaviors.rs`, `simulation/signals.rs`, `world/rendering.rs`, `systems/gameplay.rs` |
| Stamper | System | 标记目标材料面；有颜色设置和 Labeler UI。 | 5 | `blocks/stamper.rs`, `simulation/behaviors.rs`, `world/grid.rs`, `ui/systems/block_panels.rs`, `world/rendering.rs` |
| Roller | System | 标记目标材料面；优先级低于 Stamper 旧标记；有颜色设置和 Labeler UI。 | 5 | `blocks/roller.rs`, `simulation/behaviors.rs`, `world/grid.rs`, `ui/systems/block_panels.rs`, `world/rendering.rs` |
| Converter | System | 转换所在格材料类型；有输入 / 输出设置和 UI。 | 5 | `blocks/converter.rs`, `simulation/behaviors.rs`, `world/grid.rs`, `ui/systems/block_panels.rs`, `world/rendering.rs` |
| TeleportEntrance | System | 入口配对设置；行为阶段传送入口材料；有 Teleport UI。 | 5 | `blocks/teleport_entrance.rs`, `simulation/behaviors.rs`, `world/grid.rs`, `ui/systems/block_panels.rs`, `world/rendering.rs` |
| TeleportExit | System | 出口配对设置；作为入口传送目标；有 Teleport UI。 | 5 | `blocks/teleport_exit.rs`, `simulation/behaviors.rs`, `world/grid.rs`, `ui/systems/block_panels.rs`, `world/rendering.rs` |
| WeldPoint | Virtual | 焊接 marker；运行焊接行为；渲染焊接连接。 | 5 | `blocks/weld_point.rs`, `simulation/markers.rs`, `simulation/behaviors.rs`, `world/rendering.rs`, `world/grid.rs` |
| BlockerHead | Virtual | 阻拦器派生头部 marker；提供占用 / 碰撞表现。 | 4 | `blocks/blocker_head.rs`, `simulation/markers.rs`, `simulation/movement.rs`, `world/rendering.rs` |
| DrillHead | Virtual | 钻头派生头部 marker；销毁邻接材料；透明无碰撞。 | 4 | `blocks/drill_head.rs`, `simulation/markers.rs`, `simulation/behaviors.rs`, `world/rendering.rs` |

## 散落模式

### 1. 行为声明集中，行为执行分散

方块模块通常只声明行为枚举，例如 `MovementRule`、`SignalBehavior`、`MaterialDestroyer`。真正执行分散在：

- `simulation/movement.rs`
- `simulation/structures.rs`
- `simulation/signals.rs`
- `simulation/markers.rs`
- `simulation/behaviors.rs`
- `world/rendering.rs`

这让新增方块时需要同时理解声明枚举和多个运行期阶段。

### 2. 工厂结构连接规则单独维护

`factory_activity.rs` 中有硬编码的连接阻断规则：

- Detector / Drill / Welder 的朝向面阻断连接。
- DownDetector / DownWelder 的下方面阻断连接。
- Lifter / Conveyor 的上方面阻断连接。
- ReverseConveyor 的下方面阻断连接。

这些规则和方块模块里的 `marker_behavior`、`movement_rule` 语义相关，但目前没有由方块自身声明。

### 3. 编辑 UI 与方块设置绑定较紧

Generator、Goal、Stamper、Roller、Converter、TeleportEntrance、TeleportExit 通过 `EditableBlock` 暴露 UI 面板，但具体 UI 渲染在 `ui/systems/block_panels.rs`。新增可编辑方块通常要同时改：

- 方块模块。
- `BlockSettings`。
- UI 面板。
- 存档序列化结构。

### 4. Alternate 切换是方块对之间的硬链接

当前 alternate 关系包括：

- Welder <-> DownWelder
- Conveyor <-> ReverseConveyor
- Detector <-> DownDetector
- Pusher <-> Blocker
- Rotator <-> CounterRotator
- Drill <-> Laser

切换入口在 `systems/gameplay.rs`，方块模块只声明目标类型。

## 后续抽象建议

1. 先把结构连接阻断能力纳入方块声明，例如 `factory_connection_blockers(facing)`。
2. 把移动、信号、marker、行为执行继续保留分阶段，但使用统一的方块能力描述生成阶段输入。
3. 对设置类方块建立 `BlockSettingsDescriptor`，减少 UI、默认设置、存档之间的重复映射。
4. 对 Virtual marker 明确生命周期和来源，避免运行期 marker 行为散落在源方块和 marker 方块两处。
5. 把 alternate 关系集中成可验证表，启动时检查互反关系。

