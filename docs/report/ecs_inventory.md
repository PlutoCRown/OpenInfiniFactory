# ECS 清单（OpenInfiniFactory）

统计基准：仓库当前 `src/` 代码，Bevy **0.18.1**。

说明：

- **E（Entity 类型）**：按「运行时挂载的**项目自定义 Component 集合**相同」归类。每条 E 下列出**自定义 C**；实际 Entity 还会挂 Bevy 内置组件（如 `Transform`、`Node`、`Button`、`Mesh3d` 等），见文末「通用 Bevy 组件」。
- **C**：`#[derive(Component)]` 的自定义类型，共 **93** 个。
- **S**：通过 `add_systems` 注册的系统函数，共 **95** 个（另 **22** 个 Observer，见 S-Observer 节）。
- **R**：`Resource` / `NonSendResource`，含手动 `impl Resource`，共 **58** 个。

---

## 一、Entity 类型（E）

共 **42** 种（按自定义 Component 组合划分；UI 动作 marker 不同但结构相同的，合并为「带 Action Marker 的按钮」一类）。

### 1. 相机

#### E01 · UI 相机

- `UiCamera`

#### E02 · 玩家 / 游戏 3D 相机

- `FlyCamera`
- `GameplayCamera`
- `GameplayScene`

---

### 2. 游戏场景固定物件（`GameplayScene` 标记）

#### E03 · 点光源

- `GameplayScene`

#### E04 · 平行光

- `GameplayScene`

#### E05 · 悬停方块高亮框

- `HoverMarker`
- `GameplayScene`

#### E06 · 瞄准面高亮

- `AimFaceHighlight`
- `GameplayScene`

#### E07 · 放置预览占位

- `PlacementPreview`
- `GameplayScene`

---

### 3. 世界方块（3D）

#### E08 · 标准方块渲染体

- `BlockEntity`

#### E09 · 线材 / 纯 Parts 根节点（无根 Mesh）

- `BlockEntity`

#### E10 · 编辑预览方块

- `BlockEntity`
- `EditPreview`

#### E11 · 生成器 pending 预览方块

- `BlockEntity`
- `PendingGeneratedPreview`

#### E12 · 带移动/旋转/缩放动画的方块

- `BlockEntity`
- `AnimatedBlock`

#### E13 · 带活塞动画的方块（根实体）

- `BlockEntity`
- （子节点见 E15）

#### E14 · 方块模型根节点（子 Entity）

- （无项目 marker；`Transform` + `Visibility`）

#### E15 · 活塞杆动画零件

- `AnimatedPusherRod`

#### E16 · 工厂调试叠加层（子 Entity）

- `FactoryDebugOverlay`

#### E17 · 离屏图标 — 灯光

- `BlockIconRenderEntity`
- `BlockIconRenderRoot`

#### E18 · 离屏图标 — 相机

- `BlockIconRenderEntity`
- `BlockIconRenderRoot`
- `BlockIconRenderCamera`

#### E19 · 离屏图标 — 方块模型

- `BlockIconRenderEntity`
- `BlockIconRenderRoot`
- （可选 `BlockEntity` 视 spawn 路径而定）

---

### 4. 模拟特效（临时 Entity）

#### E20 · 焊接火花

- `WeldSpark`

#### E21 · 激光束 burst

- `LaserBeamBurst`

---

### 5. 调试

#### E22 · 调试文字面板

- `DebugPanel`
- `DebugText`

---

### 6. UI 根节点

#### E23 · 主菜单 UI 根

- `UiRoot`

#### E24 · 游戏内 UI 根

- `PlayingUiRoot`

#### E25 · UiHost 动态挂载容器

- `UiHostMountRoot`

---

### 7. UI 面板壳

#### E26 · 可拖拽面板窗口

- `PanelWindow`
- `PanelVisibility`
- `PanelPosition`
- `PanelTitleBar`
- `PanelCloseButton`
- `UiPanelBinding`

#### E27 · 方块面板绑定节点（无窗口壳，仅绑定）

- `UiPanelBinding`

#### E28 · 确认对话框面板

- `PanelVisibility`（值为 `ConfirmDialog`）

#### E29 · 文字输入对话框根

- `TextPromptRoot`

#### E30 · 保存列表面板根

