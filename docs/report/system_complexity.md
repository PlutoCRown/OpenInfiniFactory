# 系统复杂度报告

这份报告用于评估当前代码结构下，新增常见类型方块时通常需要修改多少个文件。统计依据包括当前方块注册表、模拟阶段拆分、UI 面板、国际化、两阶段存档和渲染资源流程。

本报告已按当前结构更新：模拟逻辑已经从 `runtime.rs` 拆到 `gravity.rs`、`movement.rs`、`markers.rs`、`behaviors.rs`；存档已经改为 `Puzzle` / `Solution` 两阶段保存。

## 1. 新增一个材料方块

当前例子：`Material`、`IronMaterial`、`CopperMaterial`。

最少修改文件数：**4 个**

典型修改文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/world/blocks/<new_material>.rs` | 新增 `Block` 实现，并实现 `MaterialBlock` 标记 trait。 |
| `src/game/world/blocks.rs` | 增加模块声明、`BlockKind` 枚举项，通常还要增加 `MaterialKind` 枚举项。 |
| `src/game/world/blocks/registry.rs` | 加入 `ALL_BLOCKS` 和 `BLOCK_REGISTRY`。材料通常不加入编辑或游玩物品栏。 |
| `assets/i18n/en.json` / `assets/i18n/zh-CN.json` | 增加完整名称、短名称，以及材料选择器中需要的名称。 |

可能额外修改的文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/world/render_assets.rs` | 只有当该材料需要不同于通用材料外观的自定义贴图或材质时才需要修改。 |
| `src/shared/save.rs` | 通常不需要。材料方块不保存，只存在于模拟期；只有材料种类进入生成块、转换器等持久化设置时才需要确认序列化兼容。 |

当前复杂度变化：

- 两阶段存档后，材料方块仍然不会进入 `Puzzle` 或 `Solution` 的持久层，所以新增普通材料不再意味着要修改保存的块过滤规则。
- 材料方块已经通过 `Block::material_kind()` 声明自己的 `MaterialKind`，`behaviors.rs` 不再维护材料双向 match。

优化建议：

- 增加 `MaterialBlockBehavior` trait，把 `MaterialKind -> BlockKind` 和 `BlockKind -> MaterialKind` 移到材料方块自身声明中。
- 增加 `BlockCatalogEntry` 或扩展现有 `Block` trait，让材料的本地化 key、颜色、材质资源和 `MaterialKind` 同源注册。
- 如果大多数材料方块逻辑完全一致，可以考虑用通用的 `MaterialBlockDefinition` 数据注册结构，而不是每种材料都写一个 Rust 模块。

## 2. 新增一个有 UI 的系统方块

当前例子：`Generator`、`Converter`、`Stamper` / `Roller`、`TeleportEntrance` / `TeleportExit`。

最少修改文件数：**11-13 个**

典型修改文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/world/blocks/<new_system>.rs` | 新增系统方块定义和行为声明。 |
| `src/game/world/blocks.rs` | 增加模块声明、`BlockKind` 枚举项、`is_system_layer()` 分类，必要时增加特殊 helper。 |
| `src/game/world/blocks/registry.rs` | 注册到 `EDIT_BLOCKS`、`EDITABLE_BLOCKS`、`ALL_BLOCKS` 和 `BLOCK_REGISTRY`。 |
| `src/game/world/grid.rs` | 存储该方块的逐位置设置，并增加默认值、getter、setter。 |
| `src/shared/save.rs` | 如果设置需要持久化，必须把它加入 `WorldLayer`、旧存档兼容迁移、capture/apply 流程。两阶段存档让这里更重要：系统方块设置属于 puzzle 快照，而不是 solution 工厂层。 |
| `src/game/state.rs` | 如果 UI 是绑定到选中方块的弹窗，需要增加面板打开状态。 |
| `src/game/systems/gameplay.rs` | 玩家交互该方块时打开面板，并在合适时机关闭或清理状态。 |
| `src/game/systems/menus.rs` | 处理 UI 按钮动作，并把设置写回 `WorldBlocks`。 |
| `src/game/ui/types.rs` | 增加面板 marker component、文本 label component 和 action enum。 |
| `src/game/ui/layout.rs` | 生成面板布局和控件。 |
| `src/game/ui/systems.rs` | 更新动态 UI 文本和面板显示/隐藏逻辑。 |
| `assets/i18n/en.json` / `assets/i18n/zh-CN.json` | 增加方块名称、面板标题、按钮文本和值文本。 |

可能额外修改的文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/simulation/behaviors.rs` | 如果系统方块会在行为阶段改变材料或世界状态。 |
| `src/game/simulation/movement.rs` | 如果系统方块参与主移动计划。 |
| `src/game/simulation/markers.rs` | 如果系统方块生成派生 marker。 |
| `src/game/simulation/signals.rs` | 如果系统方块引入新的信号拓扑规则。 |
| `src/game/world/rendering.rs` / `render_assets.rs` | 如果需要自定义连接器渲染、模型或材质资源。 |

