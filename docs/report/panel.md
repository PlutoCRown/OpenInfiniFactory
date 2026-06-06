# Panel 报告

本文档统计当前项目中的 Panel 实体，说明各自语义，并区分**全局层面 UI**与**游戏内方块相关 UI**。

*最后更新：2026-06-07*

---

## 结论

| 类别 | 数量 | 挂载根 | 典型可见性驱动 |
| --- | --- | --- | --- |
| 全局 / 菜单层 Panel | 4 | `UiRoot`（Startup） | `GameMode::StartMenu`、`StartMenuScreen`、`UiHost` / `TextPromptState` |
| 游玩流程 Panel | 2 | `PlayingUiRoot`（`OnEnter(Playing)`） | `PlayingUiState`（暂停 / 背包） |
| 方块配置 Panel | 5 | `PlayingUiRoot` | `UiRuntime.open_block` + `UiPanelBinding` |
| 跨层全局配置 | 1（设置） | `UiHost` 动态挂到当前 UI root | `UiHost::mount_settings` / `unmount_panel` |
| 非 Panel 浮层 | 1 组 | `PlayingUiRoot` | `OpenBlockPanelDropdown`（材料 / 颜色 / 传送门配对下拉） |

带 `PanelWindow` 组件的窗体共 **12 个**（含 TextPrompt；Settings 按需动态 spawn）。

---

## 分层架构

```
┌─────────────────────────────────────────────────────────────┐
│  UiRoot（Startup，全程存在）                                  │
│  └─ 主菜单 / 存档列表 / 确认框 / TextPrompt                    │
└─────────────────────────────────────────────────────────────┘
                              │
                    GameMode::Playing
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  PlayingUiRoot（进入 Playing 时 spawn，退出时 despawn）       │
│  ├─ 游玩 HUD（热键栏、准星、状态文字、携带物、tooltip）        │  ← 非 Panel
│  ├─ 暂停菜单 / 背包 Panel                                     │  ← 游玩流程
│  ├─ Generator / Goal / Labeler / Converter / Teleport Panel │  ← 方块配置
│  └─ block 下拉浮层（非独立 Panel 窗体）                       │
└─────────────────────────────────────────────────────────────┘
```

### 1. 全局 / 菜单层

**定义**：与当前编辑/游玩的方块网格无绑定；在 `GameMode::StartMenu` 或跨模式共享的流程中使用。

| Panel | 源码 | 可见条件 | 语义 |
| --- | --- | --- | --- |
| 主菜单 | `screens/menu.rs` | `StartMenu` + `StartMenuScreen::Main` | 编辑谜题、游玩、设置、退出 |
| 存档列表 | `screens/save_list.rs` | `StartMenu` + `StartMenuScreen::SaveList` | 谜题/解法列表、新建、重命名、删除 |
| 设置 | `screens/settings.rs` | `UiHost::mount_settings` 动态 spawn，`UiRuntime` 栈顶为 `Settings` + 当前 `SettingsTab` | Gameplay 滑块 / 键位绑定（840px 宽） |
| 确认框 | `layout.rs` `spawn_confirm_dialog` | `UiHost` 中存在 Confirm 实例 | 存档删除等二次确认 |
| 文本输入 | `layout.rs` `spawn_text_prompt` | `TextPromptState` open | 通用单行输入（存档重命名等） |

**设置 Panel 的特殊性**：实体不再于 Startup 预创建；主菜单或暂停菜单调用 `UiHost::mount_settings` 后，设置面板会挂到对应 root，关闭时 despawn。它是**全局配置 UI**，不是方块配置 UI。

### 2. 游玩层（非方块）

**定义**：仅在 `GameMode::Playing` 下存在，由 `PlayingUiState` 或 HUD 规则驱动，与具体方块坐标无关。