- `SaveListPanel`

---

### 8. UI 控件与文本

#### E31 · 带动作 Marker 的按钮

下列 **Action Component 之一**（互斥，每种按钮一种）：

- `MenuAction`
- `SaveListAction`
- `SaveListCloseButton`
- `SaveListPrompt`
- `SettingsAction`
- `InventorySlot`
- `ConfirmButtonId`
- `TextPromptButtonId`
- `GeneratorAction`
- `ConverterAction`
- `GoalAction`
- `LabelerAction`
- `TeleportAction`
- `KeyBindingButton`
- （及通过 `UiActionLabel` trait 挂载的其它 Action 型 Component）

#### E32 · 带 hover 样式的按钮

- `HoverButton`
- （通常与 E31 的 Action marker 叠加）

#### E33 · 本地化文本

- `LocalizedText`

#### E34 · 状态栏文本

- `StatusText`

#### E35 · 背包标题

- `InventoryTitleText`

#### E36 · 滚动区域容器

- `ScrollContainer`

#### E37 · 滚动内容

- `ScrollContent`

#### E38 · 滑条填充 / 旋钮（通用）

- `SliderFill` 或 `SliderKnob`

#### E39 · 设置滑条填充 / 旋钮

- `SettingsSliderFill` 或 `SettingsSliderKnob`

#### E40 · 设置项文本 / 数值

- `SettingsText` 或 `SettingsValueText`

#### E41 · 设置下拉框 UI 节点

- `SettingsDropdownLabel` 或 `SettingsDropdownList` 或 `SettingsDropdownRow`

#### E42 · HUD 可见性标记节点

- `Crosshair` 或 `InGameHudVisibility` 或 `InGameHudStyle` 或 `GameplayHudVisibility` 或 `CarriedItemPreview` 或 `InventoryTooltip`

---

### 方块面板专用 UI 节点（挂在 E26/E27 子树下）

以下每种为独立 Entity，Component 组合唯一，不单独编号，列举全部 marker：

| 面板                 | 专用 Component                                                                                                                           |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Generator            | `GeneratorPeriodText`, `GeneratorMaterialList`, `GeneratorMaterialSlot`, `GeneratorMaterialOption`                                       |
| Converter            | `ConverterInputRow`, `ConverterInputList`, `ConverterInputSlot`, `ConverterOutputList`, `ConverterOutputSlot`, `ConverterMaterialOption` |
| Goal                 | `GoalMaterialList`, `GoalMaterialSlot`, `GoalMaterialOption`                                                                             |
| Stamper / Labeler    | `LabelerPanelTitle`, `LabelerColorLabel`, `LabelerColorList`                                                                             |
| Teleport             | `TeleportNameText`, `TeleportPairLabel`, `TeleportPairList`, `TeleportPairOption`                                                        |
| Save List            | `SaveListTitleText`, `SaveListPuzzleColumn`, `SaveListSolutionColumn`                                                                    |
| Confirm / TextPrompt | `ConfirmTitleText`, `ConfirmMessageText`, `TextPromptText`                                                                               |

---

### 通用 Bevy 组件（未列入 E 差异项）

以下组件常随 E 类型一起出现，但由 Bevy 提供，**不参与**上表 E 分类：

`Transform`, `GlobalTransform`, `Visibility`, `InheritedVisibility`, `ViewVisibility`, `Node`, `Button`, `Interaction`, `Text`, `TextFont`, `TextColor`, `TextLayout`, `BackgroundColor`, `BorderColor`, `BorderRadius`, `ZIndex`, `GlobalZIndex`, `Camera`, `Camera2d`, `Camera3d`, `Projection`, `Mesh3d`, `MeshMaterial3d`, `PointLight`, `DirectionalLight`, `RenderLayers`, `Pickable`, `IsDefaultUiCamera`, `Msaa`, `Hdr`, `Name`, …

---

## 二、Component 全集（C）

共 **93** 个自定义 Component（字母序）。

