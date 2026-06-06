# Panel 规范

**Panel 体系统一重构** 的目标清单与分类。  
A–E **均为 Panel**，目标状态下统一动态挂载到 UiTree；**不含** HUD、附属浮层、底层 Primitives（见 `UI_ARCHITECTURE.md` §2）。当前迭代落地范围见 §0。

*最后更新：2026-06-07*

---

## 0. 当前迭代落地状态

- **已动态挂载试点**：C1 Settings，经 `UiHost::mount_settings` spawn，关闭时 despawn。
- **Legacy**：除 Settings 外，其余 Panel 本轮仍可保留预 spawn + 显隐路径，后续按本规范逐步迁移。
- **模态过渡态**：Confirm / TextPrompt 已纳入 `UiHost` 和 `UiAction` 路由，当前仍复用既有 Bevy 节点树与状态资源作为 view state。

---

## 1. 统一 Panel 模型

| 属性 | 约定 |
|------|------|
| 生命周期 | `ui.open` mount，`ui.close` / callback 内关闭 unmount |
| Config | 每个 Panel 自有 **`XxxPanelConfig`**，字段由该 Panel 契约决定（可为 `{}`） |
| 数据 | 见 **§2 Config 模式**；**不**在 `build` 中读世界 |
| 交互 | 控件 emit `PanelMessage` → callback；**callback 无 World 参数**，靠 open 处捕获 |
| 返回值 | **无**；`on_confirm: \|\|` 等，不 `take_result` |

**不采用** `ModalScrim`。

---

## 2. Config 模式（与 `UI_ARCHITECTURE.md` §4 对应）

| 模式 | Config 示例 | Panel |
|------|-------------|-------|
| **自包含** | `PuzzleListPanelConfig {}` | A2、A3（列表自读磁盘） |
| **上下文 + Callback** | `PausePanelConfig { save_kind, on_exit: \|\|, ... }` | B1（World 在 open 处捕获） |
| **纯 Callback（模态）** | `ConfirmPanelConfig { on_confirm: \|\|, ... }` | D1、D2 |
| **显式 Props + Callback** | `GeneratorPanelConfig { period, on_change: \|patch\|, ... }` | E 类（patch 为 UI payload） |

---

## 3. Panel 分类

| 类 | 场景 | Config 模式 | 注册 |
|----|------|-------------|------|
| **A** | 主菜单流程 | 自包含 / 极简 | `PanelRegistry` |
| **B** | 游玩中流程 | 上下文 + Callback | `PanelRegistry` |
| **C** | 全局配置 | 视 Panel 定 | `PanelRegistry` |
| **D** | 模态 | 纯 Callback | `PanelRegistry` |
| **E** | 方块配置 | Props + Callback | `EditableBlock` |

---

## 4. A — 主菜单流程 Panel

`GameMode::StartMenu` 下使用；通过命令式 `ui.open` 切换，**不**用 `StartMenuScreen` 显隐切屏（迁移后）。

| ID | `PanelId`（建议名） | 名称 | 当前源码 | 打开来源示例 |
|----|---------------------|------|----------|--------------|
| A1 | `MainMenu` | 主菜单 | `screens/menu.rs` | 启动时 `ui.open` |
| A2 | `PuzzleList` | 谜题列表 | `save_list.rs`（拆分） | `ui.open(PanelId::PuzzleList, PuzzleListPanelConfig {})` |
| A3 | `SolutionList` | 解法列表 | `save_list.rs`（拆分） | `ui.open(..., SolutionListPanelConfig { puzzle_name })` 等 **极简字段** |

### A2 / A3 自包含约定

| Panel | 调用方传入 | Panel 内部负责 |
|-------|------------|----------------|
| **A2** | `{}` | 读磁盘谜题列表、新建/重命名/删除/加载、`ui.open(A3)` / D 类 |
| **A3** | 如 `puzzle_name: String`（**不传**解法条目列表） | 读磁盘解法列表、CRUD、加载进游玩 |

**A1 打开 A2** 时主菜单 **零业务知识**：

```rust
ui.open(PanelId::PuzzleList, PuzzleListPanelConfig {});
```

二者为 **独立 Panel**；禁止 `WorldEntryMode` 单屏双模式。

---

## 5. B — 游玩流程 Panel

