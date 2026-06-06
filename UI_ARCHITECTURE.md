# UI 架构

> 项目处于早期阶段，可接受 breaking change。  
> 本文描述**目标架构**与**当前迭代应完成的功能**；实现以本文为准逐步迁移。

*最后更新：2026-06-07*

---

## 1. 设计目标

1. **动态挂载**：Panel / 模态框按需 `mount` / `unmount`，而不是全部预 spawn 后只靠显隐切换（常驻 HUD 除外）。
2. **UI 只传数据**：View 层只接收 `Props`、只发出 `UiAction`（意图 ID），**不写业务逻辑**。
3. **业务外置**：存档、读档、切场景等一律经 `session::*`；Controller 负责把 `UiEvent` 翻译成 session 调用。
4. **声明式 + 命令式流程**：组件写法接近 React（props + 事件 ID）；`openDialog` 类流程用 `open_then` 表达「等用户操作后再继续」（语义上等价 `await`，Bevy 主循环内用 callback 实现）。
5. **与 Bevy 共存**：底层仍是 Entity + Component + Observer；不引入 Dioxus 等完整第二套 UI 栈（除非日后单独评估）。

参考：`STYLE.tsx`（理想 API 草图）。

---

## 2. 三层模型

```
┌──────────────────────────────────────────────────────────┐
│  Controller（features/*、session）                        │
│  - 决定打开哪个 View、如何处理 UiEvent                      │
│  - 唯一调用 session::* 的 UI 边界                           │
└─────────────────────────┬────────────────────────────────┘
                          │ UiCommand（打开/关闭/更新 Props）
                          │ UiEvent（用户意图，纯数据）
┌─────────────────────────▼────────────────────────────────┐
│  UiHost（core/host.rs，由 UiRuntime 演进）                  │
│  - mount(ViewSpec) → UiInstanceId                         │
│  - unmount(id) / update_props(id, props)                  │
│  - open_then(id, |event, world| { ... })                  │
│  - 栈 / 模态层 / z-index 编排                               │
└─────────────────────────┬────────────────────────────────┘
                          │ ViewProps
┌─────────────────────────▼────────────────────────────────┐
│  View（widgets + features/*/view）                          │
│  - spawn / patch Bevy 节点树                                │
│  - 点击 → emit UiAction { instance, kind }                  │
│  - 不 import session，不 match 业务                         │
└──────────────────────────────────────────────────────────┘
```

### View（纯展示 + 意图）

- 输入：`ViewProps`（字符串、按钮列表、列表项数据等）。
- 输出：`UiAction`（例如 `Dialog(Confirm)`、`List(SaveListAction::LoadPuzzle(name))`）。
- **禁止**：在 View 内闭包调用 `session`、读写 `SaveState` 做业务决策。

### UiHost（挂载中心）

对应 `STYLE.tsx` 中的 `UI_tree`：

| API | 语义 |
|-----|------|
| `mount(spec: ViewSpec) -> UiInstanceId` | 创建实例，spawn 或从 pool 取子树 |
| `unmount(id)` | despawn，从栈移除 |
| `set_props(id, props)` | 增量更新文案/列表，避免整树重建 |
| `open_then(id, handler)` | 模态关闭后一次性回调（已有 Confirm / TextPrompt 模式） |

`ViewSpec` 枚举（逐步扩展）：

```rust
enum ViewSpec {
    Confirm(ConfirmProps),
    TextPrompt(TextPromptProps),
    // 后续逐步扩展：
    // SaveList(SaveListProps),
    // Settings(...),
    // BlockPanel(BlockPanelProps),
    // ...
}
```

### Controller（业务编排）

- 位于 `features/*/actions.rs` 或薄封装模块。
- 例：暂停菜单点「返回主菜单」→ Controller 调 `dialog.open_then(...)` → handler 里 `session::exit_to_main_menu_in_world`。
- **Save 新建谜题** 等：Controller 调 `open_new_puzzle_prompt()`，prompt 内部 `open_then` + `session::create_new_puzzle_in_world`。

### 全局 `ui()` / `i18n()`（`ui/access.rs`）

- `I18n`、`UiHost` 等仍是 Bevy `Resource`；插件注册资源，**`bind_ui_scope` 系统**在每帧 UI 区间绑定当前 `World`。
- 业务与 Controller **禁止**把 `UiHostCommands`、`&I18n` 当作函数参数传递；统一：

```rust
use crate::game::ui::access::{i18n, ui};

ui.open_confirm_then(props, on_complete);
i18n.t("button.save");
i18n.fmt("save.world", &[("name", name)]);
i18n.set_language(language);
```

- 所有需要调用 `i18n` / `ui` 的 Update 系统须加入 `UiAccessScope` 集合（在 `bind_ui_scope` 与 `unbind_ui_scope` 之间）。
- Startup / layout spawn 在 exclusive 系统内先 `bind_ui_scope(world)` 再 spawn。
- 语言切换通过 `I18nRevision` 资源通知 UI 刷新，系统不再注入 `Res<I18n>`。
- `UiHostCommands` 仅作为 `access` 内部 `SystemState` 实现细节，不暴露给 feature 代码。

---

## 3. 事件与数据流

### 统一事件（目标）

逐步将 `MenuAction`、`SaveListAction`、`ConfirmButtonId` 等收敛为：

```rust
struct UiAction {
    instance: UiInstanceId,
    kind: UiActionKind,
}

enum UiActionKind {
    Menu(MenuAction),
    SaveList(SaveListAction),
    Dialog(DialogButtonId),
    TextPromptSubmit { value: String },
    TextPromptCancel,
    PanelClose,
    // BlockPanel(...), Settings(...), ...
}
```