| #   | 名称                      | 定义文件                              |
| --- | ------------------------- | ------------------------------------- |
| 1   | `AimFaceHighlight`        | `game/world/rendering.rs`             |
| 2   | `AnimatedBlock`           | `game/world/animation.rs`             |
| 3   | `AnimatedPusher`          | `game/world/animation.rs`             |
| 4   | `AnimatedPusherRod`       | `game/world/animation.rs`             |
| 5   | `BlockEntity`             | `game/world/rendering.rs`             |
| 6   | `BlockIconRenderCamera`   | `game/world/rendering.rs`             |
| 7   | `BlockIconRenderEntity`   | `game/world/rendering.rs`             |
| 8   | `BlockIconRenderRoot`     | `game/world/rendering.rs`             |
| 9   | `CarriedItemPreview`      | `game/ui/types.rs`                    |
| 10  | `ConfirmButtonId`         | `game/ui/core/confirm_dialog.rs`      |
| 11  | `ConfirmMessageText`      | `game/ui/core/confirm_dialog.rs`      |
| 12  | `ConfirmTitleText`        | `game/ui/core/confirm_dialog.rs`      |
| 13  | `ConverterAction`         | `game/blocks/converter/ui.rs`         |
| 14  | `ConverterInputList`      | `game/blocks/converter/ui.rs`         |
| 15  | `ConverterInputRow`       | `game/blocks/converter/ui.rs`         |
| 16  | `ConverterInputSlot`      | `game/blocks/converter/ui.rs`         |
| 17  | `ConverterMaterialOption` | `game/blocks/converter/ui.rs`         |
| 18  | `ConverterOutputList`     | `game/blocks/converter/ui.rs`         |
| 19  | `ConverterOutputSlot`     | `game/blocks/converter/ui.rs`         |
| 20  | `Crosshair`               | `game/ui/types.rs`                    |
| 21  | `DebugPanel`              | `game/systems/debug.rs`               |
| 22  | `DebugText`               | `game/systems/debug.rs`               |
| 23  | `EditPreview`             | `game/world/rendering.rs`             |
| 24  | `FactoryDebugOverlay`     | `game/world/rendering.rs`             |
| 25  | `FlyCamera`               | `game/player/controller.rs`           |
| 26  | `GameplayCamera`          | `game/cameras.rs`                     |
| 27  | `GameplayHudVisibility`   | `game/ui/types.rs`                    |
| 28  | `GameplayScene`           | `game/world/rendering.rs`             |
| 29  | `GeneratorAction`         | `game/blocks/generator/ui.rs`         |
| 30  | `GeneratorMaterialList`   | `game/blocks/generator/ui.rs`         |
| 31  | `GeneratorMaterialOption` | `game/blocks/generator/ui.rs`         |
| 32  | `GeneratorMaterialSlot`   | `game/blocks/generator/ui.rs`         |
| 33  | `GeneratorPeriodText`     | `game/blocks/generator/ui.rs`         |
| 34  | `GoalAction`              | `game/blocks/goal/ui.rs`              |
| 35  | `GoalMaterialList`        | `game/blocks/goal/ui.rs`              |
| 36  | `GoalMaterialOption`      | `game/blocks/goal/ui.rs`              |
| 37  | `GoalMaterialSlot`        | `game/blocks/goal/ui.rs`              |
| 38  | `HoverButton`             | `game/ui/components/button.rs`        |
| 39  | `HoverMarker`             | `game/world/rendering.rs`             |
| 40  | `InGameHudStyle`          | `game/ui/types.rs`                    |
| 41  | `InGameHudVisibility`     | `game/ui/types.rs`                    |
| 42  | `InventorySlot`           | `game/ui/types.rs`                    |
| 43  | `InventoryTitleText`      | `game/ui/features/inventory/types.rs` |
| 44  | `InventoryTooltip`        | `game/ui/types.rs`                    |
| 45  | `KeyBindingButton`        | `game/ui/types.rs`                    |
| 46  | `LabelerAction`           | `game/blocks/stamper/ui.rs`           |
| 47  | `LabelerColorLabel`       | `game/blocks/stamper/ui.rs`           |
| 48  | `LabelerColorList`        | `game/blocks/stamper/ui.rs`           |
| 49  | `LabelerPanelTitle`       | `game/blocks/stamper/ui.rs`           |
| 50  | `LaserBeamBurst`          | `game/world/animation.rs`             |
| 51  | `LocalizedText`           | `game/ui/types.rs`                    |
| 52  | `MenuAction`              | `game/ui/features/menu/types.rs`      |
| 53  | `PanelCloseButton`        | `game/ui/core/panel.rs`               |
| 54  | `PanelPosition`           | `game/ui/core/panel.rs`               |
| 55  | `PanelTitleBar`           | `game/ui/core/panel.rs`               |
| 56  | `PanelVisibility`         | `game/ui/core/panel.rs`               |
| 57  | `PanelWindow`             | `game/ui/core/panel.rs`               |
| 58  | `PendingGeneratedPreview` | `game/world/rendering.rs`             |
| 59  | `PlacementPreview`        | `game/world/rendering.rs`             |
| 60  | `PlayingUiRoot`           | `game/ui/types.rs`                    |
| 61  | `SaveListAction`          | `game/ui/features/save/types.rs`      |
| 62  | `SaveListCloseButton`     | `game/ui/features/save/types.rs`      |
| 63  | `SaveListPanel`           | `game/ui/features/save/types.rs`      |
| 64  | `SaveListPrompt`          | `game/ui/features/save/types.rs`      |
| 65  | `SaveListPuzzleColumn`    | `game/ui/features/save/types.rs`      |
| 66  | `SaveListSolutionColumn`  | `game/ui/features/save/types.rs`      |
| 67  | `SaveListTitleText`       | `game/ui/features/save/types.rs`      |
| 68  | `ScrollContainer`         | `game/ui/components/scroll.rs`        |
| 69  | `ScrollContent`           | `game/ui/components/scroll.rs`        |
| 70  | `SettingsAction`          | `game/ui/features/settings/types.rs`  |
| 71  | `SettingsDropdownLabel`   | `game/ui/features/settings/types.rs`  |
| 72  | `SettingsDropdownList`    | `game/ui/features/settings/types.rs`  |
| 73  | `SettingsDropdownRow`     | `game/ui/features/settings/types.rs`  |
| 74  | `SettingsSliderFill`      | `game/ui/features/settings/types.rs`  |
| 75  | `SettingsSliderKnob`      | `game/ui/features/settings/types.rs`  |
| 76  | `SettingsText`            | `game/ui/features/settings/types.rs`  |
| 77  | `SettingsValueText`       | `game/ui/features/settings/types.rs`  |
| 78  | `SliderFill`              | `game/ui/components/slider.rs`        |
| 79  | `SliderKnob`              | `game/ui/components/slider.rs`        |
| 80  | `StatusText`              | `game/ui/types.rs`                    |
| 81  | `TeleportAction`          | `game/blocks/teleport_entrance/ui.rs` |
| 82  | `TeleportNameText`        | `game/blocks/teleport_entrance/ui.rs` |
| 83  | `TeleportPairLabel`       | `game/blocks/teleport_entrance/ui.rs` |
| 84  | `TeleportPairList`        | `game/blocks/teleport_entrance/ui.rs` |
| 85  | `TeleportPairOption`      | `game/blocks/teleport_entrance/ui.rs` |
| 86  | `TextPromptButtonId`      | `game/ui/core/text_prompt.rs`         |
| 87  | `TextPromptRoot`          | `game/ui/core/text_prompt.rs`         |
| 88  | `TextPromptText`          | `game/ui/core/text_prompt.rs`         |
| 89  | `UiCamera`                | `game/cameras.rs`                     |
| 90  | `UiHostMountRoot`         | `game/ui/core/host.rs`                |
| 91  | `UiPanelBinding`          | `game/ui/core/runtime.rs`             |
| 92  | `UiRoot`                  | `game/ui/types.rs`                    |
| 93  | `WeldSpark`               | `game/world/animation.rs`             |

