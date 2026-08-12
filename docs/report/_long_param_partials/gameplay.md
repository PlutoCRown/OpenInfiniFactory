# Agent-gameplay

# Agent-Gameplay 处方（gameplay-edit 16，只读）

对齐约束已落实：编辑链 `move/copy/commit/alternate/rotate/spawn_selection_result` → **C2（内嵌 C1）**；门控系统 → **C7**；`PlacementQueries` **组装 C2，不另起字段集**；颜色下拉 → **C6**；`gameplay_input` 关闭路径 → **C4**（已落地）。

---

## 处方表

格式：`函数 | params | kind | 手段 | 共享簇ID[] | local-only | 风险 | 优先级`

| 函数 | params | kind | 手段 | 共享簇ID[] | local-only | 风险 | 优先级 |
|---|---:|---|---|---|---|---|---|
| `handle_selection_area_input` | 22 | helper | **1** 聚合 C2；**3** 拆输入 vs 世界变更 | `[C2]` | `mouse_buttons, keys, current_target, place/delete_button, copy_chord, force_place, builder_mode, entry, placement, locale, toast` | 感染源：现把 C2 十参摊开传给 move/copy；禁止另起 SelectionEditCtx | **P0** |
| `move_selection` | 16 | helper | **1** | `[C2]` | `bounds, offset, force, builder_mode, entry, selection_before` | 须同步 `bool`，禁手段 2 | **P0** |
| `copy_selection` | 16 | helper | **1** | `[C2]` | 同 move | 与 move 签名双胞胎，必须同吃 C2 | **P0** |
| `spawn_selection_result` | 10 | helper | **1**（用 C2 的 `scene`+`world`） | `[C2]`（实际消费 C1+`world`） | `animations, also_dirty` | 不用 `edit_history`/`block_entities`；传整 C2 可接受，或只借 `scene`+`world`，**禁平行新簇** | **P0** |
| `sync_edit_bounds_overlays` | 14 | bevy | **1** 门控；**3** accept 余下 | `[C7]` | `placement, inventory, builder_mode, solution_state, config, world, keys, render_assets, selection_parts, delete_parts` | 非编辑写世界；勿塞 C2 | **P1** |
| `placement_input` | 15 | bevy | **1** C7 + 由 `PlacementQueries` **组装** C2 | `[C7]` + 已有 `PlacementQueries`→C2 | `mouse_buttons, keys, inventory, config, builder_mode, placement, solution_state`；Queries 内 `edit_previews, player, input, touch, pending_block_panel, locale, toast` | 禁止第三套字段；world/history/commands 与 Queries 内 C1/`block_entities` 合成 `&mut C2` 再下传 helper | **P0** |
| `commit_edit_gesture` | 16 | helper | **1** | `[C2]` | `gesture, current_place_at, current_delete_at, config, builder_mode, entry, player_pos` | 现缺 `block_entities`；收 C2 后字段闲置可接受；须同步 `bool` | **P0** |
| `spawn_gesture_previews` | 10 | helper | **3** accept / local-only | `[]` | `gesture, current_place_at, config, world, builder_mode, entry, player_pos, commands, meshes, render_assets` | 仅 C1 三字段子集；**勿硬套 C1/C5**；单调用方 | **P2** |
| `clipboard_input` | 15 | bevy | **1** 门控；编辑侧沿用已有 | `[C7]` + 已有 `PlayingWorldParams` | `keys, config, inline_edit, builder_mode, placement, inventory, clipboard, tool_swap, edit_history, solution_state` | 已用 `PlayingWorldParams`，勿改成 C2（配置粘贴非格子搬迁）；勿发明 ClipboardDeps | **P1** |
| `update_hover` | 14 | bevy | **1** 门控；预览沿用已有 | `[C7]` + 已有 `HoverPreviewDeps` | `placement, config, debug, camera, world, structure_state, hover_bounds, marker, aim_face` | 勿把 Hover 扩成 C2；simulation 仅门控后分支 | **P1** |
| `sync_factory_activity_debug_overlays` | 9 | bevy | **3** accept | `[]` | 全部：`debug, placement, world, structure_state, render_assets, block_index, commands, overlays, last` | 无 ≥3 函数共簇；勿为减参新建类型 | **P3** |
| `alternate_block_at` | 11 | helper | **1** | `[C2]` | `pos` | 字段集已 ≈ 完整 C2+pos；从 placement 下传时勿再拆 | **P0** |
| `rotate_block_at` | 10 | helper | **1** | `[C2]` | `pos, reverse` | 现无 `edit_history`/`debug`；收 C2 会多传；仍禁平行子集类型 | **P0** |
| `update_color_select_dropdowns` | 12 | bevy | **1**（强制） | `[C6]` | 块私有 Query：`slots, options, icons, lists, triggers` | 必须 C6；禁 `DropdownCtx` / ColorSelectDeps | **P1** |
| `edit_history_input` | 12 | bevy | **1** 门控；应用侧沿用已有 | `[C7]` + 已有 `EditHistoryApply`（≈C1） | `keys, config, inline_edit, edit_history, world, placement, solution_state` | `EditHistoryApply` 字段 ≈ C1，可日后 `as_scene()`，**勿改名/另起** | **P1** |
| `gameplay_input` | 12 | bevy | **1** 关闭路径已 C4；可选门控 | `[C4]`（关闭）；门控字段可再收 `[C7]` | `input, mouse_wheel, keys, placement, carried, inventory, solution_state, commands`；C4 内已含 ui 关层资源 | **关闭必须继续走 C4**（已有 `PanelCloseDeps`）；禁 OverlayDismiss*；C7 与 C4 勿合成巨包 | **P1**（C4 已完成则余量 **P2**） |

