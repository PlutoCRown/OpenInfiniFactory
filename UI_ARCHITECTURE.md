# UI 架构问题与重构方向

> 记录当前 UI 子系统的结构性问题，供后续批量重构参考。  
> 项目处于早期阶段，可接受 breaking change，不必考虑旧数据兼容。

---

## 1. 概述

当前 UI 采用 Bevy 0.18 的 Observer（`add_observer`）+ 每帧 Update 系统刷新显示的混合模式。  
功能上可以工作，但随着带 UI 的方块和功能增加，**扩展成本会非线性上升**。

主要问题分两类：

1. **方块编辑面板（Block Panel）**：中央分发 + 全局 enum 膨胀，Teleport 还另开了一套路径。
2. **全局 UI 组织**：Action 定义、UI 生成（spawn）、显示刷新（renderer/update）、点击处理（handler）四者分离且归属不一致。

---

## 2. 已核实：Action 与 Renderer 分离，且 Action 定义集中堆放

### 2.1 结论：**属实**

各 UI 区域的 **Action 枚举**、**实体生成**、**每帧刷新**、**点击/输入处理** 分布在不同文件，没有按功能模块内聚。  
更严重的是：**所有 Action 类型几乎都堆在 `src/game/ui/types.rs` 一个文件里**（约 800+ 行），与具体 UI 屏幕/面板所在目录无关。

### 2.2 当前分层（实际文件分布）

| 层次 | 职责 | 主要位置 |
|------|------|----------|
| **Action 定义** | 按钮/控件上挂的 `Component` 枚举 | `src/game/ui/types.rs`（全部集中） |
| **Spawn（布局生成）** | 启动时 `spawn_*` 创建 UI 实体树 | `src/game/ui/screens/*.rs`、`layout.rs`、`widgets.rs` |
| **Renderer（显示刷新）** | 每帧根据 Resource/World 更新 Text、Node、图标等 | `src/game/ui/systems/*.rs` |
| **Handler（交互响应）** | Observer 或 Update 系统处理点击/键盘 | **多数在** `src/game/systems/menus.rs`（1400+ 行），少数在 `ui/systems/` |

Handler 与 Renderer **不在同一模块**，Action 定义又与两者都不在同一模块——改一个 UI 功能往往要跨 3～4 个文件。

### 2.3 各 UI 区域对照表

| UI 区域 | Action 枚举（types.rs） | Spawn | Renderer / Update | Handler |
|---------|-------------------------|-------|-------------------|---------|
| 主菜单 / 暂停菜单 | `MenuAction` | `screens/menu.rs` | `systems/panels.rs`（可见性） | `systems/menus.rs` → `menu_actions` |
| 存档列表 | `SaveListAction` | `screens/save_list.rs` | `systems/save_dialogs.rs` → `update_save_list_ui` | `systems/menus.rs` → `save_list_actions` |
| 文本输入框 | `TextPromptAction` | `screens/save_list.rs` | `systems/save_dialogs.rs` → `update_text_prompt_ui` | `systems/menus.rs` → `text_prompt_actions` / `text_prompt_input` |
| 确认对话框 | `ConfirmDialogAction` | `layout.rs` | `systems/save_dialogs.rs` → `update_confirm_dialog_ui` | `systems/menus.rs` → `confirm_dialog_actions` |
| 设置面板 | `SettingsAction` | `screens/settings.rs` | `systems/settings.rs`（多个 `update_settings_*`） | `systems/menus.rs` → `settings_action_clicked` / `settings_menu_actions` |
| 背包 / 快捷栏 | （无独立 Action，用 `InventorySlot`） | `screens/inventory.rs` | `systems/inventory_render.rs` | `systems/inventory_actions.rs` → `inventory_slot_clicks` |
| 方块编辑面板 | `BlockEditAction`、`TeleportAction` | `layout.rs` + `widgets.rs` | `systems/block_panels.rs` + 多个 `update_*_ui` | `menus.rs` → `block_edit_actions` / `teleport_menu_actions` / `teleport_rename_input` |
| 面板拖拽/关闭 | （无 Action 枚举） | `components/panel.rs` | `systems/panels.rs` | `systems/panels.rs`（**少数内聚较好的例子**） |
| 通用按钮 hover | （无 Action 枚举） | `widgets.rs` | — | `components/button.rs` |