---

## 三、System 全集（S）

共 **95** 个注册系统（按函数名，字母序）。调度列：`Startup` / `OnEnter` / `OnExit` / `First` / `PreUpdate` / `Update` / `PostUpdate` / `Last`。

| 名称                                             | 调度                                                            |
| ------------------------------------------------ | --------------------------------------------------------------- |
| `animate_blocks`                                 | Update                                                          |
| `apply_fov`                                      | Update                                                          |
| `apply_ui_font`                                  | Update                                                          |
| `begin_perf_frame`                               | First                                                           |
| `bind_ui_scope`                                  | Update                                                          |
| `camera_look`                                    | Update                                                          |
| `camera_move`                                    | Update                                                          |
| `cameras::configure_ui_camera_for_playing`       | OnEnter(Playing)                                                |
| `cameras::configure_ui_camera_for_start_menu`    | OnExit(Playing)                                                 |
| `debug::poll_debug_http`                         | Update（非 wasm + debug_http）                                  |
| `dispatch_inventory_slot_actions`                | Update                                                          |
| `dispatch_menu_actions`                          | Update                                                          |
| `dispatch_save_list_actions`                     | Update                                                          |
| `dispatch_settings_actions`                      | Update                                                          |
| `dispatch_ui_action`                             | Update                                                          |
| `dispatch_ui_host_completions`                   | Update                                                          |
| `draw_hover_structure_bounds`                    | Update                                                          |
| `finish_perf_frame`                              | Last                                                            |
| `gameplay_input`                                 | Update                                                          |
| `handle_create_new_puzzle`                       | Update                                                          |
| `handle_create_new_solution`                     | Update                                                          |
| `handle_exit_to_main_menu`                       | Update                                                          |
| `handle_load_world`                              | Update                                                          |
| `handle_reset_solution`                          | Update                                                          |
| `handle_save_current_world`                      | Update                                                          |
| `handle_save_current_world_invalidate_solutions` | Update                                                          |
| `handle_save_world_as_new_puzzle`                | Update                                                          |
| `handle_switch_to_edit_mode`                     | Update                                                          |
| `on_exit_playing`                                | OnExit(Playing)                                                 |
| `perf_mark_animation`                            | Update                                                          |
| `perf_mark_debug`                                | Update                                                          |
| `perf_mark_input`                                | Update                                                          |
| `perf_mark_last`                                 | Last                                                            |
| `perf_mark_menus`                                | Update                                                          |
| `perf_mark_post_update_start`                    | PostUpdate                                                      |
| `perf_mark_post_update_transform`                | PostUpdate                                                      |
| `perf_mark_post_update_ui`                       | PostUpdate                                                      |
| `perf_mark_post_update_visibility`               | PostUpdate                                                      |
| `perf_mark_pre_update`                           | PreUpdate                                                       |
| `perf_mark_simulation`                           | Update                                                          |
| `perf_mark_ui`                                   | Update                                                          |
| `perf_mark_view`                                 | Update                                                          |
| `placement_input`                                | Update                                                          |
| `prepare_playing_session`                        | OnEnter(Playing)                                                |
| `process_teleport_rename_prompt`                 | Update                                                          |
| `rebuild_playing_world`                          | OnEnter(Playing)                                                |
| `refresh_saves_on_startup`                       | Startup                                                         |
| `retire_block_icon_renderers`                    | Update                                                          |
| `settings_menu_actions`                          | Update                                                          |
| `show_input_row`                                 | Update                                                          |
| `simulation_controls`                            | Update                                                          |
| `sim_bridge::poll_simulation_worker`             | Update                                                          |
| `sim_bridge::tick_simulation`                    | Update                                                          |
| `spawn_player`                                   | OnEnter(Playing)                                                |
| `spawn_ui_camera`                                | Startup                                                         |
| `start_debug_http_server`                        | Startup（非 wasm + debug_http）                                 |
| `sync_block_entity_index`                        | OnEnter(Playing), PostUpdate                                    |
| `sync_cursor_grab`                               | Update                                                          |
| `sync_sim_debug_log`                             | Update                                                          |
| `systems::debug::draw_player_collider`           | Update                                                          |
| `systems::debug::load_debug_font`                | Startup                                                         |
| `systems::debug::setup_debug_ui`                 | OnEnter(Playing)                                                |
| `systems::debug::toggle_debug`                   | Update                                                          |
| `systems::debug::toggle_factory_activity_debug`  | Update                                                          |
| `systems::debug::update_debug_ui`                | Update                                                          |
| `text_prompt_input`                              | Update                                                          |
| `ui::load_ui_font`                               | Startup                                                         |
| `ui::setup_menu_ui`                              | Startup                                                         |
| `ui::setup_playing_ui_system`                    | OnEnter(Playing)                                                |
| `unbind_ui_scope`                                | Update                                                          |
| `update_carried_item_ui`                         | Update                                                          |
| `update_confirm_dialog_ui`                       | Update                                                          |
| `update_dropdowns`                               | Update（×5：converter / generator / goal / stamper / teleport） |
| `update_hover`                                   | Update                                                          |
| `update_hud_visibility`                          | Update                                                          |
| `update_inventory_slots`                         | Update                                                          |
| `update_inventory_title`                         | Update                                                          |
| `update_localized_ui`                            | Update                                                          |
| `update_menu_labels`                             | Update                                                          |
| `update_panel`                                   | Update（×2：generator / teleport）                              |
| `update_panel_visibility`                        | Update                                                          |
| `update_save_list_ui`                            | Update                                                          |
| `update_scroll_containers`                       | Update                                                          |
| `update_settings_dropdowns_ui`                   | Update                                                          |
| `update_settings_slider_drag_ui`                 | Update                                                          |
| `update_settings_sliders_ui`                     | Update                                                          |
| `update_settings_tabs_ui`                        | Update                                                          |
| `update_settings_text_ui`                        | Update                                                          |
| `update_status_ui`                               | Update                                                          |
| `update_text_prompt_ui`                          | Update                                                          |
| `update_title`                                   | Update（stamper）                                               |
| `update_ui_layers`                               | Update                                                          |
| `world::rendering::setup_block_icons`            | OnEnter(Playing)                                                |
| `world::rendering::setup_scene`                  | OnEnter(Playing)                                                |