`GameMode::Playing` 下使用。

| ID | `PanelId` | 名称 | 当前源码 | 打开来源 |
|----|-----------|------|----------|----------|
| B1 | `Pause` | 暂停菜单 | `screens/menu.rs` | Esc → 见下方 Config |
| B2 | `Inventory` | 背包 | `screens/inventory.rs` | 背包键 |

### B1 `PausePanelConfig`（上下文 + Callback）

调用方（快捷键 system）在 **已持有 World/Commands 的上下文** 里 `ui.open`；callback **`||`**，捕获 `commands` / `ui` / 预先算好的 `dirty`：

```rust
fn open_pause_menu(ui: &mut UiTree, commands: &mut Commands, save_kind: SaveKind, dirty: bool) {
    ui.open(PanelId::Pause, PausePanelConfig {
        save_kind,
        on_exit: move || {
            if dirty {
                ui.open(PanelId::Confirm, ConfirmPanelConfig {
                    on_confirm: move || {
                        commands.queue(|world| { /* session */ });
                        ui.close_all();
                    },
                    on_cancel: move || { ui.close(PanelId::Confirm); },
                    // title / message ...
                });
            } else {
                commands.queue(|world| { /* session */ });
                ui.close(PanelId::Pause);
            }
        },
        on_save: move || {
            commands.queue(|world| session::save_current_world_in_world(world));
        },
    });
}
```

- **`save_kind`**：仅影响按钮文案/显隐。
- **UiTree 不向 callback 传 `World`**；世界操作靠 **open 处闭包捕获**。

---

## 6. C — 全局配置 Panel

| ID | `PanelId` | 名称 | 当前源码 | 打开来源 |
|----|-----------|------|----------|----------|
| C1 | `Settings` | 设置 | `screens/settings.rs` | A1 / B1 设置按钮 |

Config 模式待定（可能自包含读设置文件，或 callback 写键位）；**不传**整个世界状态。

---

## 7. D — 模态 Panel

**纯 Callback Config**；`on_confirm: ||` 等，无 `Result` 回传。

| ID | `PanelId` | 名称 | 当前实现 | 打开来源示例 |
|----|-----------|------|----------|--------------|
| D1 | `Confirm` | 二次确认 | `ConfirmDialogState` | 删存档、回主菜单、重置解法 |
| D2 | `TextPrompt` | 文本输入 | `TextPromptState` | 新建谜题/解法、重命名、另存为 |

**纯 Callback Config**；关闭 = 执行 `on_confirm` / `on_cancel`，无 `Result` 回传给 open 调用栈。

| Panel | Config 字段示例 |
|-------|-----------------|
| D1 | `title`, `message`, `on_confirm`, `on_cancel`, `on_extra?` |
| D2 | `title`, `default_value`, `on_save`, `on_cancel` |

```rust
ui.open(PanelId::Confirm, ConfirmPanelConfig {
    title: "...",
    message: "...",
    on_confirm: move || { /* 捕获 world/commands 后直接做事 */ },
    on_cancel: move || { ui.close(PanelId::Confirm); },
});
```

**已移除**：~~ModalScrim~~

---

## 8. E — 方块配置 Panel

每方块类型 **一个独立 Panel**；UI 定义在 `EditableBlock`（推荐 `world/blocks/<block>/ui.rs`）。

| ID | 方块类型 | 当前文件 | 主要配置项 | 备注 |
|----|----------|----------|------------|------|
| E1 | `Generator` | `generator.rs` | 周期、输出材料 | |
| E2 | `Goal` | `goal.rs` | 目标材料 | |
| E3 | `Stamper` | `stamper.rs` | stamp 颜色 | **独立 Panel**，不共用 Labeler |
| E4 | `Roller` | `roller.rs` | stamp 颜色 | **独立 Panel** |
| E5 | `Converter` | `converter.rs` | 输入 / 输出材料 | |
| E6 | `TeleportEntrance` | `teleport_entrance.rs` | 名称、配对 | 可与 Exit 共用定义或拆分 |
| E7 | `TeleportExit` | `teleport_exit.rs` | 名称、配对 | |

**Props + Callback**；gameplay 读世界填 Config，**不传 `pos`**：

