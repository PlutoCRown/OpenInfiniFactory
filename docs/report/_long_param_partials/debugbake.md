# Agent-debugbake

# Agent-DebugBake 处方（清单 4 · 只读）

权威簇仅引用 C1–C8 + 已有 `DebugHttpSessionSnapshot` / `DebugHttpPerfSnapshot` / `PlayingWorldParams`。未自创类型。

---

## 逐函数处方

| 函数 | params | kind | 手段 | 共享簇ID[] | local-only / 说明 | 风险 | 优先级 |
|------|--------|------|------|------------|-------------------|------|--------|
| `handle_embedded_debug_command` | 10 | helper | **1**（已落地）+ **3 accept** 余量 | `DebugHttpSessionSnapshot`, `DebugHttpPerfSnapshot`, `PlayingWorldParams`；（Run/RunOneTurn 门控字段与 **C7** 同形，**不**整函数挂 C7） | 会话/perf/世界已聚合；旧预取状态参数已删除，余下参数均属同步命令分发。仅 1 调用方，禁止再拆空 helper。 | 勿把 HTTP 命令总线改成异步事件（须同步 `String` 回包）。 | **P2** |
| `poll_debug_http` | 14 | bevy | **1**（已落地）+ **3 accept** | 同上三已有 SystemParam | Bevy 入口已用 Session/Perf/`PlayingWorldParams` 压参；再聚会逼出自创类型。`commands` 与 `PlayingWorldParams.commands` 双通道属现状（session 进出 vs 世界刷新），处方不强制合并 | 同左；`bridge`/`player` 留入口 | **P3** |
| `embedded_status_json` | 10 | helper | **3 accept** | `[]`（调用链已由 `DebugHttpSessionSnapshot::status_json` 聚合；字段与 C7 有交集但职责是 JSON 序列化，**不**标 C7） | 纯值→JSON，无 ECS。单调用点在 `status_json`；保持扁平签名，避免 `snapshot.rs` 依赖 SystemParam | 若强行吃 SessionSnapshot 会倒依赖分层 | **P3** |
| `push_bake_target` | 11 | helper | **1** | **C5**（内部 `spawn_bake_icon_model` → `spawn_block_model`，`icon_render`/`layer` 路径） | bake 专用留下：`images`, `icon_layer`, `targets`, `index`, `out_path`, `size`。与游戏世界 spawn 同核不同壳 | 离屏 icon ≠ 玩法 spawn；勿用 C1/C2。C5 落地时用既有 `icon_render` 通道，勿平行 `SpawnArgs` | **P1**（对齐 Wave D） |

---

## 细节要点

### debug HTTP（3）
- **已做对的事：** `DebugHttpSessionSnapshot` / `DebugHttpPerfSnapshot` + `PlayingWorldParams` 正是手段 1；**勿改名**。
- **不命中主簇：** 无完整 C1/C2（无 `edit_history` 编辑路径）；Place 不经 C5；C3 是 tick/present 包，HTTP 重置 worker/snapshot 只用子集，**禁止**整包挂 C3。
- **C7：** 仅 `Run`/`RunOneTurn` 内 `playing_ui`+`ui_runtime`+`simulation`（外加 `builder_mode`）像门控；整函数是命令交换机，共享簇列不写死 C7。

### bake（1）
- `push_bake_target` **像 spawn → 标 C5**：固定核 `commands`/`meshes`/`assets(=render_assets)`/`icon_world(=world)`/`kind→BlockData`；Opts 侧对齐 `icon_render` + layer/origin。
- 同文件还有 `push_selection_bake_target` / `push_light_panel_bake_target`（不在本清单）参数高度同构 → 见缺口。

---

## 本领域命中注册表统计

| ID / 已有名 | 命中函数数 | 说明 |
|-------------|------------|------|
| C1 | 0 | 仅经 `PlayingWorldParams` 间接含渲染字段，处方不单列 C1 |
| C2 | 0 | — |
| C3 | 0 | 禁止误挂 |
| C4 | 0 | — |
| **C5** | **1** | `push_bake_target` |
| C6 | 0 | — |
| C7 | 0（软相关） | 仅 HTTP Run 门控字段同形 |
| C8 | 0 | — |
| `DebugHttpSessionSnapshot` | 2 | `poll` + `handle`（及 status 包装） |
| `DebugHttpPerfSnapshot` | 2 | `poll` + `handle` |
| `PlayingWorldParams` | 2 | `poll` + `handle` |

**手段分布：** 手段1×3（含已落地） / 手段2×0 / 手段3 accept×3（`poll`/`handle` 余量 + `embedded_status_json`）。  
**优先级：** P1×1（bake/C5） / P2×1 / P3×2。

---

## 候选缺口（只报字段列表，不命名）

1. **Bake 目标登记公共组**（≥3：`push_bake_target` / `push_selection_bake_target` / `push_light_panel_bake_target`）  
   `commands`, `images`, `meshes`, `icon_layer`, `targets`, `index`, `size`  
   （`materials` 仅后两者；`assets`/`icon_world`/`kind`/`out_path` 仅主路径 → 留给 C5，不进本缺口。）

2. **HTTP 命令可变副作用组**（仅 2 函数转发，**未达 ≥3，不建议扩注册表**）  
   `simulation`, `presentation`, `pending_generated`, `signal_cache`, `turn_cache`, `worker`, `sim_log`  
   → 维持 accept；禁止自创 `*CmdCtx`。

**本域无其它须扩 C1–C8 的字段组。**
