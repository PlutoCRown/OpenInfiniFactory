# 系统复杂度报告

这份报告用于评估当前代码结构下，新增常见类型方块时通常需要修改多少个文件。统计依据包括当前的方块注册表、模拟运行时、UI 面板、国际化、存档/配置模式和渲染资源流程。

## 1. 新增一个材料方块

当前例子：`Material`、`IronMaterial`、`CopperMaterial`。

最少修改文件数：**5 个**

典型修改文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/world/blocks/<new_material>.rs` | 新增 `Block` 实现，并实现 `MaterialBlock` 标记 trait。 |
| `src/game/world/blocks.rs` | 增加模块声明、`BlockKind` 枚举项，通常还要增加 `MaterialKind` 枚举项和名称 key。 |
| `src/game/world/blocks/registry.rs` | 把新方块加入 `ALL_BLOCKS` 和 `BLOCK_REGISTRY`。 |
| `src/game/simulation/runtime.rs` | 增加 `MaterialKind -> BlockKind` 和 `BlockKind -> MaterialKind` 的映射。 |
| `assets/i18n/en.json` / `assets/i18n/zh-CN.json` | 增加完整名称、短名称，以及材料选择器中需要的名称。 |

可能额外修改的文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/world/render_assets.rs` | 只有当该材料需要不同于 `BlockKind::material()` 的自定义贴图或材质时才需要修改。 |
| `src/shared/save.rs` | 只有当新材料引入新的持久化数据时才需要修改。 |

优化建议：

- 把 `MaterialKind -> BlockKind` 和 `BlockKind -> MaterialKind` 这两组重复 match 移到方块元数据里，例如在 `Block` 上提供 `fn material_kind(&self) -> Option<MaterialKind>`。
- 可以用一张材料注册表统一描述 `MaterialKind`、`BlockKind`、本地化 key、颜色和可选贴图，这样新增材料时只需要新增一个方块模块或一条数据记录，再补国际化。
- 如果大多数材料方块逻辑完全一致，可以考虑用通用的 `MaterialBlockDefinition` 数据注册结构，而不是每种材料都写一个 Rust 模块。

## 2. 新增一个有 UI 的系统方块

当前例子：`Generator`、`Converter`、`Stamper` / `Roller` 的标签面板。

最少修改文件数：**10-12 个**

典型修改文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/world/blocks/<new_system>.rs` | 新增系统方块定义和行为声明。 |
| `src/game/world/blocks.rs` | 增加模块声明、`BlockKind` 枚举项、`is_system_layer()` 之类的分类 helper，必要时增加特殊 helper。 |
| `src/game/world/blocks/registry.rs` | 注册到 `EDIT_BLOCKS`、`EDITABLE_BLOCKS`、`ALL_BLOCKS` 和 `BLOCK_REGISTRY`。 |
| `src/game/world/grid.rs` | 存储该方块的逐位置设置，并增加默认值、getter、setter。 |
| `src/game/state.rs` | 如果 UI 是绑定到选中方块的弹窗，需要增加面板打开状态。 |
| `src/game/systems/gameplay.rs` | 玩家交互该方块时打开面板，并在合适时机关闭或清理状态。 |
| `src/game/systems/menus.rs` | 处理 UI 按钮动作，并把设置写回 `WorldBlocks`。 |
| `src/game/ui/types.rs` | 增加面板 marker component、文本 label component 和 action enum。 |
| `src/game/ui/layout.rs` | 生成面板布局和控件。 |
| `src/game/ui/widgets.rs` | 如果现有 widget helper 不够用，需要增加新的按钮或控件生成函数。 |
| `src/game/ui/systems.rs` | 更新动态 UI 文本和面板显示/隐藏逻辑。 |
| `assets/i18n/en.json` / `assets/i18n/zh-CN.json` | 增加方块名称、面板标题、按钮文本和值文本。 |

可能额外修改的文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/shared/save.rs` | 如果设置需要进入存档，必须修改。 |
| `src/game/simulation/runtime.rs` | 如果系统方块会改变模拟状态，需要修改。 |
| `src/game/world/rendering.rs` / `render_assets.rs` | 如果需要自定义连接器渲染或自定义材质，需要修改。 |

优化建议：

- 抽象一个通用的“方块设置面板”，由描述数据驱动：标题 key、行、控件、值读取器和值写入器。当前 Generator、Converter、Labeler 面板结构相似，但实现仍然分散在多处。
- 把每个方块的 UI 行为移动到方块侧元数据，例如 `fn settings_panel(&self) -> Option<BlockSettingsPanelSpec>`。
- 把逐位置设置统一收敛到一个类型化 map 或 `BlockSettings` 枚举里，并按 `BlockKind` 存取，避免每新增一个可配置方块都要在 `grid.rs` 增加新的字段、getter 和 setter。
- 提供可复用的 UI 控件：枚举下拉框、数字滑动条/步进器、关闭按钮等。这样新增面板时不需要同时修改 `types`、`layout`、`widgets`、`systems` 和 `menus` 多个模块。

## 3. 新增一个可以造成模拟期方块移动逻辑的工厂方块

当前例子：`Conveyor`、`ReverseConveyor`、`Lifter`、`Rotator`、`CounterRotator`、`Piston`。

