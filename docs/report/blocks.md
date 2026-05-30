# 方块报告

当前方块总数：**33**

本文档记录每个方块当前实现的 Rust trait，以及它在 `Block` trait 上覆写的行为方法。

## Trait 说明

当前方块系统里有一个基础 trait 和五个分类 marker trait：

| Trait | 用途 |
| --- | --- |
| `Block` | 所有方块都必须实现。提供 ID、定义、旋转、模拟行为、渲染行为、替换方块、默认设置等扩展点。 |
| `SceneBlock` | 场景/地形方块。保存到 puzzle 层，不参与材料模拟重力。 |
| `MaterialBlock` | 材料方块。参与材料结构、重力、焊接、标签、转换、传送等模拟逻辑。 |
| `FactoryBlock` | 工厂方块。保存到 solution factory 层，通常由玩家在游玩模式放置，参与工厂结构重力。 |
| `SystemBlock` | 系统层方块。存储在 `WorldBlocks::system_blocks`，可以和普通 `blocks` 层同格叠放。 |
| `EditableBlock` | 可在编辑模式直接放置的系统层方块。 |

`Block` trait 上还有一组行为方法。它们不是独立 trait，但实际承担了“能力 trait”的职责：

| 行为方法 | 用途 |
| --- | --- |
| `is_directional()` | 是否可以四向旋转。当前旋转能力直接由它表达。 |
| `marker_behavior()` | 生成焊点、阻拦头、钻头头等派生 marker。 |
| `material_source()` | 生成材料。当前只有生成器使用。 |
| `material_kind()` | 标识材料方块对应的 `MaterialKind`。 |
| `default_settings()` | 给可配置系统方块创建默认设置。 |
| `movement_rule()` | 在模拟期移动或旋转材料结构。 |
| `material_destroyer()` | 删除材料。 |
| `material_labeler()` | 给材料面打标。 |
| `weld_behavior()` | 作为焊接节点参与材料焊接。 |
| `signal_behavior()` | 作为电线、探测器或耗电设备参与信号网络。 |
| `render_behavior()` | 提供特殊渲染信息，例如连接线、目标顶盖。 |
| `alternate()` | Alternate 键切换到替换方块。 |

## SceneBlock

模拟期重力效果：❌

| 方块 | 中文名 | 可四向旋转 | 实现 Trait | 覆写行为方法 |
| --- | --- | --- | --- | --- |
| Grass | 草方块 | ❌ | `Block`, `SceneBlock` | 无 |
| Stone | 石头 | ❌ | `Block`, `SceneBlock` | 无 |
| Dirt | 泥土 | ❌ | `Block`, `SceneBlock` | 无 |
| Planks | 木板 | ❌ | `Block`, `SceneBlock` | 无 |

## MaterialBlock

模拟期重力效果：✅

| 方块 | 中文名 | 可四向旋转 | 可被生成器选择 | 实现 Trait | 覆写行为方法 |
| --- | --- | --- | --- | --- | --- |
| Material | 材料 | ❌ | ✅ | `Block`, `MaterialBlock` | `material_kind()` |
| Iron Material | 铁材料 | ❌ | ✅ | `Block`, `MaterialBlock` | `material_kind()` |
| Copper Material | 铜材料 | ❌ | ✅ | `Block`, `MaterialBlock` | `material_kind()` |

## FactoryBlock

模拟期重力效果：✅

