# 模拟回合结构检查报告

这份报告检查当前运行期一个回合的阶段划分，重点对照目标结构：

1. 重力检查
2. 信号检查
3. 移动标记
4. 统一执行移动
5. 执行行为

## 当前回合顺序

当前入口是 `src/game/simulation/runtime.rs` 的 `run_turn`。实际执行顺序如下：

| 顺序 | 当前阶段 | 主要函数 / 逻辑 | 对应目标 |
| --- | --- | --- | --- |
| 1 | 动画快照 | `animation_snapshot(world)` | 回合前记录，不属于模拟逻辑 |
| 2 | 清理生成标记 | `world.clear_generated_markers()` | 回合准备 |
| 3 | 重力移动标记 | `gravity::mark_gravity_phase(world)` | 重力检查 |
| 4 | 信号网络刷新 | `signal_cache.refresh(world)` | 信号检查 |
| 5 | 查询激活网络和受电器 | `powered_components` / `powered_devices` | 信号检查 |
| 6 | 材料移动标记 | `movement::mark_material_movement_phase(world, &powered_devices)` | 移动标记 |
| 7 | 统一执行移动 | `execute_structure_moves(world, movement_plan)` | 统一执行移动 |
| 8 | 重新生成静态 marker | `markers::run_static_marker_phase(world)` | 移动后同步生成方块 |
| 9 | 执行材料行为阶段 | `behaviors::run_material_behavior_phase(world, turn, &powered_devices)` | 执行行为 |
| 10 | 回合后信号刷新 | `signal_cache.refresh(world)` | 维护缓存 |
| 11 | 重建动画和渲染实体 | `pair_block_animations` / `rebuild_world_with_timed_animations` | 渲染同步 |

`run_material_behavior_phase` 内部顺序如下：

| 顺序 | 行为 | 主要函数 |
| --- | --- | --- |
| 1 | 材料验收 | `run_material_acceptance_phase` |
| 2 | 生成块生成材料 | `run_material_source_phase` |
| 3 | 焊接 | `run_weld_phase` |
| 4 | 钻头 / 激光删除材料 | `run_material_destroy_phase` |
| 5 | 印花机 / 滚刷机标记材料面 | `run_material_label_phase` |
| 6 | 转换器转换材料 | `run_material_conversion_phase` |
| 7 | 传送入口 / 出口传送材料 | `run_material_teleport_phase` |
| 8 | 再次材料验收 | `run_material_acceptance_phase` |

## 对目标结构的符合情况

| 目标阶段 | 当前状态 | 说明 |
| --- | --- | --- |
| 重力检查 | 已对齐 | `gravity::mark_gravity_phase` 只生成 `StructureMove`，没有立即修改世界。材料结构和工厂结构分别由 `material_gravity_moves`、`factory_gravity_moves` 计算。 |
| 信号检查 | 基本对齐 | 信号网络通过 `SignalNetworkCache` 统一刷新，然后生成激活网络和激活受电器集合。受电器是否能行动由 `powered_devices` 控制。 |
| 移动标记 | 已对齐 | 传送带、抬升器、旋转器、活塞等通过 `movement::mark_material_movement_phase` 生成移动计划。 |
| 统一执行移动 | 已对齐 | `execute_structure_moves` 统一处理平移和旋转，并做基础冲突检查。 |
| 执行行为 | 基本对齐 | 焊接、激光、印花、滚刷、转换、传送、验收都收敛在 `behaviors::run_material_behavior_phase`，并在主移动执行之后运行。 |

## 目前比较好的点

- 移动已经从“发现后立即移动”改成了 `StructureMove` 计划，再由 `execute_structure_moves` 统一执行。
- `runtime.rs` 现在只保留回合编排、仿真 tick、重置和动画快照，具体阶段已经拆到独立模块。
- `structures.rs` 集中维护结构移动、结构旋转、焊接结构查找、碰撞检查和面标记迁移，公共移动逻辑没有散落在各个系统里。
- 方块行为大多由方块侧方法声明，例如 `material_mover`、`material_source`、`material_destroyer`、`material_labeler`、`marker_behavior`，运行期主要消费行为描述。
- 信号检查已经集中到 `SignalNetworkCache`，运行期后续阶段只依赖 `powered_devices`，不需要每个系统自己遍历电线网络。
- 生成标记会在移动后重新生成，避免焊点、钻头头部、阻拦器头部停留在旧位置。

## 仍然需要注意的问题

| 问题 | 影响 | 建议 |
| --- | --- | --- |
| 传送行为内部再次调用 `execute_structure_moves` | 传送属于行为阶段，语义上合理；但它是行为阶段内的额外移动，不属于主移动计划 | 保留可以接受，但报告 / 代码注释中应明确“传送是行为移动，不参与本回合主移动冲突计划”。 |
| 移动冲突处理仍然比较简单 | `execute_structure_moves` 当前遇到源或目标已移动会跳过后续移动，但没有更复杂的优先级或全局冲突解算 | 后续如果需要更确定的规则，可以增加 `MovementPlan`，先收集所有目标，再统一解析冲突。 |
| marker 生成夹在信号检查和移动阶段之间 | 这是为了让阻拦器头部影响本回合移动，但也让阶段边界不够纯粹 | 可以把 marker 视作“信号派生碰撞层”，单独命名为 `derived_collision_phase`。 |
| 行为阶段顺序已经有规则，但还没有显式类型表达 | 后续新增行为时可能不清楚应该放在哪个行为之前或之后 | 可以定义 `BehaviorPhase` 枚举或分文件函数顺序，固定验收、生成、焊接、切削、标记、转换、传送等顺序。 |

## 当前模块结构

运行期现在已经按下面的模块边界拆分：

| 模块 | 责任 |
| --- | --- |
| `runtime.rs` | 只保留 `run_turn` 的阶段编排和渲染同步调用。 |
| `gravity.rs` | 计算材料结构和工厂结构的重力移动。 |
| `signals.rs` | 保留信号网络缓存、激活网络、激活受电器查询。 |
| `movement.rs` | 收集传送带、抬升器、活塞、旋转器等主移动计划。 |
| `structures.rs` | 保留结构查找、移动、旋转、碰撞、焊接面和印花面的迁移。 |
| `markers.rs` | 生成焊点、钻头头部、阻拦器头部等派生方块。 |
| `behaviors.rs` | 执行验收、生成、焊接、切削、印花、转换、传送等行为。 |

目标不是把逻辑拆散，而是让公共回合编排保持稳定。新增方块时，优先让方块模块声明行为描述；只有新增了真正的新阶段能力时，才修改对应阶段模块。

## 结论

当前代码已经基本符合要求的回合模型：先做重力和信号检查，再标记移动，统一执行移动，最后执行行为。模块边界也已经按这个模型拆开。后续最值得继续优化的是阶段类型表达和更明确的移动冲突规则，而不是再把更多 `is_xxx` 判断加回公共运行期。