当前复杂度变化：

- 模拟阶段拆分后，新增系统方块不再默认修改 `runtime.rs`，而是根据行为类型进入 `behaviors.rs`、`movement.rs`、`markers.rs` 或 `signals.rs`。
- 两阶段存档让有配置的系统方块成本上升了一点：配置不仅要能保存，还要能正确进入 `Puzzle` 快照，并随 `Solution` 内置快照一起写入。
- UI 仍然是主要复杂度来源。新增一个有完整设置面板的系统方块，通常仍会横跨 `state`、`gameplay`、`menus`、`types`、`layout`、`systems` 和 i18n。

优化建议：

- 增加 `ConfigurableBlock` trait，统一声明默认设置、读取设置、写回设置和持久化 schema。
- 增加 `BlockSettingsPanel` trait，由方块返回 `SettingsPanelSpec`：标题 key、控件列表、显示值、action 映射。
- 把逐位置设置统一收敛到 `HashMap<IVec3, BlockSettings>`，让 `save.rs` 只序列化一个设置集合。
- 提供 `SettingsControl` 控件描述：枚举下拉框、数字滑动条/步进器、颜色选择器、关闭按钮，避免每个面板新增一组 `types/layout/systems/menus` 代码。

## 3. 新增一个可以造成模拟期移动的工厂方块

当前例子：`Conveyor`、`ReverseConveyor`、`Lifter`、`Rotator`、`CounterRotator`、`Piston`。

最少修改文件数：**4-5 个**

典型修改文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/world/blocks/<new_factory>.rs` | 新增 `Block` 实现，提供 `movement_rule()`，必要时增加信号或渲染行为。 |
| `src/game/world/blocks.rs` | 增加模块声明、`BlockKind` 枚举项；如果是新的移动规则，还要增加 `MovementRule` 枚举项。 |
| `src/game/world/blocks/registry.rs` | 注册到 `PLAY_BLOCKS`、`ALL_BLOCKS` 和 `BLOCK_REGISTRY`。 |
| `assets/i18n/en.json` / `assets/i18n/zh-CN.json` | 增加完整名称和短名称。 |

如果复用已有的 `MovementRule` 变体，通常不需要改模拟执行代码。如果移动行为是全新的，还需要修改：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/simulation/movement.rs` | 对新的 `MovementRule` 变体增加移动标记逻辑。以前这项在 `runtime.rs`，现在已经迁移到移动阶段模块。 |
| `src/game/simulation/structures.rs` | 如果已有平移、旋转、碰撞原语无法表达新逻辑，需要增加新的结构操作 helper。 |