```rust
fn open_generator_panel(ui: &mut UiTree, commands: &mut Commands, period: u32, material: MaterialKind) {
    ui.open(PanelId::Generator, GeneratorPanelConfig {
        period,
        output_material: material,
        material_choices: ...,
        on_change: move |patch| {
            commands.queue(move |world| { apply_generator_patch(world, patch); });
        },
        on_close: move || { ui.close(PanelId::Generator); },
    });
}
```

**迁移后废弃**：`UiPanelId`、`UiPanelContext::Block { pos }`、预 spawn 窗体。

---

## 9. 完整 Panel 清单

| # | ID | 类 | 名称 |
|---|-----|-----|------|
| 1 | A1 | A | 主菜单 |
| 2 | A2 | A | 谜题列表 |
| 3 | A3 | A | 解法列表 |
| 4 | B1 | B | 暂停（`PanelId::Pause`） |
| 5 | B2 | B | 背包 |
| 6 | C1 | C | 设置 |
| 7 | D1 | D | 二次确认 |
| 8 | D2 | D | 文本输入 |
| 9 | E1 | E | 发电机 |
| 10 | E2 | E | 验收器 |
| 11 | E3 | E | 印花机 |
| 12 | E4 | E | 滚刷器 |
| 13 | E5 | E | 转换器 |
| 14 | E6 | E | 传送入口 |
| 15 | E7 | E | 传送出口 |

**合计：15 个 Panel**（重构范围）。

---

## 10. 打开方式矩阵

| Panel | Config 模式 | 调用示例 |
|-------|-------------|----------|
| A2 | 自包含 | `PuzzleListPanelConfig {}` |
| A3 | 自包含 + 极简 | `SolutionListPanelConfig { puzzle_name }` |
| B1 | Callback | `PausePanelConfig { save_kind, on_exit, ... }` |
| D1 | Callback | `ConfirmPanelConfig { on_confirm, ... }` |
| E* | Props + Callback | `GeneratorPanelConfig { period, on_change, ... }` |

---

## 11. 新增 Panel 检查清单

### A–D 类

1. 定义 `XxxPanelConfig` 并标明 **§2 哪种模式**
2. `panels/<name>.rs` + `registry.rs` 一行
3. 自包含型：内部读 Panel 域数据；Callback 型：调用方在 open 时填 callback

### E 类

1. 方块 `ui.rs`：`GeneratorPanelConfig` + `panel_definition()`
2. gameplay 填 Config 后 `ui.open`；**禁止** `pos`、禁止 build 读世界

---

## 12. 与旧实现对应

| 旧 | 新 |
|----|-----|
| `StartMenuScreen::Main` / `SaveList` | A1 / A2 / A3 独立 Panel |
| `playing_ui.paused` / `inventory_open` | B1 / B2 `ui.open` |
| `UiRuntime.open(Settings, …)` | C1 `ui.open` |
| `ConfirmDialogState` / `TextPromptState` | D1 / D2 |
| `UiPanelId` + `open_block(pos)` | E* `ui.open(PanelId, props 由调用方填)` |
| `ModalScrim` | **删除** |
| `open_then` / `take_result` | `ConfirmPanelConfig { on_confirm, ... }` |
| 主菜单传 SaveList 数据 | `PuzzleListPanelConfig {}` |
| `PauseMenu` + 巨型 actions | `PausePanelConfig { on_exit, ... }` |

细节见 `docs/report/panel.md`（旧实现报告）。

---

## 13. 快速判定

| 问题 | 答案 |
|------|------|
| 这是不是 Panel？ | 在 A–E 清单内 → 是；HUD/准星 → 否（本次不 refactor） |
| UI 能否调 session / 读世界？ | **否** |
| 业务写在哪？ | `ui.open` 传入的 handler（调用方 / gameplay） |
| 谜题列表要传 entries 吗？ | **否**，`PuzzleListPanelConfig {}` |
| 暂停菜单怎么传世界？ | **不传**；只传 `save_kind` 等 + callback |
| UI 会 return 结果吗？ | **否**，`on_confirm: \|\|` |
| UiTree 会传 World 给 callback 吗？ | **否**，open 处捕获 |
| E 类 `on_change` 带什么参？ | 仅 **UI patch**，不带 World |
| 谜题和解法列表是一个 Panel 吗？ | **否**，A2 / A3 两个 |
| 有没有遮罩层？ | **无** |
