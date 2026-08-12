# 长参数函数处方汇总（>8 params · 87）

来源：8 路只读子 agent；共享簇仅允许权威注册表 C1–C8 及已有非主簇名。同义词黑名单已校验：无违规自创类型名。

## 权威注册表（版本锁定）

| ID | 名称 | 状态 |
|---|---|---|
| C1 | `SceneRenderMut` | **已落地** `src/scene/scene_render.rs` |
| C2 | `WorldEditScene`（内嵌 C1） | **已落地** `src/scene/world_edit.rs` |
| C3 | `SimulationPresentationDeps` | **已有且透传**回合表现系统 |
| C4 | `PanelCloseDeps` | **已贯穿** dismiss / 关钮 / Esc（定义迁至 `panels.rs`） |
| C5 | `SpawnBlockOpts` + `SpawnMode` | **已落地** `spawn.rs` |
| C6 | `BlockPanelDropdownDeps` | **已落地** `block_editing/panel_dropdown_deps.rs` |
| C7 | `GameplayPlayGate` | **已落地** `systems/gameplay/play_gate.rs` |
| C8 | `SimMovementMarkCtx` | **已落地** `oif-sim/.../movement/mod.rs` |

## 本地玩家会话（后续结构）

- 模块：[`src/game/local_player/`](../../src/game/local_player/)
- `LocalPlayer` / `LocalPlayerMut`：放置、背包、携带、模式、撤销、PlayingUi、工具剪贴板
- `GameplayPlayGate` 不再持有 `PlayingUi`；门控方法接收 `&PlayingUiState`（通常 `&player.playing_ui`）


## P0（优先改码）

| 簇 | 函数 |
|---|---|
| **C4** | `panel_close_clicked`, `dismiss_playing_overlay`, `dismiss_start_menu_overlay`, `dismiss_modals_and_host_panels`, `start_menu_escape` |
| **C3** | `present_turn` |
| **C1** | `apply_turn_output`, `apply_turn_output_incremental` |
| **C2** | `handle_selection_area_input`, `move_selection`, `copy_selection`, `spawn_selection_result`, `commit_edit_gesture`, `alternate_block_at`, `rotate_block_at`；`placement_input` 组装下传 |
| **C5** | `spawn_block_model`, `spawn_world_block_entity` |
| **C6** | converter/generator/sign/teleport `update_dropdowns`, goal `update_slot_icons` |
| **C8** | `try_deform_action`, `mark_pusher_movement` |

## 分波

1. **Wave A — C4**：`PanelCloseDeps` 贯穿 dismiss / 关钮 / Esc
2. **Wave B — C1+C3**：`SceneRenderMut`；`present_turn` 透传 TickDeps；`apply_turn_output*` 吃 C1
3. **Wave C — C2**：`WorldEditScene` 收口编辑 helper；`PlacementQueries` 只组装
4. **Wave D — C5**：`SpawnBlockOpts`/`SpawnMode`
5. **Wave E — C6/C7/C8**：块下拉、玩法门控、oif-sim mark

## 分区原文

见 [`_long_param_partials/`](_long_param_partials/)：`ui` / `gameplay` / `render` / `blockui` / `simpresent` / `other` / `simcrate` / `debugbake`。

## 命中统计（约）

| 簇 | 约命中 |
|---|---:|
| C1 | ~12（含经 C2 嵌套） |
| C2 | 8 |
| C3 | 1 |
| C4 | 5+（gameplay 关闭路径已用） |
| C5 | ~6 |
| C6 | 6 |
| C7 | ~9 |
| C8 | 2 |
| accept / local-only | 余下多数 Bevy UI/动画/session |