### 2.4 Observer 注册也分散

| 注册位置 | 注册的 Handler |
|----------|----------------|
| `src/game/mod.rs` | `menu_actions`、`save_list_actions`、`slider_self_update` |
| `src/game/ui/mod.rs` | `block_edit_actions`、`teleport_menu_actions`、`confirm_dialog_actions`、`settings_action_clicked`、`text_prompt_actions`、`inventory_slot_clicks`、面板拖拽、按钮 hover 等 |

**同类 UI 的 observer 没有统一注册点**，需要在两个 Plugin 里分别查找。

### 2.5 `types.rs` 作为「全局 junk drawer」

除 Action 枚举外，`types.rs` 还混合存放：

- `UiRuntime`、`OpenBlockPanelDropdown` 等 Resource
- `BlockPanelDropdown`、`PanelVisibility` 等 Component / 标记类型
- `SettingsField`、`SettingsTab` 等设置相关类型
- `UiActionLabel` trait 及其实现
- 大量与具体 UI 区域强相关的类型

结果是：**types.rs 既是类型定义中心，又缺乏按功能域的边界**。

### 2.6 对比：相对内聚的例外

- **背包**：`inventory_actions.rs`（handler）与 `inventory_render.rs`（renderer）同在 `ui/systems/`，只是 Action 仍无独立 enum。
- **面板拖拽/关闭**：spawn、handler、部分 visibility 逻辑都在 `panels.rs` / `components/panel.rs` 附近。

说明项目并非不能做好模块内聚，只是**大部分 UI 没有遵循同一模式**。

---

## 3. 方块编辑面板（Block Panel）专项问题

### 3.1 两套动作路径并存

| | Generator / Goal / Converter / Labeler | Teleport |
|---|--------------------------------------|----------|
| 按钮组件 | `BlockEditAction` | `TeleportAction` |
| 点击入口 | `block_edit_actions` | `teleport_menu_actions` |
| 业务逻辑 | 方块文件 `handle_edit_action` | `menus.rs` 内联 |
| 额外输入 | 无 | `TeleportRenameState` + `teleport_rename_input` |

Teleport 的 entrance/exit 实现了 `EditableBlock::ui_panel()`，但**未实现** `handle_edit_action`（使用 trait 默认空实现），编辑逻辑游离在方块模块之外。

### 3.2 全局 enum 随方块数量膨胀

每增加一种面板控件，通常需要修改：

- `UiPanelId`（`state.rs`）
- `BlockEditAction` 或另起新 Action enum（`types.rs`）
- `BlockPanelDropdown`（`types.rs`）
- `BlockPanelTextKind`（`types.rs`，目前仅 Period / TeleportName）
- `layout.rs` 的 `spawn_*_panel`
- `block_panels.rs` 的 `update_block_panel_dropdowns_ui`（巨型 match）
- 可能还有独立的 `update_xxx_ui` 系统

### 3.3 `update_block_panel_dropdowns_ui` 已是上帝系统

单一系统负责：所有 dropdown 标签、material 图标、dropdown 定位、Teleport pair 动态 despawn/spawn。  
Teleport pair 与其他 dropdown（启动时静态生成选项）使用**两种不同生命周期**，共享代码中充满特判（如 `block_dropdown_toggle_action` 对 `TeleportPair` 返回 `None`）。

### 3.4 各 `update_*_ui` 系统价值不均

