# 模拟回合阶段（三层世界）

运行期入口为 `src/game/simulation/core.rs` 的 `simulate_turn`。模拟使用 **solution / turn / realtime** 三层世界：

| 层 | 含义 | 生命周期 |
| --- | --- | --- |
| `solution` | 模拟开始时的工厂布局（冻结） | 模拟开始写入 `SimSnapshot`，回滚时恢复 |
| `turn` | 当前已提交回合世界 | worker / 客户端每回合消费 |
| `realtime` | 运动执行 scratch | 每回合 `turn.clone()`，execute 后写回 `turn` |

工厂连通判定（活塞推工厂子集）读 **solution** 侧结构图；材料推/落等读 **turn** 侧结构图。

## 回合顺序

| 顺序 | 阶段 | 主要函数 |
| --- | --- | --- |
| 1 | 生成器落地 | `place_ready_generated_materials` |
| 2 | 信号检测 | `signal_cache.refresh` → `powered_devices` / `render_powered_wires` |
| 3 | 六类运动标记 | `collect_movement_plan`（只读 turn，连通读 solution） |
| 4 | 运动执行 | `realtime = turn.clone()` → `execute_movement_plan` → `turn = realtime` |
| 5 | 结构位置刷新 | `turn_structures.move_positions` / `refresh_material_structures` |
| 6 | 静态 marker | `run_static_marker_phase` |
| 7 | 通电 marker | `run_powered_marker_phase`（BlockerHead 等） |
| 8 | 行为阶段 | `run_material_behavior_phase`（直接改 turn，无跨回合 pending） |
| 9 | 下回合生成预约 | `prepare_upcoming_generation` |
| 10 | 信号缓存刷新 | `signal_cache.refresh` |

## 运动：标记顺序与执行

标记按设备 `(x, y, z)` 升序遍历；同类候选按 **source 坐标** 升序。标记阶段**不**改 `PusherState.extended`。

| Phase | 标记 | 说明 |
| --- | --- | --- |
| 1 | Fixed | 预留（吸盘未实现） |
| 2 | Rotate | 旋转器 |
| 3 | Push | 活塞 / 阻拦器 / 通电平移 |
| 4 | Lift | 抬升器 |
| 5 | Conveyor | 传送带运输（推下方结构） |
| 6 | Gravity | 材料与工厂重力 |
| 7 | Conveyor | 反向传送带反推自身（fallback，本回合已因重力移动则跳过） |

执行在 **realtime** 上逐 phase、逐 candidate 尝试 primary → fallbacks；成功则写 realtime 并更新 `PusherState.extended`（extend/retract 在 Push execute 成功时）。

## 结构运动 history（旋转 / 传送带）

`MovementInfluenceCache` 在 `SimSnapshot` 中跨回合保留：

| 设备 | 行为 |
| --- | --- |
| **旋转器** | 结构被某旋转器成功旋转后，该旋转器的标记被忽略，直到某回合该旋转器不再对该结构打标记（结构离开工作范围） |
| **传送带** | 记录对本结构打标过的传送带；本回合仍打标的老传送带候选优先级降低，新接触的传送带优先；离开传送带后 history 清除 |

## 行为阶段顺序

`run_material_behavior_phase` 内部顺序（运动后，直接改 turn）：

| 顺序 | 行为 |
| --- | --- |
| 1 | 滚刷 / 印花 |
| 2 | 转换器 |
| 3 | 焊接 |
| 4 | 钻头 / 激光（立刻删除） |
| 5 | 传送（同回合立刻执行） |
| 6 | 验收（立刻删材料并计数） |

`PendingGeneratedMaterials` 仅保留**生成器预约**。

## 模块职责

| 模块 | 责任 |
| --- | --- |
| `worlds.rs` | `SimulationWorlds` 与模拟开始/快照组装 |
| `core.rs` | `simulate_turn` 编排 |
| `movement_plan.rs` | `collect_movement_plan` / `execute_movement_plan` |
| `movement.rs` | 设备候选收集、`PusherState` |
| `structures.rs` | 结构平移/旋转/碰撞、`MovementInfluenceCache` |
| `structure_state.rs` | 结构图；活塞目标子集 BFS 用 solution |
| `markers.rs` | 焊点、钻头头、阻拦器头等 marker |
| `behaviors.rs` | 行为阶段 |
| `runtime.rs` | 客户端 tick、worker 预取、增量呈现 |