---

## 逐函数要点（不省略）

### selection.rs

1. **`handle_selection_area_input`（22）** — helper。手段 1+3：签名应收 `&mut WorldEditScene`，输入/模式/toast 留 local。自己几乎只做状态机，真正变更在 move/copy。**P0**。
2. **`move_selection`（16）** — helper。手段 1 → C2；local 为几何与合法性参数。返回 `bool`，禁消息化。**P0**。
3. **`copy_selection`（16）** — 同 move。**P0**。
4. **`spawn_selection_result`（10）** — helper。手段 1：吃 C2，只用 `scene`（C1）+`world`；`animations`/`also_dirty` local。**P0**。
5. **`sync_edit_bounds_overlays`（14）** — bevy。手段 1 → C7；overlay Query 与 placement 等 local/accept。**P1**。

### placement.rs

6. **`placement_input`（15）** — bevy。手段 1：`GameplayPlayGate` + 现有 `PlacementQueries` **转换成** C2（commands + world + edit_history + Queries 内 meshes/render/debug/structure/index/chunks/block_entities），再调用 handle/commit/alternate/rotate。**禁止**再定义 PlacementEditBundle。**P0**。
7. **`commit_edit_gesture`（16）** — helper。手段 1 → C2；手势/config/mode/player_pos local。现无 block_entities。**P0**。
8. **`spawn_gesture_previews`（10）** — helper。手段 3 accept：预览只要 commands/meshes/render_assets+world，不是完整 C1，也不是 C5。**P2**。

### clipboard.rs / hover.rs / edit_ops.rs / color / history / input

9. **`clipboard_input`（15）** — bevy。C7 + `PlayingWorldParams`；clipboard/tool_swap 等 local。**P1**。
10. **`update_hover`（14）** — bevy。C7 + `HoverPreviewDeps`；准星 Query local。**P1**。
11. **`sync_factory_activity_debug_overlays`（9）** — bevy。手段 3 accept。**P3**。
12. **`alternate_block_at`（11）** — helper。完整 C2 + `pos`。**P0**。
13. **`rotate_block_at`（10）** — helper。对齐收 C2；缺 history/debug。**P0**。
14. **`update_color_select_dropdowns`（12）** — bevy。`[C6]`；私有 Query 不上簇。**P1**。
15. **`edit_history_input`（12）** — bevy。C7 + `EditHistoryApply`。**P1**。
16. **`gameplay_input`（12）** — bevy。关闭路径已 `[C4]`；可另收 C7，勿与 C4 合并。**P1/P2**。

---

## 注册表命中统计（本领域 16）

| ID | 名称 | 命中函数数 | 函数 |
|---|---|---:|---|
| **C1** | SceneRenderMut | 0 直接 / **7 经 C2 嵌套** | 经 C2：handle / move / copy / spawn_selection_result / commit / alternate / rotate；（`EditHistoryApply`≈C1，算已有非主簇） |
| **C2** | WorldEditScene | **8** | handle_selection_area_input, move_selection, copy_selection, spawn_selection_result, commit_edit_gesture, alternate_block_at, rotate_block_at；+ placement_input（组装下传） |
| **C3** | SimulationPresentationDeps | **0** | — |
| **C4** | PanelCloseDeps | **1** | gameplay_input（关闭路径，已落地） |
| **C5** | SpawnBlockOpts+SpawnMode | **0** | spawn_gesture_previews 明确不套 |
| **C6** | BlockPanelDropdownDeps | **1** | update_color_select_dropdowns |
| **C7** | GameplayPlayGate | **6** | placement_input, clipboard_input, update_hover, sync_edit_bounds_overlays, edit_history_input；（gameplay_input 可选） |
| **C8** | SimMovementMarkCtx | **0** | — |
| 已有非主簇 | PlacementQueries / HoverPreviewDeps / EditHistoryApply / PlayingWorldParams | **4** | placement_input / update_hover / edit_history_input / clipboard_input |

**未命中注册表、建议手段 3：** `spawn_gesture_previews`、`sync_factory_activity_debug_overlays`（2）。

---

## 候选缺口（仅字段列表，不命名）

跨 ≥3 本清单函数、且**未**被 C1–C8 覆盖的重复组：

1. **候选缺口：** `builder_mode`, `entry`（`WorldEntryMode`）  
   出现于：handle / move / copy / commit / spawn_gesture_previews（及同类合法性判断）。仅 2 字段，父 agent 可决定是否入册；子 agent **不自创** EditModeCtx 之类。

2. **无其它强缺口。**  
   - 选区输入键鼠包仅 handle 一处 → local-only。  
   - PlacementQueries 超额字段（preview/player/touch/toast…）→ 留在已有 `PlacementQueries`，不扩注册表。  
   - `EditHistoryApply` 与 C1 高度同构 → 汇总时考虑「可转换成 C1」，**勿平行新名**。

---

## Wave 建议（供父 agent）

- **Wave C / P0：** 落地 `WorldEditScene`（内嵌 `SceneRenderMut`），收口 handle→move/copy→spawn_selection_result 与 commit/alternate/rotate；`placement_input` 的 `PlacementQueries` 提供 `as_edit_scene()` 类组装，禁止第三套。
- **Wave E / P1：** 门控系统抽 `GameplayPlayGate`；`update_color_select_dropdowns` 并入 C6（与 BlockUI 一致）。
- **C4：** gameplay 关闭路径已合规，改码时勿回退拆开。