| 系统 | 实际作用 | 问题 |
|------|----------|------|
| `update_generator_ui` | 只刷新 period 数字 | material 由 dropdown 系统负责，职责分裂 |
| `update_converter_ui` | 将 `ConverterInputRow` 设为 `Display::Flex` | 近乎占位，不像真正的数据驱动刷新 |
| `update_labeler_ui` | 比对 i18n 字符串，动态改面板标题 | Stamper/Roller 共用 `UiPanelId::Labeler`，靠运行时 hack 区分 |
| `update_teleport_ui` | 刷新名称 + 编辑态光标 | 依赖独立的 `TeleportRenameState` |

### 3.5 依赖方向：`world` → `ui`

`src/game/world/blocks/*.rs` 直接 `use crate::game::ui::{BlockEditAction, BlockPanelDropdown, UiPanelId}`。  
方块域依赖 UI 域，导致 UI action 变更会波及所有方块文件，方块模块无法独立描述「可编辑语义」。

### 3.6 `block_edit_actions` 与世界重建耦合

修改 material/period 等设置后可能触发整世界 mesh 重建；dropdown 切换则通过 `block_edit_action_mutates_world` 白名单排除。  
UI 动作与渲染副作用的边界靠手工维护，扩展新 action 时容易遗漏或过度重建。

### 3.7 UI 临时状态错放层级

`TeleportRenameState` 定义在 `game/state.rs`，与 `PlacementState` 等核心游戏状态并列，但被 `gameplay_input`、`panel_close_clicked`、`update_teleport_ui` 等多处引用——本质是 **UI 输入态**，不是游戏 simulation 状态。

---

## 4. 其他全局 UI 问题

### 4.1 `menus.rs` 职责过载

`src/game/systems/menus.rs` 同时包含：主菜单、存档列表、确认框、文本输入、设置、方块 panel 等 handler，以及若干辅助函数和世界重建逻辑。  
**名称叫 menus，实际是整个游戏 UI 的交互中枢**，与 `ui/` 目录形成「ui 管显示、menus 管逻辑」的不对称分工。

### 4.2 文本输入两套实现

- **Modal 文本输入**：`TextPromptState` + `text_prompt_input` + `text_prompt_actions`（存档命名等）
- **Inline 文本输入**：`TeleportRenameState` + `teleport_rename_input`（传送门重命名）

两者行为相似（Enter 确认、Escape 取消、Backspace、字符输入），但没有共用抽象；`gameplay_input` 等处需分别判断「是否在打字」。

### 4.3 启动时生成全部 UI

`setup_playing_ui` 一次性 spawn 所有方块 panel 及全部 dropdown 层（material/color 等静态列表；Teleport pair 为空壳后动态填充）。  
方块和 dropdown 类型增多时，实体数量和启动时工作量会持续增长（早期可接受，中期需评估按需生成或按 panel 模块化 spawn）。

---

## 5. 修复方向（建议分阶段）

### 阶段 A：文档化约束（立即可做，低成本）

在新增 UI 功能前约定：

1. **禁止**再新增全局 Observer（如 `xxx_menu_actions`）和全局 RenameState。
2. **禁止**在 `types.rs` 无节制追加 Action variant；新功能优先在对应 screen/panel 模块内定义 Action。
3. 新方块必须实现 `handle_edit_action`（或等价 panel handler），不允许 trait 默认空实现蒙混。

### 阶段 B：模块内聚（中等改动，优先于大规模重写）

目标：**每个 UI 功能域自包含 action + spawn + render + handler**。

建议目录形态（示例）：

