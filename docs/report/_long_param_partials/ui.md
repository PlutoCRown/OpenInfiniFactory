# Agent-ui

# UI 域长参数处方（26）

对齐约束：`dismiss_*` / 关闭路径一律引用 **C4 PanelCloseDeps**（字段以 `input.rs` 为准，不含 `commands`）。

---

### panels.rs

`panel_close_clicked` | **17** | bevy | **1** 聚合 C4；关钮事件本地保留 | **[C4]** | local-only：`On<Click>`、`close_buttons`、`commands`、以及 playing 关闭链额外态（`playing_ui`/`carried`/`inventory`/`placement`/`solution_state`） | 与 `gameplay_input` 双路径易漂移 | **P0**

`dismiss_playing_overlay` | **15** | helper | **1** 入参改为接受 `&mut PanelCloseDeps`（或等价拆包） | **[C4]** | local-only：playing 叠层态 + `commands` | helper 签名与 C4 字段顺序不一致时易漏传 | **P0**

`dismiss_start_menu_overlay` | **11** | helper | **1** 同上，收 C4 | **[C4]** | local-only：`start_menu_screen` + `commands` | 低 | **P0**

`dismiss_modals_and_host_panels` | **10** | helper | **1** 几乎就是 C4+`commands` | **[C4]** | local-only：`commands` | 低 | **P0**

`update_panel_visibility` | **13** | bevy | **3** 拆职责：可见性判定 vs 节点写回；Query/`Local` 留本地 | **[]** | local-only：三组 `Added` + `ParamSet` 节点查询、`settings_tab`/`world`/`open_block_dropdown` 判定输入 | 聚合可见性状态易引入无关 change 检测 | **P2**

---

### pause_menu/mod.rs

`dispatch_pause_menu_clicks` | **15** | bevy | **1** 聚合已有 **SessionStateParams**；按钮体已用 `PauseMenuCtx`（手段3 已部分落地） | **[SessionStateParams]** | local-only：`UiMainThread`、`MessageReader`、`mode` 门、`busy`、`world`、`playing_ui_root`、`commands`（若并入 Session 需注意与 `PlayingWorldParams.commands` 冲突） | `SessionStateParams` 含多余字段（`free_inventory_tab`/`pending_player`） | **P1**

---

### status.rs

`update_status_ui` | **14** | bevy | **3** 按脏区拆本地缓存职责（已有 cache）；不新建公共簇 | **[]** | local-only：全部 Res/`Local`/Query；只读拼文案 | 为减参硬聚合无收益 | **P3**

---

### inventory/

`update_inventory_slots` | **13** | bevy | **3** Query/`Local` 本地化；内容同步可继续内联 | **[]** | local-only：图标/选中/hover 的 `Local` + 多 Query | 低 | **P3**

`inventory_hotbar_digit_input` | **11** | bevy | **3** typing 门控只读三件套本地保留；**非** dismiss 关闭路径，勿强行整包 C4 | **[]** | local-only：`text_prompt`/`pending_key_bind`/`inline_edit`（typing gate）+ 热键应用态 | 若误并 C4 会把关闭路径 mut 借进输入系统 | **P2**

---

### settings/

`update_settings_dropdowns_ui` | **13** | bevy | **3** 标签/数值/列表三路已分脏；Query 本地 | **[]** | local-only：窗口与下拉定位 Query | 低 | **P3**

`update_settings_text_ui` | **10** | bevy | **3** local-only | **[]** | local-only：`bridge` cfg、文本 Query | 低 | **P3**

`update_settings_sliders_ui` | **9** | bevy | **3** local-only | **[]** | local-only：fill/knob/value Query + `commands` | 低 | **P3**

`settings_menu_actions` | **11** | bevy | **3** 改键/滑条提交可再拆消息，但当前同面板内聚可接受 | **[]** | local-only：输入设备 + settings 可变态 + slider Query | 拆消息增加延迟/顺序风险 | **P2**

`dispatch_settings_actions` | **9** | bevy | **2** 已消息驱动；继续瘦 action 分支即可 | **[]** | local-only：settings 瞬时 UI 态 + `touch` + `commands` | 低 | **P3**

---

### start_menu/

`start_menu_escape` | **13** | bevy | **1** 聚合 C4 后调 `dismiss_start_menu_overlay` | **[C4]** | local-only：`GameplayInputState`、`mode`、`start_menu_screen`、`commands` | 与 playing 关闭路径不一致 | **P0**

