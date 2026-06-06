# UI 架构

> 项目处于早期阶段，可接受 breaking change。

*最后更新：2026-06-07*

---

## 1. 目录结构

```
src/game/
  block_editing/              # BlockPanelAction、dropdown 语义、markers
    action.rs                   # label_key、mutates_world
    dropdown.rs                 # toggle_action、selected_label、selected_material
    context.rs / markers.rs

  ui/
    core/                       # UiRuntime、ConfirmDialogState、TextPromptState、InlineTextEditState
    features/
      menu/                     # types + actions + update（动态 label）
      save/                     # types + view + actions + update + prompt + confirm
      settings/                 # types（GAMEPLAY_SETTINGS 表）+ actions + update
      block_panels/             # actions + update
      inventory/                # actions + render + types（InventoryTitleText）
    screens/                    # 各屏 spawn
    systems/                    # 跨域通用：panels、localized、status、hud、font、hover
    layout.rs / widgets.rs
```

**原则**：spawn 在 `screens/` / `layout`，handler + update 在同 feature 目录；同一 Action 的 label / enabled / visible 用 `view()` 或 trait 单点定义，禁止平行 `foo_label` + `foo_enabled` 函数。

**核心规则**：`core/` 只做通用 shell（状态 + marker + update），业务语义（title、message、按钮文案、handler）放在 `features/*/`，禁止在通用层 `match` 上层调用场景。

---

## 2. 各域内聚方式

| 域 | 类型 | 视图语义 | Handler | Update |
|----|------|----------|---------|--------|
| **存档列表** | `SaveListAction` | `button_view(ctx)` | `save/actions.rs` | `save/update.rs` |
| **存档列 rebuild** | `SaveListColumn` | load/rename/delete/create | — | `save/update.rs` |
| **确认框（通用）** | `ConfirmDialogState` | `open(ConfirmOpen { title, message, confirm_text, cancel_text, extra? })` | core 点击 → `resolve()` | `take_result()` → `Confirmed / Cancelled / Extra(tag)` |
| **确认框（业务）** | 各域 `*ConfirmPending` | 调用方组 `ConfirmOpen` 文案 | 各域 `apply_*_confirm` match pending+result | — |
| **文本输入（通用）** | `TextPromptState` | `open(TextPromptOpen { title, default_value, save_text, cancel_text })` | core 点击/键盘 → `submit()` / `cancel()` | `take_result()` → `Saved(value) / Cancelled` |
| **文本输入（业务）** | `SaveTextPromptPending` | 调用方 open + 设 pending | `save/actions::apply_save_text_prompt` | — |
| **设置** | `SettingsAction` / `SettingsDropdown` | `UiActionLabel` + `dropdown.trigger_label()` + `action.tab_selected()` | `settings/actions.rs` | `settings/update.rs` |
| **菜单** | `MenuAction` | `label()` + `pause_menu_visible()` | `menu/actions.rs` | `menu/update.rs` |
| **Block Panel** | `BlockPanelAction` | `label_key()` in action.rs | `block_panels/actions.rs` + block `handle_edit_action` | `block_panels/update.rs` |
| **Block Panel 标题** | `BlockPanelTitle` marker | 按 block kind 选 i18n key | — | `block_panels/update.rs` |
| **Block Dropdown** | `BlockPanelDropdown` | `selected_label()` / `toggle_action()` in dropdown.rs | — | `block_panels/update.rs` |
| **背包** | `InventorySlot`（无 Action enum） | 数据驱动 | `inventory/actions.rs` | `inventory/render.rs` |
| **背包标题** | `InventoryTitleText` marker | 按 builder mode 格式化 | — | `inventory/render.rs` |
| **存档列表标题** | `SaveListTitleText` marker | `save_list_title(...)` | — | `save/update.rs` |

---

## 3. 新增控件检查清单

1. 在 feature 的 `types.rs` 加 enum variant（或表驱动项如 `SettingsItem`）
2. 在同文件或 `view.rs` 加 **一处** 视图语义（`view()` / `UiActionLabel` / descriptor method）
3. 在 `screens/` 或 `layout` spawn，用 **域专属 marker 组件**（如 `SaveListTitleText`），勿用全局 `PanelTextKind`
4. 在 `features/*/actions.rs` 加 handler arm
5. 若需每帧刷新，在 **同 feature 的 update/render** 扩展（勿在 `systems/localized.rs` 加业务 special-case）

---

## 4. 2026-06-07 重构记录

### 已完成

- 删除 `systems/save_dialogs.rs` → `features/save/update.rs` + `view.rs`
- `SaveListAction` 三 match 合并为 `button_view()`
- `SaveListColumn` 替代硬编码 load/rename/delete 函数指针
- **ConfirmDialog** 拆为 `core/confirm_dialog.rs`（通用 shell）+ `save/confirm.rs`（`SaveConfirmAction` + session）
- **TextPrompt** 拆为 `core/text_prompt.rs` + `save/prompt.rs`（`SaveTextPromptAction`）
- 删除全局 `PanelTextKind`，改为各域 marker（`SaveListTitleText`、`InventoryTitleText`、`ConfirmTitleText` 等）
- `localized.rs` 只做 `i18n.text(key)`；菜单动态 label 迁入 `MenuAction::label()` + `menu/update.rs`
- Block Panel 标题用 `BlockPanelTitle` marker，去掉 LocalizedText 字符串比对 hack
- `InlineTextEditState.field` 改为 `&'static str`，core 不再依赖 `BlockPanelTextKind`
- 删除 `systems/settings.rs` → `features/settings/update.rs`
- `MenuAction::pause_menu_visible`（从 `status.rs` 迁入）
- 背包 handler/render 迁入 `features/inventory/`
- `BlockPanelDropdown` 语义方法（`toggle_action`、`selected_label` 等）
- 各 feature Plugin 自行注册 update 系统

### 仍可后续改进

- Block Panel 业务 handler 仍分散在 `world/blocks/*.rs`（可考虑注册表）
- `PanelVisibility` 仍含 `StartMenuScreen` / `SettingsTab` 等业务枚举（显隐编排层，优先级低）
- 确认框打开后切换语言时 message 字符串不会随 i18n 重算（可让 session 在 update 时重建 spec）

---

## 5. 保留的设计

- `UiRuntime` + `UiPanelBinding` + `update_panel_visibility`
- Observer 处理点击（Bevy 0.18）
- `widgets.rs` 通用控件
- `GAMEPLAY_SETTINGS` 表驱动 spawn
- `InlineTextEditState` 统一 inline 文本输入