最少修改文件数：**4-5 个**

典型修改文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/world/blocks/<new_factory>.rs` | 新增 `Block` 实现，提供 `material_mover()`，必要时增加信号或渲染行为。 |
| `src/game/world/blocks.rs` | 增加模块声明、`BlockKind` 枚举项；如果是新的移动行为，还要增加 `MaterialMover` 枚举项。 |
| `src/game/world/blocks/registry.rs` | 注册到 `PLAY_BLOCKS`、`ALL_BLOCKS` 和 `BLOCK_REGISTRY`。 |
| `assets/i18n/en.json` / `assets/i18n/zh-CN.json` | 增加完整名称和短名称。 |

如果复用已有的 `MaterialMover` 变体，通常不需要改模拟运行时。如果移动行为是全新的，还需要修改：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/simulation/runtime.rs` | 对新的 `MaterialMover` 变体增加 match 分支并执行逻辑。 |
| `src/game/simulation/structures.rs` | 如果已有移动、碰撞、旋转原语无法表达新逻辑，需要增加新的结构操作 helper。 |

可能额外修改的文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/world/rendering.rs` | 如果需要新的连接器、指示器或特殊渲染行为，需要修改。 |
| `src/game/world/render_assets.rs` | 如果需要新的 mesh 或材质资源，需要修改。 |
| `src/game/world/blocks/<alternate>.rs` | 如果该方块有替换方块，可能还需要新增或修改替换方块模块。 |

优化建议：

- 当前 `MaterialMover` 枚举已经能减少一部分分散逻辑，但执行仍然集中在 `runtime.rs`。如果移动方块继续增加，可以让方块返回 `SimulationOp` 之类的操作描述，再由通用 executor 执行。
- 把常见移动原语保留在 `structures.rs`，例如“移动结构”、“旋转结构”、“通电后推动”。但如果一个数据化的 mover spec 可以表达，就避免每新增一个方块都给 runtime 增加一个专用 match 分支。
- 增加 `MovementSpec` 类型，描述来源选择、位移、是否需要供电、旋转方向等。大部分移动方块可以变成数据配置。
- 可以把“模拟阶段”拆成多个注册点。方块分别贡献 marker、destroyer、source、mover、labeler 等行为，避免 `run_turn` 持续膨胀。

## 4. 新增一个可以六面连接的工厂方块

当前接近例子：`Wire` 可以六面连接；`WeldPoint` 使用 `WeldConnectorBehavior::AllSides`；通电设备使用 `WireConnectorBehavior::Device { blocked_offset }`。

最少修改文件数：**4 个**

典型修改文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/world/blocks/<new_factory>.rs` | 新增方块定义，并通常要提供 `render_behavior()` 和 `signal_behavior()`。 |
| `src/game/world/blocks.rs` | 增加模块声明和 `BlockKind` 枚举项。 |
| `src/game/world/blocks/registry.rs` | 注册到 `PLAY_BLOCKS`、`ALL_BLOCKS` 和 `BLOCK_REGISTRY`。 |
| `assets/i18n/en.json` / `assets/i18n/zh-CN.json` | 增加完整名称和短名称。 |

可能额外修改的文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/simulation/signals.rs` | 只有当信号拓扑规则不同于现有 wire/device/detector 规则时才需要修改。 |
| `src/game/world/rendering.rs` | 只有当连接器渲染需要现有 all-sides wire/device 行为之外的新模式时才需要修改。 |
| `src/game/world/blocks.rs` | 如果现有 `WireConnectorBehavior` 无法表达，需要增加新的连接器枚举项。 |

优化建议：

- `WireConnectorBehavior` 当前有 `Wire` 和 `Device { blocked_offset }`。如果之后会出现更多连接类型，建议增加显式的 `AllSides` 或 `BlockedSides(&'static [IVec3])` 表达方式。
- 尽量让模拟拓扑和视觉渲染复用同一个连接规则。当前信号连接和渲染连接有关联，但仍然分布在不同系统里。
- 把连接规则抽象成声明式 `ConnectorSpec`，包含连接器类型、允许连接的面、阻挡的面，以及是否传递/消耗/产生信号。
- 这样六面连接方块就可以只新增一个方块模块，再补注册和国际化，不需要改 `signals.rs` 或 `rendering.rs`。

## 总体观察

当前系统正在朝正确方向演进：方块本地的行为方法，例如 `material_mover`、`signal_behavior`、`render_behavior`、`alternate`、`is_directional`，已经减少了中心化分支。剩余复杂度主要来自四类横切注册点：

- `BlockKind` 枚举和模块声明。
- `registry.rs` 中的多个数组。
- 国际化文件。
- 可配置方块专用的 map、UI 和存档代码。

最值得优先做的抽象，是一个统一的方块注册宏或数据 builder。它应该能集中生成或描述：

- `BlockKind` 注册数据。
- 放置分类。
- 国际化 key。
- 渲染颜色。
- trait / class。
- 可选模拟行为。
- 可选设置面板 spec。

这样不会牺牲 Rust 的类型安全，但可以让“新增一个方块”尽量局部化，避免在 registry、UI、runtime、save、localization 多个位置反复同步修改。