可能额外修改的文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/simulation/signals.rs` | 如果移动方块依赖新的供电规则或连接规则。 |
| `src/game/world/rendering.rs` | 如果需要新的连接器、方向指示器或特殊渲染行为。 |
| `src/game/world/render_assets.rs` | 如果需要新的 mesh 或材质资源。 |
| `src/game/world/blocks/<alternate>.rs` | 如果该方块有替换方块，可能还需要新增或修改替换方块模块。 |

当前复杂度变化：

- `Solution` 现在专门保存工厂方块层，所以普通工厂方块只要 `kind.is_factory()` 成立，就会自动进入 solution 保存，不需要改 `save.rs`。
- 模拟回滚现在保存完整模拟开始快照，因此新增移动工厂方块时，不需要额外处理“停止模拟恢复位置”的保存逻辑。
- 运行期移动逻辑从 `runtime.rs` 拆到 `movement.rs`，职责更清楚；`MovementRule` 已经是规则语义，但 `movement.rs` 里仍有中心化 rule match。

优化建议：

- 增加 `MovesMaterial` trait，由方块返回 `MovementSpec` 或 `MovementRule`，描述来源选择、位移、是否需要供电、旋转方向等。
- 让 `movement.rs` 只做通用执行：遍历实现 `MovesMaterial` 的方块，收集 `StructureMove`，而不是按 `MaterialMover` 变体写大量分支。
- 保留 `structures.rs` 作为结构平移、旋转、碰撞、面标记迁移的唯一公共原语层。

## 4. 新增一个可以执行行为的系统方块

当前例子：`Welder` / `DownWelder`、`Drill` / `Laser`、`Stamper` / `Roller`、`Converter`、`TeleportEntrance` / `TeleportExit`。

最少修改文件数：**5-8 个**

典型修改文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/world/blocks/<new_block>.rs` | 新增行为声明，例如 `material_destroyer()`、`material_labeler()`、`material_source()`、`marker_behavior()`。 |
| `src/game/world/blocks.rs` | 增加 `BlockKind` 和必要的行为枚举变体。 |
| `src/game/world/blocks/registry.rs` | 注册到对应物品栏和全局注册表。 |
| `src/game/simulation/behaviors.rs` | 如果是行为阶段的新机制，需要在这里执行。 |
| `assets/i18n/en.json` / `assets/i18n/zh-CN.json` | 增加名称和 UI 文案。 |

可能额外修改的文件：

| 文件 | 修改原因 |
| --- | --- |
| `src/game/simulation/markers.rs` | 如果行为依赖生成的焊点、钻头头部、阻拦头等派生方块。 |
| `src/game/world/grid.rs` / `src/shared/save.rs` | 如果行为需要逐位置设置并持久化。 |
| `src/game/ui/*` | 如果行为方块有设置面板。 |

当前复杂度变化：

- 行为阶段已经集中到 `behaviors.rs`，不再继续膨胀 `runtime.rs`。
- 但 `behaviors.rs` 会成为新的增长点。新增行为时需要明确它的执行顺序，例如验收、生成、焊接、切削、印花、转换、传送。

优化建议：

- 增加 `SimulationBehavior` trait，明确 `phase()` 和 `run()`，让方块声明自己属于生成、焊接、切削、标记、转换、传送等阶段。
- 定义显式的 `BehaviorPhase` 枚举，避免新增行为时只能靠函数顺序理解执行时机。
- 对可数据化的行为继续使用行为描述，例如 `MaterialDestroyer`、`MaterialLabeler`；只有真正新机制才实现自定义 `SimulationBehavior`。

## 5. 新增一个可以六面连接的工厂方块

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

- 增加 `ConnectableBlock` trait，返回统一的 `ConnectorSpec`。
- `ConnectorSpec` 同时服务信号拓扑和连接器渲染，包含连接器类型、允许连接的面、阻挡的面，以及是否传递、消耗或产生信号。
- `WireConnectorBehavior` 和 `WeldConnectorBehavior` 可以逐步退化为 `ConnectorSpec` 的两个具体配置，而不是两套独立枚举。

## 建议抽象的 trait

下面这些 trait 可以把“新增方块”从多个中心化 match 中解耦出来。建议按优先级逐步做，不需要一次性全改。

