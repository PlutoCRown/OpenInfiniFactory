# Agent-other

# Agent-Other 处方（清单 7）

权威簇：C1–C8 + `PlayingWorldParams` / `SessionStateParams` / `EditHistoryApply`。禁自创类型名。

---

## 逐函数处方

| 函数 | params | kind | 手段 | 共享簇ID | local-only | 风险 | 优先级 |
|---|---:|---|---|---|---|---|---|
| `update_debug_ui` | 11 | bevy | **3 accept** | — | 全保留：`debug, perf, diagnostics, world, builder_mode, simulation, sim_stats, player, block_entities, panel, refresh_at` | 无第二调用方；聚合并不减维护 | **P3** |
| `animate_blocks` | 11 | bevy | **3 accept** | — | 全保留：`time, simulation, camera, commands` + 7 路互斥 `Query`（block/pusher/drill/spark/burst/debris/laser） | Query `Without` 链是 Bevy 冲突规避，不宜塞进共享簇 | **P3** |
| `camera_move` | 10 | bevy | **1**（门控聚） | **C7** | 门控外：`time, input, keys, settings, world, scene_registry, query`；C7 的 `simulation` 本函数未用（只读门控可闲置） | 与 gameplay 输入门控对齐；勿把碰撞/世界塞进 C7 | **P2** |
| `process_pending_save` | 10 | bevy | **1**（会话切片） | **SessionStateParams**（子集：`inventory, save_state, solution_state, simulation`） | `pending, busy, world, player, commands, view_image(cfg)` | 硬塞完整 `SessionStateParams` 会带入未用字段；勿标 C2/EditHistoryApply | **P2** |
| `rebuild_playing_world` | 9 | bevy | **1** | **C1** | C1 外：`world, builder_mode`（`set_goal_play_visual`） | 与 `PlayingWorldParams.rebuild_scene` 相近但路径不同（`rebuild_world_on_enter` + `ResMut` assets）；勿强行改成 Playing 整包 | **P1** |
| `on_exit_playing` | 9 | bevy | **1**（局部）+ **3** | **SessionStateParams**（仅 `playing_ui`） | `commands, ui_runtime, ui_host, gameplay_scene, icon_roots, playing_ui_roots, meshes, scene_chunks` | **勿标 C4**：仅有 `ui_runtime/ui_host`，无 dismiss 全家桶；非 C1 全量 | **P2** |
| `process_deferred_main_menu_exit` | 9 | bevy | **3 accept**（已聚合） | **PlayingWorldParams** + **SessionStateParams** | 已聚合外：`player, next_state, start_menu_screen, pending_exit, edit_history, busy, view_image(cfg)` | `edit_history.clear` ≠ `EditHistoryApply`（后者是 undo 刷景） | **P3** |

---

## 要点说明

### `rebuild_playing_world` → C1（P1）
字段命中 C1：`commands, meshes, render_assets, block_index, scene_chunks, debug, structure_state`。  
签名约：`(scene: C1/SceneRenderMut, world, builder_mode)`。  
不建议整包 `PlayingWorldParams`：多出 `movement_influence/pusher_state/block_entities`，且 assets 可变性/重建入口不一致。

### `camera_move` → C7
入口门控与 C7 一致：`mode + playing_ui + ui_runtime`（本处无 `simulation.is_active()`）。手段 1 收门控即可，运动逻辑留函数上。

### Session 进出
- `process_deferred_main_menu_exit`：**已完成** Playing+Session 聚合，处方为 accept。
- `process_pending_save`：会话写档字段对齐 `SessionStateParams` 子集；`world/commands` 不必升格为 Playing 整包。
- `on_exit_playing`：只稳拿 `playing_ui`；拆景/卸 UI 为 exit-local。

### 明确不命中
| 簇 | 本清单 |
|---|---|
| C2 / C3 / C5 / C6 / C8 | 无 |
| C4 | `on_exit` 仅部分 UI 字段，职责是 exit teardown |
| EditHistoryApply | 仅 deferred exit 清 history，无刷景 |

---

## 统计

| 项 | 值 |
|---|---|
| 函数数 | **7** |
| kind | 全部 **bevy**（0 helper） |
| 手段 1 | **4**（camera / pending_save / rebuild / on_exit 局部） |
| 手段 2 | **0** |
| 手段 3 accept | **3**（debug_ui / animate / deferred 已聚合） |
| 注册表命中 | C1×1，C7×1，SessionStateParams×3，PlayingWorldParams×1 |
| 优先级 | P1×1，P2×3，P3×3 |

---

## 候选缺口（仅字段列表，不命名新类型）

1. **Playing 进场重建（C1 外）**  
   `world`, `builder_mode`  
   （可选差异：`render_assets` 需 `ResMut` vs Playing 内 `Option<Res>`）

2. **排队保存（Session 子集外）**  
   `pending`/`PendingSave`, `busy`/`SessionBusy`, `world`, `player: Query<(&FlyCamera,&Transform)>`, `commands`, `view_image: Option<GameplayViewImage>`（非 wasm）

3. **Playing 退出拆景**  
   `commands`, `ui_runtime`, `ui_host`, `Query<GameplayScene>`, `Query<BlockIconRenderRoot>`, `Query<PlayingUiRoot>`, `meshes`, `scene_chunks`  
   （可选：`playing_ui` 已可由 SessionStateParams 提供）

4. **延后回主菜单（已有 Playing+Session 之外）**  
   `player`, `next_state`, `start_menu_screen`, `pending_exit`, `edit_history`, `busy`, `view_image(cfg)`

5. **不做新组（accept）**  
   - debug UI 只读诊断 Res/Query  
   - `animate_blocks` 多路互斥动画 Query  

本域 **不建议** 为缺口 1–4 新建共享类型（单路径/已有包可覆盖）；落地优先：`rebuild`→C1，`camera_move`→C7。