| UI | 类型 | 源码 | 可见条件 | 语义 |
| --- | --- | --- | --- | --- |
| 暂停菜单 | Panel | `screens/menu.rs` | `playing_ui.paused` | 继续、存盘、回主菜单、打开设置等 |
| 背包 | Panel | `screens/inventory.rs` | `playing_ui.inventory_open` | 热键栏 + 背包格子 |
| 热键栏 | HUD 节点 | `screens/inventory.rs` | `GameplayHudVisibility` | 底部快捷栏，非 Panel |
| 准星 / 状态文字 | HUD 文本 | `layout.rs` `spawn_status_overlays` | `InGameHudVisibility` / `GameplayHudVisibility` | 模拟状态、当前存档名等 |
| 携带物预览 / tooltip | HUD 节点 | `screens/inventory.rs` | 数据驱动 | 拖拽方块时的跟随 UI |

### 3. 方块配置层

**定义**：通过 `UiPanelBinding(UiPanelId::…)` 标记；打开时 `UiRuntime` 栈写入 `UiPanelContext::Block { pos }`，内容随 `WorldBlocks::system_blocks` 中该格方块刷新。

| Panel | `UiPanelId` | 对应方块类型 | 配置项 |
| --- | --- | --- | --- |
| 发电机 | `Generator` | `Generator` | 周期、输出材料 |
| 目标 | `Goal` | `Goal` | 目标材料 |
| 标签机 | `Labeler` | `Roller`、`Stamper` |  stamp 颜色 |
| 转换器 | `Converter` | `Converter` | 输入 / 输出材料 |
| 传送门 | `Teleport` | `TeleportEntrance`、`TeleportExit` | 名称、配对 |

**打开路径**：游玩中点击可编辑系统方块 → `gameplay.rs` 调用 `ui_runtime.open_block(panel, pos)`。

**附属 UI（非 Panel 窗体）**：`layout.rs` 的 `spawn_block_dropdown_layers` 为上述 Panel 内的下拉控件提供浮层列表（材料图标、颜色、传送门配对），由 `OpenBlockPanelDropdown` 控制显示，不属于独立 Panel 计数。

---

## 挂载生命周期

| 阶段 | 系统 | 创建内容 |
| --- | --- | --- |
| `Startup` | `ui::setup_menu_ui` | `UiRoot` 及其子节点（菜单、存档列表、确认框、TextPrompt） |
| `OnEnter(Playing)` | `ui::setup_playing_ui_system` | `PlayingUiRoot` 及游玩 HUD、流程 Panel、方块 Panel、下拉层 |
| `OnExit(Playing)` | `session::on_exit_playing` | despawn `PlayingUiRoot`（方块 Panel 与游玩 HUD 一并销毁） |

主菜单、存档列表、Confirm、TextPrompt 在整局应用生命周期内保留；Settings 由 `UiHost` 按需创建/销毁。

---

## 可见性机制

项目用三种方式控制 UI 显示，不要混为一谈：

| 机制 | 标记 / 资源 | 更新系统 | 适用对象 |
| --- | --- | --- | --- |
| 流程可见性 | `PanelVisibility` | `update_panel_visibility` → `panel_visible()` | 主菜单、存档列表、暂停、背包、设置 Tab、确认框 |
| 栈顶 Panel | `UiPanelBinding` + `UiRuntime` | 同上，`active_panel == binding.0` | 设置 + 5 个方块 Panel |
| 独立状态 | `TextPromptState` | `update_text_prompt_ui` | TextPrompt（有 `PanelWindow` 但排除在 `panel_visible` 之外） |

`PanelWindow` 实体在 `display: None` 时还会重置 `PanelPosition`（取消拖动居中），并通过 `Visibility::Hidden` 避免误渲染。

层级：`update_ui_layers` 按 `UiRuntime` 栈深度与 `UiHost` Confirm 实例计算 `GlobalZIndex`。Modal 打开时由业务 action emitter 检查 `UiHost::modal_open()`，不再使用 `ModalScrim` 节点阻挡下层点击。

---

## Panel 完整清单

### 带 `PanelWindow` 的窗体（12）

