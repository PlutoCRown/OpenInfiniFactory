# 系统架构（四层）

OpenInfiniFactory 按依赖向下分层。模拟核心在独立 crate（无 Bevy）；主 crate 只做表现、UI 与调试接入。

```
┌─────────────────────────────────────────┐
│  UI / gameplay（game/ui, systems）      │
├─────────────────────────────────────────┤
│  场景渲染（scene/, game/world/render）  │
├─────────────────────────────────────────┤
│  表现桥接 + 预取（sim_bridge）          │
├─────────────────────────────────────────┤
│  模拟核心 crates/oif-sim（glam/serde）  │
└─────────────────────────────────────────┘
```

## 1. 模拟核心（`crates/oif-sim`）

主 crate 依赖 `oif-sim`。职责：世界状态、方块 Meta/Behavior、`simulate_turn()`、自有 `SimSession`（无 Bevy App）。

| 模块 | 说明 |
|------|------|
| `world/` | 网格、朝向等纯世界数据（glam） |
| `blocks/` | `BlockMeta` / `BlockBehavior` + 各方块声明 |
| `simulation/` | 四阶段 `simulate_turn` → `TurnOutput`（含运动 / 激光等纯数据 DTO） |
| `session/` | 自有 `SimSession`、控制面与日志 |

回合四阶段：信号探测 → 运动标记 → 执行运动 → 结构后处理。细节见 [`simulation_turn_phases.md`](simulation_turn_phases.md)。

游戏侧通过 `Deref` Resource 包装（如 `game::world::grid::WorldBlocks`）把同一数据挂进 Bevy。

## 2. 表现桥接与预取（`src/sim_bridge/`）

`sim_bridge` 同时负责表现编排与预取：把 `SimSnapshot` / `TurnOutput` 增量应用到 Bevy，并用 `SimulationWorker` / `TurnCache` 预计算未来回合；会话类型 re-export 自 `oif_sim`。

| 模块 | 说明 |
|------|------|
| `present.rs` | 把快照 / 回合输出应用到 Bevy 世界与渲染 |
| `cache.rs` / `worker.rs` / `snapshot.rs` | 预取缓存、后台 worker、`SimSnapshot` / `CachedTurn` |

游戏内回合流程：

1. `SimulationWorker` 预计算未来回合，写入 `TurnCache`
2. `poll_simulation_worker` 同步意图并 ingest
3. `tick_simulation` 从缓存取出回合，增量 `apply_turn_output`
4. 编辑期放置/删除：`scene/incremental` 只刷改动邻域

## 3. HTTP Debug

| 入口 | 命令 | 说明 |
|------|------|------|
| 嵌入游戏 | `cargo run -- --debug-http` | 主循环 + `poll_debug_http` |
| 独立无头 | `cargo run --bin oif-debug-http` | 自有 `SimSession` + HTTP（无 Bevy App） |

协议：`debug_http/protocol.rs`。无头模式直接驱动 `SimSession`，不复制模拟逻辑。

## 4. UI 与场景

- **UI**：菜单、HUD、建造；经 Resource / session API 读写世界，不直接改模拟阶段。
- **场景**：`scene/` + `game/world/rendering` 可视化 `TurnOutput` 与编辑 diff。
- **表现类型**：`RenderBehavior` / `BlockModel` 等在 `game/blocks/render_types.rs`，不进入 `oif-sim`。

## 依赖规则

- `simulate_turn` 不得依赖 `Commands`、渲染资产或 UI 类型
- UI / HTTP 只触发会话或消费 `TurnOutput`，不复制回合逻辑
- 回滚时清空 `TurnCache`
- `oif-sim` 不得依赖 Bevy

## 已知剩余债务

无架构债务。

## E2E

```bash
cargo build --bin oif-debug-http
cd e2e && bun run generate-fixtures && bun test
```