### S-Observer（事件响应，非 Update 轮询）

共 **22** 个 `add_observer`：

| 名称                          | 说明                          |
| ----------------------------- | ----------------------------- |
| `button_hovered`              | UI 按钮 hover                 |
| `button_pressed`              | UI 按钮按下                   |
| `button_released`             | UI 按钮释放                   |
| `button_unhovered`            | UI 按钮 unhover               |
| `emit_confirm_dialog_actions` | 确认框                        |
| `emit_inventory_slot_actions` | 背包槽                        |
| `emit_menu_actions`           | 主菜单                        |
| `emit_save_list_actions`      | 保存列表                      |
| `emit_settings_actions`       | 设置                          |
| `emit_text_prompt_actions`    | 文字输入框                    |
| `on_click`                    | 方块面板按钮（×5 模块各注册） |
| `panel_close_clicked`         | 面板关闭                      |
| `panel_drag_started`          | 面板拖拽开始                  |
| `panel_dragged`               | 面板拖拽中                    |
| `panel_drag_ended`            | 面板拖拽结束                  |
| `slider_self_update`          | Bevy UI slider                |
| `ui_hovered`                  | 通用 UI hover                 |
| `ui_unhovered`                | 通用 UI unhover               |

---

## 四、Resource 全集（R）

