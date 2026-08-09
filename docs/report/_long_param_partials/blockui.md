# Agent-blockui

## Agent-BlockUI 处方（block-panels 11）

**C6 锁定字段（与 color 同类 `update_color_select_dropdowns` 交集一致）：**  
`UiMainThread` / `ui_runtime` / `OpenBlockPanelDropdown` / `world: WorldBlocks` / `block_icons: Option<BlockIconAssets>` / `commands` / `windows: PrimaryWindow`

**硬规则：** 下拉共性只进 **C6**；禁止 `ConverterPanelDeps` 等私有 deps 装共享字段；块私有 `Query`/`Local` 留在函数参数上。

---

### 完整处方表

| # | 函数 | 路径 | params | kind | 手段 | 共享簇ID | local-only / 说明 | 风险 | 优先级 |
|---|---|---|---:|---|---|---|---|---|---|
| 1 | `update_dropdowns` | `src/game/blocks/converter/ui.rs` | 13 | bevy | **1** 聚合 | **C6** | 私有留函数：`material_slots`/`output_slots`/`material_options`/`material_icons`/`list_queries`/`triggers` | C6 落地后约 7 参；勿把双槽 Query 塞进 C6 | **P0** |
| 2 | `on_click` | 同上 | 9 | bevy | **3** accept | — | 标准块面板点击入口；`actions: Query<&ConverterAction>` 私有；可选 `carried` 不进共享 | 勿并入 C6（缺 icons/windows，且 `PlayingWorldParams`+`edit_history`） | **P2** |
| 3 | `update_panel` | `src/game/blocks/generator/ui.rs` | 12 | bevy | **3** accept | — | 仅文字/行显隐同步；与 C6 仅薄交集（`UiMainThread`+`ui_runtime`+`world`），**禁止**为减参强吃 C6 | 强绑 C6 会空拉 `open_dropdown`/`block_icons`/`windows` | **P3** |
| 4 | `update_dropdowns` | 同上 | 12 | bevy | **1** 聚合 | **C6** | 私有：`material_slots`/`material_options`/`material_icons`/`lists`/`triggers` | 同 converter | **P0** |
| 5 | `on_click` | 同上 | 9 | bevy | **3** accept | — | 同 converter；另有 `dispatch_action` helper，入口仍 accept | 同 #2 | **P2** |
| 6 | `update_slot_icons` | `src/game/blocks/goal/ui.rs` | 12 | bevy | **1** 聚合 | **C6**（前半） | 前半对齐 C6；本系统无 `open_dropdown`/`windows` 使用可接受空载；私有：三组 slot/option Query + `material_icons` | C6 含未用字段属可接受过取；勿为此另开子集类型 | **P0** |
| 7 | `on_click` | 同上 | 9 | bevy | **3** accept | — | 同 #2；含 `carried` | 同 #2 | **P2** |
| 8 | `update_dropdowns` | `src/game/blocks/sign/ui.rs` | 12 | bevy | **1** 聚合 | **C6** | 私有：`display_slots`/`material_options`/`material_icons`/`lists`/`triggers` | 同 converter | **P0** |
| 9 | `on_click` | 同上 | **10*** | bevy | **3** accept | — | 清单标 9，源码实为 10（多 `PendingSignTextEdit`）；私有 `actions` | 同 #2；pending 更不能进 C6 | **P2** |
| 10 | `update_dropdowns` | `src/game/blocks/teleport/ui.rs` | 12 | bevy | **1** 聚合 | **C6** | 无图标路径：`block_icons` 空载 OK；私有：`last_label`/`labels`/`lists`/`pair_options`/`triggers`/`pair_cache` | 勿因无 icons 自创平行 deps | **P0** |
| 11 | `on_click` | 同上 | 9 | bevy | **3** accept | — | `pending_rename` 块私有；无私有 carried | 同 #2 | **P2** |

\*清单 `sign/on_click(9)` 与源码 10 参不一致，以源码为准。

**域外对齐（非本清单行，供父 agent 汇总）：**  
`src/game/block_editing/color_slot_ui.rs` → `update_color_select_dropdowns`(12) 同构前半 → **必须 C6**（与本域 update_dropdowns 锁定同一字段集）。

---

### 统计

| 项 | 数量 |
|---|---:|
| 本清单函数 | **11** |
| 手段1 → C6 | **5**（4×`update_dropdowns` + 1×`update_slot_icons`） |
| 手段3 accept | **6**（1×`update_panel` + 5×`on_click`） |
| 手段2 消息 | **0** |
| 命中注册表 C6 | **5** |
| 命中 C1–C5/C7/C8 | **0** |
| 引用已有非主簇 | `PlayingWorldParams`（仅 on_click，不新建） |
| P0 / P2 / P3 | **5 / 5 / 1** |
| 禁止的私有 `XxxPanelDeps` 建议 | **0**（均未开） |

C6 落地后预期：5 个下拉/图标系统入口参数从 12–13 降到约 **6–8**（C6 + 私有 Query）。

---

### 候选缺口（不命名；供父 agent 扩表）

**缺口 A — 块面板 `on_click` 共性（≥5 处平行，且与 C6/C4 字段错位）：**

- `On<Pointer<Click>>`
- `UiHost`
- `UiRuntime`
- `OpenBlockPanelDropdown`（**ResMut**，与 C6 只读不同）
- `SolutionState`
- `EditHistory`
- `PlayingWorldParams`

**不进缺口：** 各块 `Query<&XxxAction>`；`CarriedItem` / `PendingSignTextEdit` / `PendingTeleportRename`（非整组必现）。

**本轮建议：** 仍 **accept**；等父 agent 决定是否登记新簇后再聚合。**禁止**用私有 `XxxPanelDeps` 或强塞 **C6**。