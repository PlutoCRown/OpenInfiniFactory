# 系统架构（单一权威模拟状态）

OpenInfiniFactory 按依赖向下分层。模拟核心在独立 crate，仅依赖 `bevy_ecs`、`glam` 与序列化库；主 crate 负责表现、UI 与平台接入。

```
┌─────────────────────────────────────────┐
│  UI / gameplay（game/ui, systems）      │
├─────────────────────────────────────────┤
│  场景渲染（scene/, game/world/render）  │
├─────────────────────────────────────────┤
│  表现桥接（sim_bridge）                 │
├─────────────────────────────────────────┤
│  模拟核心 crates/oif-sim（ECS Resource）│
└─────────────────────────────────────────┘
```

## 1. 模拟核心（`crates/oif-sim`）

主 crate 依赖 `oif-sim`。职责：定义唯一的权威世界与模拟状态、方块 Meta/Behavior、`simulate_turn()`，以及供无头工具组合使用的 `SimSession`。

| 模块 | 说明 |
|------|------|
| `world/` | 网格、朝向等纯世界数据（glam） |
| `blocks/` | `BlockMeta` / `BlockBehavior` + 各方块声明 |
| `simulation/` | 四阶段 `simulate_turn` → `TurnOutput`（含运动 / 激光等纯数据 DTO） |
| `session/` | 游戏与无头端共用的 `SimulationControl`、`SimSession` 与日志 |

回合四阶段：信号探测 → 运动标记 → 执行运动 → 结构后处理。细节见 [`simulation_turn_phases.md`](simulation_turn_phases.md)。

`WorldBlocks`、结构、信号、推杆和跨回合挂起状态直接实现 `Resource`。游戏侧只 re-export，不再维护包装类型或第二套控制状态。

## 2. 表现桥接（`src/sim_bridge/`）

`sim_bridge` 在主线程原地推进权威模拟 Resource，把 `TurnOutput` 增量应用到场景。表现状态只保存通电集合等可丢弃数据，不保存世界副本。

| 模块 | 说明 |
|------|------|
| `present.rs` | 推进权威状态，并把回合输出应用到 Bevy 场景、动画和音效 |

游戏内回合流程：

1. 输入或 HTTP 更新共用的 `SimulationControl`
2. `advance_simulation` 原地调用 `simulate_turn`，发布 `TurnCommitted`
3. `present_simulation_turns` 消费事件，增量驱动场景、动画和音效
4. 编辑期放置/删除由 `scene/incremental` 只刷新改动邻域

## 3. HTTP Debug

| 入口 | 命令 | 说明 |
|------|------|------|
| 嵌入游戏 | `cargo run -- --debug-http` | 主循环 + `poll_debug_http` |
| 独立无头 | `cargo run --bin oif-debug-http` | 自有 `SimSession` + HTTP（无 Bevy App） |

协议：`debug_http/protocol.rs`。无头模式直接驱动 `SimSession`，不复制模拟逻辑。

## 4. UI 与场景

- **UI**：`UiNavigation` 是页面、覆盖层、面板、模态框和教程的唯一导航状态；`UiMountCache` 只缓存实体句柄。交互通过意图更新导航，挂载系统据此增量同步实体树。
- **教程**：`TutorialCatalog` 注册数据化步骤，所有教程共用一个面板壳；新增教程无需增加新的 Rust 面板类型。
- **场景**：`scene/` + `game/world/rendering` 可视化 `TurnOutput` 与编辑 diff。
- **表现类型**：`RenderBehavior` / `BlockModel` 等在 `game/blocks/render_types.rs`，不进入 `oif-sim`。

## 依赖规则

- `simulate_turn` 不得依赖 `Commands`、渲染资产或 UI 类型
- UI / HTTP 只触发会话或消费 `TurnOutput`，不复制回合逻辑
- `oif-sim` 只允许依赖 `bevy_ecs`，不得依赖渲染、窗口、输入或平台 API
- 表现层不得保存 `WorldBlocks` 副本

## 已知剩余债务

`simulate_turn` 仍是单一回合事务入口。只有在阶段需要独立调度、并行或复用时才继续拆成 ECS System，避免制造只有一处调用的薄函数。

## 调试存档

通过 `--load-save` 加载 Puzzle / Solution / Free 存档，用 `oif-debug-http` 驱动模拟。详见 `.agents/skills/sim-debug-http/`。