共 **58** 项：其中 **45** 个 `#[derive(Resource)]`，**8** 个手动 `impl Resource`，**2** 个 `NonSendResource`，**3** 个 Bevy 内置 Resource 类型在本项目中 `insert_resource`。

### 4.1 自定义 Resource（derive）

| 名称                        | 定义文件                              |
| --------------------------- | ------------------------------------- |
| `ActiveSettingsSlider`      | `game/ui/features/settings/types.rs`  |
| `BlockIconAssets`           | `game/world/rendering.rs`             |
| `BlockIconRenderState`      | `game/world/rendering.rs`             |
| `BuilderMode`               | `game/state.rs`                       |
| `CarriedItem`               | `game/ui/types.rs`                    |
| `ConfirmDialogState`        | `game/ui/core/confirm_dialog.rs`      |
| `DebugFont`                 | `game/systems/debug.rs`               |
| `DebugHttpBridge`           | `debug_http/embedded.rs`              |
| `DebugState`                | `game/systems/debug.rs`               |
| `GameConfig`                | `shared/config.rs`                    |
| `GameSettings`              | `game/state.rs`                       |
| `HoverStructureBounds`      | `game/world/rendering.rs`             |
| `I18n`                      | `shared/i18n.rs`                      |
| `I18nRevision`              | `game/ui/access.rs`                   |
| `InlineTextEditState`       | `game/ui/core/text_input.rs`          |
| `InventoryItems`            | `game/ui/types.rs`                    |
| `MovementInfluenceCache`    | `game/simulation/structures.rs`       |
| `OpenBlockPanelDropdown`    | `game/block_editing/panel_state.rs`   |
| `OpenSettingsDropdown`      | `game/ui/features/settings/types.rs`  |
| `PanelDragState`            | `game/ui/core/panel.rs`               |
| `PendingGeneratedMaterials` | `oif-sim/simulation/pending.rs`       |
| `PendingKeyBind`            | `game/ui/features/settings/types.rs`  |
| `PendingTeleportRename`     | `game/blocks/teleport_entrance/ui.rs` |
| `PerfStats`                 | `game/systems/perf.rs`                |
| `PlacementState`            | `game/state.rs`                       |
| `PlayingUiRootEntity`       | `game/ui/core/host.rs`                |
| `PlayingUiState`            | `game/state.rs`                       |
| `PusherState`               | `game/simulation/movement.rs`         |
| `SaveListRenderState`       | `game/ui/features/save/types.rs`      |
| `SaveState`                 | `shared/save.rs`                      |
| `SettingsTab`               | `game/ui/features/settings/types.rs`  |
| `SignalNetworkCache`        | `game/simulation/signals.rs`          |
| `SimulationState`           | `game/state.rs`                       |
| `SimulationStepStats`       | `oif-sim/simulation/stats.rs`         |
| `SolutionState`             | `game/state.rs`                       |
| `StartMenuScreen`           | `game/state.rs`                       |
| `StructureState`            | `game/simulation/structure_state.rs`  |
| `TextPromptState`           | `game/ui/core/text_prompt.rs`         |
| `UiFont`                    | `game/ui/systems/font.rs`             |
| `UiHost`                    | `game/ui/core/host.rs`                |
| `UiHoverState`              | `game/ui/core/panel.rs`               |
| `UiRootEntity`              | `game/ui/core/host.rs`                |
| `UiRuntime`                 | `game/ui/core/runtime.rs`             |
| `WorldBlocks`               | `game/world/grid.rs`                  |
| `WorldRenderAssets`         | `game/world/render_assets.rs`         |

