# 系统架构（四层）

OpenInfiniFactory 按依赖关系分为四层。后三层都依赖 **模拟核心**，可以单独演进。

```
┌─────────────────────────────────────────┐
│  UI 系统 (game/ui, game/systems)        │
├─────────────────────────────────────────┤
│  场景渲染 (scene/, game/world/render)   │
├─────────────────────────────────────────┤
│  HTTP Debug (debug_http/)               │
├─────────────────────────────────────────┤
│  模拟核心 (sim_core/, game/simulation)  │
└─────────────────────────────────────────┘
```

## 双 Bevy 运行时

模拟核心**使用 Bevy ECS**（Resource / System），但不等于完整游戏客户端。项目里有两个独立的 Bevy `App`：

| App | 入口 | 插件 | 窗口 / 渲染 |
|-----|------|------|-------------|
| 游戏客户端 | `cargo run` | `DefaultPlugins` + `GamePlugin` | 有 |
| 无头模拟 | `cargo run --bin oif-debug-http` | `MinimalPlugins`（无窗口）+ `SimCorePlugin` | 无 |

两者注册相同的模拟 Resource（`WorldBlocks`、`StructureState`、`PusherState` 等），回合逻辑统一走 `simulate_turn()`。无头 App 由 HTTP 线程直接读写 `World`，不跑渲染系统。

## 1. 模拟核心

**职责**：回合逻辑运算；通过 ECS Resource 持有状态，不创建场景实体、不重建网格。

| 模块 | 说明 |
|------|------|
| `game/simulation/core.rs` | `simulate_turn()` → `TurnOutput` |
| `sim_core/ecs.rs` | `SimCoreWorld`：在无头 App 上操作模拟 Resource |
| `sim_core/control.rs` | `SimulationControl`：回合 / 运行 / 回滚控制 |
| `sim_core/plugin.rs` | `SimCorePlugin`：注册无头模拟 Resource |
| `sim_core/headless.rs` | `build_headless_sim_app()` |
| `sim_core/cache.rs` | `TurnCache`：预计算下一回合（游戏客户端） |
| `sim_core/log.rs` | `SimulationDebugLog` |

**游戏内回合流程**：

1. `prefetch_simulation_turn`：在展示帧之前运行 `simulate_turn`，结果写入 `TurnCache`
2. `tick_simulation`：按时间轴从缓存取出 `TurnOutput`，调用 `scene::apply_turn_output` 做渲染

这样模拟计算与场景重建分离，避免「该跑回合的那一帧」同时承担逻辑 + 网格重建。

## 2. UI 系统

**职责**：菜单、HUD、输入、建造模式。通过 Bevy Resource（`WorldBlocks`、`SimulationState` 等）读写模拟状态，不直接操作渲染实体。

## 3. HTTP Debug

**两种入口**：

| 入口 | 命令 | 说明 |
|------|------|------|
| 嵌入游戏 | `cargo run -- --debug-http` | 完整 Bevy 主循环 + `poll_debug_http` |
| 独立无头 | `cargo run --bin oif-debug-http` | 无头 Bevy ECS App + HTTP，无窗口 |

共享协议：`debug_http/protocol.rs`（`/getPosBlock`、`/status`、`/runOneTurn`、`/logs` 等）。

无头模式通过 `SimCoreWorld::simulate_next_turn()` 驱动回合，适合自动化测试与脚本调试。

## 4. 场景渲染

**职责**：把模拟结果可视化。

| 模块 | 说明 |
|------|------|
| `scene/turn_visuals.rs` | `apply_turn_output()`：despawn + rebuild + 特效 |
| `game/world/rendering.rs` | 网格、材质、动画组件 |

## 依赖规则

- `simulate_turn` **不得** import `Commands` / `WorldRenderAssets`
- UI / HTTP 通过 `SimulationState` 或 `SimCoreWorld` 触发模拟，不复制回合逻辑
- 回滚时同步清空 `TurnCache`（`simulation_controls`）

## 后续方向

- 将 `WorldBlocks` 类型逐步迁出 Bevy（`bevy_math` / `glam`）
- 无头 binary 支持批量跑存档 puzzle/solution 回归
- 游戏内 `TurnCache` 深度 > 1 需配合世界快照（当前游戏内深度为 1）

## E2E 测试

目录：`e2e/`

```bash
cargo build --bin oif-debug-http
cd e2e && bun run generate-fixtures && bun test
```

通过 Debug HTTP 驱动无头 Bevy ECS App，覆盖全部 32 种 `BlockKind` 的放置/派生 marker 用例。