| # | 名称 | 根 | 分类 | 关闭方式 |
| --- | --- | --- | --- | --- |
| 1 | 主菜单 | UiRoot | 全局 | 切屏 / 进入 Playing |
| 2 | 存档列表 | UiRoot | 全局 | 返回主菜单 |
| 3 | 设置 | UiHost 动态根 | 全局（跨模式） | 关闭按钮 → `UiHost::unmount_panel(Settings)` |
| 4 | 确认框 | UiRoot | 全局 | 按钮 → `UiAction` → `UiHost::dispatch_completions` |
| 5 | TextPrompt | UiRoot | 全局 | 确认 / 取消 → `UiAction` → `UiHost::dispatch_completions` |
| 6 | 暂停菜单 | PlayingUiRoot | 游玩流程 | Esc / 继续 |
| 7 | 背包 | PlayingUiRoot | 游玩流程 | 再按背包键 |
| 8 | Generator | PlayingUiRoot | 方块 | 标题栏 × |
| 9 | Goal | PlayingUiRoot | 方块 | 标题栏 × |
| 10 | Labeler | PlayingUiRoot | 方块 | 标题栏 × |
| 11 | Converter | PlayingUiRoot | 方块 | 标题栏 × |
| 12 | Teleport | PlayingUiRoot | 方块 | 标题栏 × |

### 无 `PanelWindow` 的相关 UI

| 名称 | 组件 | 说明 |
| --- | --- | --- |
| Block 下拉层 | `BlockPanelDropdown` 标记 | 6 组浮层列表，依附方块 Panel |

---

## `UiPanelId` 与方块映射

定义于 `src/game/state.rs`：

```rust
pub enum UiPanelId {
    Settings,    // 全局配置，非方块
    Generator, Goal, Labeler, Converter, Teleport,  // 方块配置
}
```

各方块 `ui_panel()` 实现：

| 方块 | 文件 | 返回 |
| --- | --- | --- |
| Generator | `world/blocks/generator.rs` | `Some(Generator)` |
| Goal | `world/blocks/goal.rs` | `Some(Goal)` |
| Roller / Stamper | `roller.rs` / `stamper.rs` | `Some(Labeler)` |
| Converter | `converter.rs` | `Some(Converter)` |
| TeleportEntrance / Exit | `teleport_*.rs` | `Some(Teleport)` |

---

## 快速区分表

| 问题 | 判断方式 |
| --- | --- |
| 是否全局 UI？ | 挂在 `UiRoot`，或仅用 `PanelVisibility` / `TextPromptState` / `ConfirmDialogState` |
| 是否方块 UI？ | 带 `UiPanelBinding` 且 id 为 Generator / Goal / Labeler / Converter / Teleport |
| 是否游玩 HUD？ | 在 `PlayingUiRoot` 但无 `PanelWindow`，或仅有 `InGameHudVisibility` |
| 设置算哪一层？ | **全局配置**；虽用 `UiPanelBinding`，但 context 为 `SettingsFromStartMenu` / `SettingsFromPause`，不是 `Block { pos }` |

---

## 相关源码

| 路径 | 职责 |
| --- | --- |
| `src/game/ui/layout.rs` | `setup_menu_ui` / `setup_playing_ui`、方块 Panel spawn、确认框、TextPrompt |
| `src/game/ui/screens/` | 主菜单、暂停、背包、设置、存档列表 spawn |
| `src/game/ui/components/panel.rs` | `spawn_panel`、`panel_bundle`（含 `PanelWindow`） |
| `src/game/ui/core/panel.rs` | `PanelVisibility` 枚举 |
| `src/game/ui/core/runtime.rs` | `UiRuntime` 栈、`UiPanelBinding` |
| `src/game/ui/systems/panels.rs` | 可见性、拖动、层级 |
| `src/game/state.rs` | `GameMode`、`PlayingUiState`、`UiPanelId` |
| `UI_ARCHITECTURE.md` | feature 目录与 Action 内聚约定 |