### 4.2 手动 `impl Resource`（无 derive）

| 名称                          | 定义文件                                                 |
| ----------------------------- | -------------------------------------------------------- |
| `BlockEntityIndex`            | `scene/entity_index.rs`（impl 于 `game/bevy_bridge.rs`） |
| `LaunchOptions`               | `shared/launch.rs`                                       |
| `SimulationControl`           | `oif-sim/session/control.rs`                             |
| `SimulationDebugLog`          | `oif-sim/session/log.rs`                                 |
| `SimulationPresentationState` | `sim_bridge/present.rs`                                  |
| `SimulationWorker`            | `sim_bridge/worker.rs`                                   |
| `TurnCache`                   | `sim_bridge/cache.rs`                                    |

### 4.3 NonSend Resource

| 名称                       | 定义文件                         |
| -------------------------- | -------------------------------- |
| `PendingConfirmHandler`    | `game/ui/core/confirm_dialog.rs` |
| `PendingTextPromptHandler` | `game/ui/core/text_prompt.rs`    |

### 4.4 Bevy 内置 Resource（本项目 insert）

| 名称                        | 用途         |
| --------------------------- | ------------ |
| `ClearColor`                | 清屏色       |
| `GlobalAmbientLight`        | 环境光       |
| `DirectionalLightShadowMap` | 阴影贴图尺寸 |
| `UiScale`                   | UI 缩放      |

### 4.5 State（特殊 Resource）

| 名称       | 说明                                 |
| ---------- | ------------------------------------ |
| `GameMode` | `States`：`StartMenu` / `Playing` 等 |

---

## 五、汇总

| 类别                              | 数量                  |
| --------------------------------- | --------------------- |
| Entity 类型（E，自定义 C 组合）   | 42                    |
| 自定义 Component（C）             | 93                    |
| System（S）                       | 95                    |
| Observer                          | 22                    |
| Resource（R，含 NonSend + State） | 58 + `GameMode` State |

---

_文档由代码静态扫描生成；若 spawn 路径新增 optional Component 组合，E 类型可能增加。重新统计可搜索 `#[derive(Component)]` 与 `add_systems`。_