短期可保留各域 enum，由 Host 在 dispatch 时包一层 `instance` id；长期统一路由到一个 `dispatch_ui_action` 系统。

### 模态流程（已实现，需纳入 Host）

`ActiveConfirmDialog::open_then` / `ActiveTextPrompt::open_then`：

1. View 关闭 → `take_result()`
2. `dispatch_*_completion` → 执行 Controller 注册的 `FnOnce`
3. 业务在 callback 里跑，不在 core 里 `match` 场景

Confirm / TextPrompt 应成为 UiHost 管理的 **`ViewSpec` 实例**，而不是独立的全局 Resource 特例。

---

## 4. Panel 与 View 的关系

- **Panel**：带 `PanelWindow` 的可拖动弹窗（见 `docs/spec/panel.md` 分类）。
- **Screen**：主菜单 / 存档列表等整屏流程，由 `StartMenuScreen` 等状态驱动；可预挂载或随 Host 动态挂载。
- **Modal**：Confirm、TextPrompt；由 `UiHost` 阻塞下层业务 action，不使用 `ModalScrim`。
- **HUD**：热键栏、准星等，**不**走 UiHost 栈，数据驱动显隐即可。

完整 Panel 清单与分类见 **`docs/spec/panel.md`**。

---

## 5. 目录结构（目标）

```
src/game/ui/
  core/
    host.rs          # UiHost、ViewSpec、UiInstanceId、mount/unmount/open_then
    runtime.rs       # 由 UiRuntime 合并或委托给 host
    confirm_dialog.rs
    text_prompt.rs
    panel.rs         # PanelWindow、拖动、可见性 primitive
    ...
  features/
    menu/            # Controller + MenuAction + update
    save/
    settings/
    block_panels/
    inventory/
  view/              # （可选）跨 feature 的通用 ViewProps 定义
  screens/           # 薄 spawn 入口，逐步改为 host.mount 调用
  widgets.rs
  layout.rs          # 仅 UiRoot / PlayingUiRoot 骨架 + HUD
```

**原则**：

- spawn 细节在 View / widgets；Controller 只组 Props。
- 同一 Action 的 label / visible / enabled 单点定义（`view()` / trait），禁止平行散落函数。
- `core/` 不含业务 `match`；`features/*` 不含 Bevy 节点结构细节。

---

## 6. 本次迭代应完成的功能

按优先级排列，完成后 UI 具备「Host + 纯数据 View + 外置 Controller」的骨架。

### P0 — 规范落地（文档 + 类型草图）

- [x] 本文档与 `docs/spec/panel.md` 定稿
- [x] 在 `core/` 增加 `host.rs`：`UiInstanceId`、`ViewSpec`（先含 `Confirm`、`TextPrompt`）、`UiHost` Resource 骨架
- [x] 明确 `UiAction` / `UiEvent` 命名与 `features` 边界（注释或空 enum 即可）

### P1 — 模态纳入 Host

- [x] Confirm：`ConfirmOpen` 重命名为/alias 为 `ConfirmProps`；经 `UiHost::mount` + `open_then` 打开，去掉「单例 Resource 特例」路径
- [x] TextPrompt：同上
- [x] `dispatch_confirm_completion` / `dispatch_text_prompt_completion` 并入或委托 `UiHost::dispatch_completions`

### P2 — Panel 动态挂载（择一 Panel 试点）

- [x] 选 **Settings** 或 **BlockPanel（Generator）** 做第一个「mount 时 spawn、despawn 时销毁」试点
- [x] 其余 Panel 仍可用预 spawn + 显隐，文档标注为 legacy，逐步迁移

### P3 — 事件路由统一

- [x] 新增 `dispatch_ui_action`（或扩展 Host），observer 只 emit `UiAction`
- [x] `menu/actions`、`save/actions` 改为根据 `instance + kind` 分发，session 调用留在 handler 内

### 不在本次范围

- 完整 VDOM / Dioxus 集成
- 所有 Panel 一次性改为动态 spawn
- BSN / `bevy_widgets` 迁移
- `.bsn` 资产工作流

---

## 7. 与 session 的边界

| 层 | 允许 |
|----|------|
| View | 读 i18n、格式化展示；emit `UiAction` |
| UiHost | 管理实例生命周期、回调队列、栈 |
| Controller / features | 读 Write `SaveState` 等；调 `session::*` |
| session | 世界/存档/导航；**不**依赖具体 Widget |

命令式 API 约定：

- 有 `Commands`：`session::load_world(&mut commands, ...)`
- 在 `open_then` callback：`session::*_in_world(world, ...)`

---

## 8. 当前代码基线（迁移起点）

已实现、迁移时保留语义：

- `UiRuntime` 栈 + `UiPanelBinding` + `update_panel_visibility`
- `open_then`（Confirm / TextPrompt）+ `Pending*Handler`（NonSend）
- `features/*` 分域 actions + update
- Observer 点击（Bevy 0.18 picking）
- `widgets.rs`、`GAMEPLAY_SETTINGS` 表驱动

待废弃模式（迁移中逐步删除）：

- 预 spawn 全部 Panel 仅切显隐（非 HUD）
- **Legacy**：除 Settings 外，其余 Panel 暂仍可预 spawn + 显隐，后续逐步迁移到 `UiHost::mount`。
- 各域分散的 pending session resource（已删 SaveConfirm / MenuConfirm；勿再新增）
- 在 `core` 内 `match` 上层业务场景

---

## 9. 相关文档

| 文档 | 内容 |
|------|------|
| `docs/spec/panel.md` | Panel 分类与完整清单（规范） |
| `docs/report/panel.md` | 当前实现细节与源码索引（报告） |
| `STYLE.tsx` | 理想 React 风格 API 示例 |