当前已落地的部分：

- `Block::material_kind()` 已提供材料方块到 `MaterialKind` 的声明，`behaviors.rs` 不再维护材料反向 match。
- `BlockKind::material_block_kind()` 已通过注册表从 `MaterialKind` 查找材料方块。
- `Block::persistent_layer()` 和 `PersistentLayer` 已提供两阶段存档归属，`save.rs` 不再维护独立的 `is_puzzle_block()` 过滤逻辑。
- `MaterialMover` 已重命名并数据化为 `MovementRule`，移动方块现在贡献移动规则而不是暴露方块类型名。
- `WorldBlocks` 已把多个逐位置设置 map 收敛到 `block_settings: HashMap<IVec3, BlockSettings>`。
- `save.rs` 已新增统一的 `block_settings` 存档列表，同时保留旧字段读取兼容。

### 1. `BlockCatalogEntry`

目的：把注册表、物品栏分类、名称 key、颜色、渲染基础数据统一到方块自身。

建议接口：

```rust
pub trait BlockCatalogEntry: Block {
    fn class(&self) -> BlockClass;
    fn placement_layer(&self) -> PlacementLayer;
    fn inventory_group(&self) -> Option<InventoryGroup>;
    fn i18n(&self) -> BlockI18n;
    fn visuals(&self) -> BlockVisuals;
}
```

可替代的现有分散点：

| 当前位置 | 可迁移内容 |
| --- | --- |
| `registry.rs` | `EDIT_BLOCKS`、`PLAY_BLOCKS`、`ALL_BLOCKS` 的分类来源。 |
| `BlockDefinition` | class、颜色、slot 颜色、shape、collision、transparent。 |
| `BlockKind` helper | `is_factory()`、`is_scene()`、`is_system_layer()` 等可以改为读 catalog。 |

优先级：**高**。这是后续其它 trait 能否干净注册的基础。

### 2. `MaterialBlockBehavior`

状态：**已部分落地**。当前作为 `Block::material_kind()` 默认方法实现，尚未拆成独立 trait。

目的：解决材料新增时还要在 `behaviors.rs` 手写双向映射的问题。

建议接口：

```rust
pub trait MaterialBlockBehavior: Block {
    fn material_kind(&self) -> MaterialKind;
}
```

配套改动：

- `material_block_kind(MaterialKind)` 改成从材料方块注册表查找。
- `block_material_kind(BlockKind)` 改成从 `BlockKind` 对应方块读取 `material_kind()`。
- 生成块、转换器只依赖 `MaterialKind`，不需要知道具体 `BlockKind` match。

优先级：**已执行第一步**。剩余优化是把材料注册表做成显式 catalog，而不是每次遍历 `BLOCK_REGISTRY`。

### 3. `ConfigurableBlock`

状态：**已部分落地**。当前已经有统一 `BlockSettings` 存储和存档 schema，但还没有把默认设置和 schema 声明移动到方块 trait。

目的：统一逐位置设置、默认值、保存和加载，降低有 UI 系统方块的成本。

建议接口：

```rust
pub trait ConfigurableBlock: Block {
    fn default_settings(&self) -> BlockSettings;
    fn settings_schema(&self) -> BlockSettingsSchema;
}
```

配套数据：

```rust
pub enum BlockSettings {
    Generator(GeneratorSettings),
    Labeler(LabelerSettings),
    Converter(ConverterSettings),
    Teleport(TeleportSettings),
}
```

可替代的现有分散点：

| 当前位置 | 可迁移内容 |
| --- | --- |
| `WorldBlocks` | 多个 settings map 已合并为 `HashMap<IVec3, BlockSettings>`。 |
| `save.rs` | 已新增统一 `block_settings` 列表；旧 `generator_settings`、`labeler_settings`、`converter_settings`、`teleport_settings` 仍作为兼容读取字段保留。 |
| `menus.rs` | 设置读取 / 写回可以通过统一 API 完成。 |

