# Agent-simpresent

# Agent-SimPresent 处方（sim-present 7）

**硬约束遵守：** 无 `TurnPresentCtx` / `PresentDeps` / `SimPresentBundle`；`present_turn→C3`，`apply_turn_output*→C1`，`spawn_and_index→C5`。全员 **手段1**；**手段2 不适用**（需同步改世界/实体，禁止消息化）。

---

## 逐函数处方

| 函数 | params | kind | 手段 | 共享簇ID[] | local-only说明 | 风险 | 优先级 |
|---|---:|---|---|---|---|---|---|
| `present_turn` | 18 | helper | **1** | **C3** | — | `SimulationTickDeps` 字段同模块私有，透传 `&mut deps` 即可；`commands` / `CachedTurn` / `animation_duration` / `last_powered_devices` 留调用期 | **P0** |
| `apply_turn_output` | 14 | helper | **1** | **C1** | 薄转发，不另拆空函数 | 与 incremental 签名对齐；`stats`/`portal_flash_queue` 留调用期 | **P0** |
| `apply_turn_output_incremental` | 14 | helper | **1** | **C1** | — | 内调 `apply_structure_animations` + `refresh_positions` 须同传 `&mut SceneRenderMut` | **P0** |
| `apply_structure_animations` | 13 | helper | **1** | **C1** | `_debug`/`_structure_state` 随 C1 带入即可，勿为减参另造子集类型 | 无 `scene_chunks` 使用；仍吃完整 C1，避免平行半包 | **P1** |
| `spawn_and_index` | 12 | helper | **1**（辅可 3 内联） | **C5** | 多处调用，保留包装；C5 落地后可直接调核 | 与 `spawn_world_block_entity` 对齐 Opts/Mode | **P1** |
| `refresh_positions` | 12 | helper | **1** | **C1** | — | 同上，C1 含未用 `scene_chunks`；`world`+集合参数留调用期 | **P1** |
| `refresh_edit_changes` | 9 | helper | **1** | **C1** | **非 C2**：无 `edit_history`/`block_entities` | `structure_state` 需 `&mut`（`apply_factory_edit`）；调用方多（selection/placement/edit_history/PlayingWorldParams） | **P1** |

---

### 目标签名（示意）

**`present_turn`（C3）**  
`present_turn(cached, animation_duration, last_powered_devices, &mut deps: SimulationTickDeps, &mut commands)`  
→ 约 5 参。内部用 deps 做 snapshot + 组装 C1 调 `apply_turn_output`。

**`apply_turn_output` / `apply_turn_output_incremental`（C1）**  
`(before, after, output, previous_powered_wires, animation_duration, &mut scene: SceneRenderMut, stats, portal_flash_queue)`  
→ 约 8 参。C1 覆盖：`commands, meshes, render_assets, block_index, scene_chunks, debug, structure_state`。

**`apply_structure_animations`（C1）**  
`(world, powered_wires, animations, pusher_animations, timing, paint_changed_ids, teleported_from, &mut scene)`  
→ 约 8 参。

**`refresh_positions`（C1）**  
`(world, powered_wires, positions, skip, pusher_animations, timing, &mut scene)`  
→ 约 7 参。

**`refresh_edit_changes`（C1）**  
`(world, changed, &mut scene)`  
→ 约 3 参。

**`spawn_and_index`（C5）**  
吃 `SpawnBlockOpts`(+`SpawnMode`)；固定核：`commands/meshes/render_assets/world/pos/data`，其余进 Opts。

---

### 手段选型说明

| 手段 | 本清单结论 |
|---|---|
| **1 聚合** | 7/7。C3 已存在却被拆开（`tick_simulation`→18 参瀑布）；C1 在 turn/edit 刷新链重复；C5 收 spawn 包装。 |
| **2 消息** | **0**。`present_turn`/`apply_*`/`spawn_*` 必须同帧同步完成，禁消息总线。 |
| **3 accept/拆职责** | 仅辅：`apply_turn_output` 与 `spawn_and_index` 为薄包装，可随 C1/C5 落地选择内联，**不**为此新建类型。 |

**非 Rust 风格提醒：** 不要为每个函数私有一个 PresentCtx；已有 `SimulationTickDeps` 应直接透传（惯用 SystemParam→`&mut` helper）。

---

## 统计

| 项 | 值 |
|---|---|
| 审查函数 | **7** |
| 手段1 / 2 / 3主 | **7 / 0 / 0** |
| 命中 **C3** | 1（`present_turn`） |
| 命中 **C1** | 5（两 `apply_turn_output*`、`apply_structure_animations`、`refresh_positions`、`refresh_edit_changes`） |
| 命中 **C5** | 1（`spawn_and_index`） |
| 命中 C2/C4/C6/C7/C8 | **0** |
| P0 / P1 | 3 / 4 |
| 与注册表冲突/同义词 | **无** |

---

## 候选缺口

**无。**  
（`world` / `stats` / `portal_flash_queue` / 动画与格子集合均为调用期参数；`portal_flash_queue` 已在 C3 内，不必升新簇。`SceneRenderMut + world` 故意不做成 C2，因缺 `edit_history`/`block_entities`。）