`sync_start_menu_mounts` | **10** | bevy | **3** 挂载/卸载职责本地；可见性门与存档列表共享字段见文末缺口 | **[]** | local-only：`busy`/`root`/`ui_scale`/`windows`/`mounts`/`save_list_render`/`commands` | 与 save list 门控重复但不宜硬并 C* | **P2**

---

### virtual_remote/

`sync_virtual_remote_visibility` | **13** | bevy | **1** 门控聚合 **C7**；显隐规则本地 | **[C7]** | local-only：`touch`/`editor_open`/`builder_mode`/`solution_state`/`placement`/`world` + 显隐 Query | C7 不含 touch/editor，门控仍需外层 | **P1**

`on_virtual_press` | **12** | bevy | **1** 部分对齐 C7（缺 `simulation`）；可仍聚 C7 并忽略未用字段，或保持局部门控 | **[C7]**（建议，含未用 `simulation`） | local-only：`touch`/`editor_open`/`placement`/`windows`/控件 Query/`runtime`/`input`/`On<Press>` | 引入未用 `simulation` 增加调度依赖 | **P1**

`on_virtual_drag` | **9** | bevy | **3** 无 C7；touch/editor 门控本地 | **[]** | local-only：拖拽与摇杆节点 Query | 低 | **P3**

`on_virtual_click` | **9** | bevy | **1** 完整命中 C7 | **[C7]** | local-only：`touch`/`editor_open`/控件 Query/`input`/`On<Click>` | 低 | **P1**

`on_editor_control_click` | **11** | bevy | **3** 多按钮 Query 可按 marker 合并为少 Query，或 accept 分派 | **[]** | local-only：编辑器态 + 五类按钮 Query + `commands` | Query 合并可能触发 filter 冲突 | **P2**

---

### save/

`update_save_list_styles` | **11** | bevy | **3** 样式刷新本地；与 rows/scroll 共享门控见缺口 | **[]** | local-only：`save_state`/`hover`/`render_state` + 样式 Query | 低 | **P2**

`update_save_list_rows` | **10** | bevy | **3** 行重建本地 | **[]** | local-only：行宿主 Query/`commands`/`render_state` | 低 | **P2**

`update_save_list_scroll` | **10** | bevy | **3** 滚动本地 | **[]** | local-only：双轴 scroll Query + wheel | 低 | **P3**

`dispatch_save_list_actions` | **9** | bevy | **2** 已消息驱动 | **[]** | local-only：存档选择/忙碌/封面 Query | 低 | **P3**

---

### hud.rs

`update_hud_visibility` | **9** | bevy | **3** 接近 C7 但缺 `ui_runtime`，勿硬套；本地脏检查即可 | **[]** | local-only：`builder_mode`/`save_state` + HUD Query/`Local` | 误用 C7 会多订阅 `ui_runtime` | **P3**

---

## 本领域命中的注册表 ID 统计

| ID | 命中函数数 | 函数 |
|---|---|---|
| **C4** PanelCloseDeps | **5** | `panel_close_clicked`, `dismiss_playing_overlay`, `dismiss_start_menu_overlay`, `dismiss_modals_and_host_panels`, `start_menu_escape` |
| **C7** GameplayPlayGate | **3**（+1 建议） | `on_virtual_click`, `sync_virtual_remote_visibility`,（建议）`on_virtual_press` |
| **SessionStateParams** | **1** | `dispatch_pause_menu_clicks` |
| C1 / C2 / C3 / C5 / C6 / C8 | **0** | — |
| PlacementQueries / HoverPreviewDeps / EditHistoryApply / PlayingWorldParams | **0** | — |

未命中但相关：`inventory_hotbar_digit_input` 仅 typing 子集，**不**标 C4；`update_hud_visibility` 缺 `ui_runtime`，**不**标 C7。

---

## 候选缺口（跨 ≥3 函数、注册表无对应名）

1. **字段**：`State<GameMode>` + `StartMenuScreen`  
   出现于：`update_save_list_rows` / `update_save_list_scroll` / `update_save_list_styles`（另：`sync_start_menu_mounts` 同构）

2. **字段**：`TouchProfile` + `VirtualLayoutEditorOpen`  
   出现于：`on_virtual_press` / `on_virtual_drag` / `on_virtual_click` / `sync_virtual_remote_visibility`