优先级：**已执行第一步**。剩余优化是让方块通过 trait 声明默认设置和设置 schema，并让 UI 面板直接消费 schema。

### 4. `BlockSettingsPanel`

目的：让 UI 面板由方块返回描述，而不是每个方块修改 `types/layout/systems/menus` 多处。

建议接口：

```rust
pub trait BlockSettingsPanel: ConfigurableBlock {
    fn panel_spec(&self, settings: &BlockSettings) -> SettingsPanelSpec;
    fn apply_panel_action(&self, settings: &mut BlockSettings, action: SettingsPanelAction);
}
```

建议控件描述：

```rust
pub enum SettingsControl {
    NumberStepper { key: &'static str, value: i64, min: i64, max: i64 },
    EnumCycle { key: &'static str, value_key: &'static str },
    ColorCycle { key: &'static str, color: StampColor },
    TextEdit { key: &'static str, value: String },
    Close,
}
```

可替代的现有分散点：

| 当前位置 | 可迁移内容 |
| --- | --- |
| `state.rs` | 多个 `generator_panel` / `labeler_panel` / `converter_panel` / `teleport_panel` 可以变成一个 `active_block_panel: Option<IVec3>`。 |
| `ui/types.rs` | 多套 action enum 可以合并为通用 `SettingsPanelAction`。 |
| `ui/layout.rs` | 多个面板布局可以合并为一个动态面板。 |
| `ui/systems.rs` | 多套文本更新和可见性系统可以合并。 |
| `menus.rs` | 多套 action handler 可以合并为按 spec 派发。 |

优先级：**中高**。实现量较大，但收益也最大。

### 5. `MovesMaterial`

状态：**已部分落地**。当前作为 `Block::movement_rule()` 默认方法实现，尚未拆成独立 trait。

目的：把移动逻辑从 `MaterialMover` 中心化 match 变成方块贡献移动规则。

建议接口：

```rust
pub trait MovesMaterial: Block {
    fn movement_rule(&self, facing: Facing) -> Option<MovementRule>;
}
```

建议数据：

```rust
pub enum MovementRule {
    Translate {
        source: IVec3,
        offset: IVec3,
    },
    Lift {
        range: i32,
    },
    Rotate {
        clockwise: bool,
    },
    PoweredTranslate {
        source: IVec3,
        offset: IVec3,
    },
}
```

可替代的现有分散点：

| 当前位置 | 可迁移内容 |
| --- | --- |
| `MaterialMover` | 已改成 `MovementRule`。 |
| `movement.rs` | 仍有 rule match，后续可以继续抽成 rule executor。 |

优先级：**已执行第一步**。剩余优化是把 `movement.rs` 的 match 进一步拆成 `MovementRule::to_structure_move()` 或 executor 方法。

### 6. `SimulationBehavior`

目的：把行为阶段做成显式注册点，并表达执行顺序。

建议接口：

```rust
pub trait SimulationBehavior: Block {
    fn behavior_phase(&self) -> BehaviorPhase;
    fn run_behavior(&self, ctx: &mut BehaviorContext, pos: IVec3, block: BlockData);
}
```

建议阶段：

```rust
pub enum BehaviorPhase {
    Acceptance,
    Source,
    Weld,
    Destroy,
    Label,
    Convert,
    Teleport,
}
```

注意：

- 不建议所有行为都立即改成 trait object 回调。简单、稳定、可数据化的行为仍可以保留 `MaterialDestroyer`、`MaterialLabeler` 这种描述式 enum。
- `SimulationBehavior` 更适合转换器、传送器、未来复杂结构检查这类难以用简单 spec 表达的机制。

优先级：**中**。先把阶段顺序类型化，再决定哪些行为迁移。

### 7. `ConnectableBlock`

目的：统一信号连接、焊接连接和视觉连接器渲染规则。

建议接口：

```rust
pub trait ConnectableBlock: Block {
    fn connectors(&self, facing: Facing) -> &'static [ConnectorSpec];
}
```