| 方块 | 中文名 | 可四向旋转 | 替换方块 | 实现 Trait | 覆写行为方法 |
| --- | --- | --- | --- | --- | --- |
| Solid | 工厂方块 | ❌ |  | `Block`, `FactoryBlock` | 无 |
| Welder | 焊接器 | ✅ | Down Welder | `Block`, `FactoryBlock` | `is_directional()`, `marker_behavior()`, `render_behavior()`, `alternate()` |
| Down Welder | 向下焊接器 | ❌ | Welder | `Block`, `FactoryBlock` | `marker_behavior()`, `render_behavior()`, `alternate()` |
| Conveyor | 传送带 | ✅ | Reverse Conveyor | `Block`, `FactoryBlock` | `is_directional()`, `movement_rule()`, `alternate()` |
| Reverse Conveyor | 反向传送带 | ✅ | Conveyor | `Block`, `FactoryBlock` | `is_directional()`, `movement_rule()`, `alternate()` |
| Detector | 方块识别器 | ✅ | Down Detector | `Block`, `FactoryBlock` | `is_directional()`, `signal_behavior()`, `render_behavior()`, `alternate()` |
| Down Detector | 向下方块识别器 | ❌ | Detector | `Block`, `FactoryBlock` | `signal_behavior()`, `render_behavior()`, `alternate()` |
| Wire | 电线 | ❌ |  | `Block`, `FactoryBlock` | `signal_behavior()`, `render_behavior()` |
| Pusher | 活塞 | ✅ | Blocker | `Block`, `FactoryBlock` | `is_directional()`, `movement_rule()`, `signal_behavior()`, `render_behavior()`, `alternate()` |
| Lifter | 抬升器 | ✅ |  | `Block`, `FactoryBlock` | `is_directional()`, `movement_rule()` |
| Rotator | 旋转器 | ✅ | Counter Rotator | `Block`, `FactoryBlock` | `is_directional()`, `movement_rule()`, `alternate()` |
| Counter Rotator | 逆向旋转器 | ✅ | Rotator | `Block`, `FactoryBlock` | `is_directional()`, `movement_rule()`, `alternate()` |
| Blocker | 阻拦器 | ✅ | Pusher | `Block`, `FactoryBlock` | `is_directional()`, `marker_behavior()`, `signal_behavior()`, `render_behavior()`, `alternate()` |
| Drill | 钻头 | ✅ | Laser | `Block`, `FactoryBlock` | `is_directional()`, `marker_behavior()`, `material_destroyer()`, `signal_behavior()`, `render_behavior()`, `alternate()` |
| Laser | 激光 | ✅ | Drill | `Block`, `FactoryBlock` | `is_directional()`, `material_destroyer()`, `signal_behavior()`, `render_behavior()`, `alternate()` |

## SystemBlock

模拟期重力效果：❌

系统层方块保存在 `WorldBlocks::system_blocks` 中，可以和普通 `blocks` 层的材料/工厂/场景方块同格存在。生成器、转换器、传送方块、焊点等都依赖这个双层结构。

| 方块 | 中文名 | 可四向旋转 | 编辑模式可放置 | 实现 Trait | 覆写行为方法 |
| --- | --- | --- | --- | --- | --- |
| Generator | 生成器 | ❌ | ✅ | `Block`, `SystemBlock`, `EditableBlock` | `material_source()`, `default_settings()` |
| Goal | 目标块 | ❌ | ✅ | `Block`, `SystemBlock`, `EditableBlock` | `render_behavior()` |
| Stamper | 印花器 | ✅ | ✅ | `Block`, `SystemBlock`, `EditableBlock` | `is_directional()`, `material_labeler()`, `default_settings()` |
| Roller | 滚刷器 | ✅ | ✅ | `Block`, `SystemBlock`, `EditableBlock` | `is_directional()`, `material_labeler()`, `default_settings()` |
| Converter | 转换器 | ❌ | ✅ | `Block`, `SystemBlock`, `EditableBlock` | `default_settings()` |
| Teleport Entrance | 传送入口块 | ❌ | ✅ | `Block`, `SystemBlock`, `EditableBlock` | `default_settings()` |
| Teleport Exit | 传送出口块 | ❌ | ✅ | `Block`, `SystemBlock`, `EditableBlock` | `default_settings()` |
| Weld Point | 焊接点 | ❌ | ❌ | `Block`, `SystemBlock` | `render_behavior()`, `weld_behavior()` |
| Blocker Head | 阻拦头 | ❌ | ❌ | `Block`, `SystemBlock` | 无 |
| Drill Head | 钻头头 | ❌ | ❌ | `Block`, `SystemBlock` | `material_destroyer()` |

## 结构观察

现在的分类 trait 只负责声明“方块属于哪一类”，真正的玩法能力主要挂在 `Block` trait 的可选方法上。这个方向和最近的 Alternate / Rotatable 合并思路是一致的：不要再为每个能力额外拆 marker trait，而是让方块通过统一接口暴露能力。

如果后续方块数量继续增加，可以进一步把这些行为方法拆成声明式 spec，例如：

- `MovementSpec`：描述移动来源、目标偏移、是否需要供电、旋转方向。
- `ConnectorSpec`：描述电线/焊点连接面、阻挡面、信号角色。
- `SettingsPanelSpec`：描述系统方块 UI。
- `AlternateSpec`：描述可切换方块对。

这样文档里的“覆写行为方法”会逐步变成“方块能力配置”，新增方块时也能少改中心化代码。