```
src/game/ui/
  panels/                    # 或 screens/ 下按域分子目录
    block_panels/
      mod.rs                 # Plugin：注册 observer + update systems
      types.rs               # BlockPanelAction、BlockPanelDropdown（从全局 types 迁出）
      layout.rs              # spawn_*_panel
      update.rs              # 合并 update_*_ui + update_block_panel_dropdowns_ui
      actions.rs             # block_edit_actions、inline edit input
    settings/
      mod.rs
      types.rs               # SettingsAction、SettingsField
      layout.rs
      update.rs
      actions.rs
    save_list/
      ...
    inventory/
      ...                    # 已有 actions/render 分离，补 types 即可
  shared/                    # 真正跨域共用：UiRuntime、PanelWindow、widgets 基础件
```

同步迁移：

- 将 `menus.rs` 中各 UI 域 handler **拆到对应 `ui/panels/*/actions.rs`**
- Observer 注册集中到各子模块的 Plugin，或在 `GameUiPlugin` 中显式 `add_plugins(BlockPanelsPlugin, SettingsPanelPlugin, ...)`
- **`types.rs` 拆散**：只保留真正全局的 `UiRuntime`、`UiPanelId` 等

### 阶段 C：方块 Panel 域模型（与阶段 B 并行或紧随其后）

1. **统一动作路径**：Teleport 收编进 panel handler；删除 `TeleportAction` / `teleport_menu_actions` 或改为 panel 内部 enum。
2. **Per-panel Action enum**：如 `GeneratorPanelAction`、`TeleportPanelAction`，由统一 dispatcher 按 `UiPanelId` 路由，而非全部塞进 `BlockEditAction`。
3. **`EditableBlock` 收紧**：去掉默认空 `handle_edit_action`，或拆成 `HasUiPanel` + `HandlesPanelAction` 两个 trait。
4. **反转依赖**：`world/blocks` 不 import `ui`；panel 编辑语义通过 `BlockSettings` 变更 API 或 `game/block_editing` 中间层表达。
5. **Dropdown 统一策略**：用 descriptor / callback 描述「选项从哪来、如何渲染」，消除 `block_panels.rs` 里的 Teleport 特判；静态列表与动态列表走同一接口。
6. **Inline 文本编辑泛化**：`InlineTextEditState { panel, pos, buffer, field }` 替代 `TeleportRenameState`；与 `TextPromptState` 共享输入处理核心。
7. **合并 refresh 系统**：单个 `update_active_block_panel`，内部按 `UiPanelId` dispatch，替代多个薄 `update_xxx_ui` + 一个 mega dropdown 系统。

### 阶段 D：渲染副作用解耦（可后置）

- 方块 settings 变更通过 event 或 explicit `WorldRevision` 通知渲染层，而不是在 UI handler 里直接 `rebuild_world`。
- 细化「哪些 panel action 需要重建 mesh / 哪些只改数据」的声明（可在 panel action 或 block trait 上标注）。

---

## 6. 批量重构时的建议顺序

1. **拆 `types.rs`** + **迁出 `menus.rs` handler**（改善导航性，不改变行为）
2. **Teleport 对齐 Block Panel 主路径**（消除双轨）
3. **合并 block panel update 系统** + **Dropdown 抽象**
4. **统一文本输入**（TextPrompt + InlineEdit）
5. **world/ui 依赖反转**
6. **世界重建与 UI 解耦**

每步完成后跑 `bun scripts/log_rs_lines.js` 记录行数变化；提交信息写「为什么重构」，而非改了哪些文件。

---

## 7. 参考：现有可保留的设计

以下方向正确，重构时应保留而非推翻：

- `UiRuntime` + `open_block(panel, pos)`：追踪当前编辑对象
- `UiPanelBinding` + `update_panel_visibility`：面板显隐
- `EditableBlock` 思路：方块声明 panel、处理编辑（需 enforce）
- Observer 处理 UI 点击：符合 Bevy 0.18 惯例
- `widgets.rs` 中的通用控件（material icon slot、dropdown 容器等）
- `screens/` 与 `systems/` 分离 spawn / update 的思路（需在**同一功能域目录**下完成内聚，而非跨 `menus.rs`）

---

*最后更新：2026-06-07*