建议数据：

```rust
pub struct ConnectorSpec {
    pub kind: ConnectorKind,
    pub sides: ConnectorSides,
    pub transmits_signal: bool,
    pub consumes_signal: bool,
    pub emits_signal: bool,
}
```

可替代的现有分散点：

| 当前位置 | 可迁移内容 |
| --- | --- |
| `SignalBehavior` | wire/device/detector 的拓扑规则。 |
| `WireConnectorBehavior` | 信号线视觉连接规则。 |
| `WeldConnectorBehavior` | 焊点视觉连接规则。 |
| `signals.rs` / `rendering.rs` | 可以读取同一个 connector spec。 |

优先级：**中低**。收益明确，但要同时碰信号和渲染，适合单独一轮做。

### 8. `PersistentBlockLayer`

状态：**已部分落地**。当前作为 `Block::persistent_layer()` 默认方法实现，尚未拆成独立 trait。

目的：把两阶段存档的保存归属从 `save.rs` 的过滤函数移动到方块自身。

建议接口：

```rust
pub trait PersistentBlockLayer: Block {
    fn persistent_layer(&self) -> Option<PersistentLayer>;
}
```

建议数据：

```rust
pub enum PersistentLayer {
    Puzzle,
    SolutionFactory,
}
```

现有规则：

| 方块类型 | 当前保存层 |
| --- | --- |
| SceneBlock | `Puzzle` |
| SystemBlock | `Puzzle` |
| FactoryBlock | `SolutionFactory` |
| MaterialBlock | 不保存 |
| GeneratedMarker | 不保存 |

优先级：**已执行第一步**。当前规则仍默认由 `BlockDefinition` 的 class 和 marker role 推导；如果以后出现特殊保存层方块，可以单独覆写 `persistent_layer()`。

## 建议实施顺序

| 优先级 | 抽象 | 原因 |
| --- | --- | --- |
| 1 | `MaterialBlockBehavior` | 改动小，立刻减少材料双向映射。 |
| 2 | `PersistentBlockLayer` | 把两阶段存档过滤规则移到方块元数据。 |
| 3 | `MovesMaterial` | 防止移动方块继续让 `movement.rs` 膨胀。 |
| 4 | `ConfigurableBlock` | 已统一存储和存档 schema，下一步是方块侧声明默认设置和 schema。 |
| 5 | `BlockSettingsPanel` | 能大幅压缩 UI 长文件，但实现量较大。 |
| 6 | `BlockCatalogEntry` | 统一注册和分类，适合作为一次结构性重构。 |
| 7 | `SimulationBehavior` | 先类型化阶段，再迁移复杂行为。 |
| 8 | `ConnectableBlock` | 统一信号和渲染连接规则，适合独立重构。 |

其中 1、2、3、4 已执行第一步，后续最值得继续的是 `BlockSettingsPanel`。

## 总体观察

当前系统相比上一版有两个明显变化：

- 模拟运行期已经按阶段拆分，`runtime.rs` 不再是新增模拟行为的默认修改点。
- 两阶段存档让 `save.rs` 从“保存整个世界”变成了“捕获 puzzle 层、捕获 solution 工厂层、加载时叠层”的专门模块。

剩余复杂度主要来自四类横切注册点：

- `BlockKind` 枚举和模块声明。
- `registry.rs` 中的多个数组。
- 国际化文件。
- 可配置方块专用的 map、UI 和两阶段存档配置序列化。

最值得优先做的抽象仍然是统一的方块注册和设置系统。它应该能集中生成或描述：

- `BlockKind` 注册数据。
- 放置分类。
- 国际化 key。
- 渲染颜色。
- trait / class。
- 可选模拟行为。
- 可选设置面板 spec。
- 可选持久化设置 schema。

这样不会牺牲 Rust 的类型安全，但可以让“新增一个方块”尽量局部化，避免在 registry、UI、simulation、save、localization 多个位置反复同步修改。
