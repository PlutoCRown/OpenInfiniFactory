# 结构处理逻辑报告

本文说明当前游戏中“结构”的判定、缓存、移动和调试显示逻辑。相关代码主要在：

- `src/game/simulation/factory_activity.rs`
- `src/game/simulation/structures.rs`
- `src/game/simulation/movement.rs`
- `src/game/simulation/gravity.rs`
- `src/game/systems/gameplay.rs`

## 结构分类

当前结构分为两类：

| 结构类型 | 组成 | 连接规则 | 是否缓存 |
| --- | --- | --- | --- |
| 材料结构 | 只包含材料方块 | 通过 `material_welds` 焊接关系连接 | 不持久缓存，使用时即时计算 |
| 工厂结构 | 只包含工厂方块 | 六方向相邻连接 | 缓存在 `FactoryStructureState` |

这两类结构不会混合。游玩模式只能放工厂方块；模拟期焊接器目前只处理材料方块。因此一个结构内部要么全是工厂方块，要么全是材料方块。

## 工厂结构缓存

工厂结构由 `FactoryStructureState` 保存：

```rust
pub struct FactoryStructureState {
    structures: Vec<FactoryStructure>,
    structure_by_pos: HashMap<IVec3, usize>,
}
```

其中：

- `structures` 保存所有工厂结构。
- `structure_by_pos` 从方块坐标映射到结构下标，用于快速查询某个工厂方块属于哪个结构。

单个工厂结构保存：

```rust
pub struct FactoryStructure {
    pub kind: StructureKind,
    pub positions: HashSet<IVec3>,
    pub activity: FactoryActivity,
    pub freedom: StructureFreedom,
}
```

字段含义：

| 字段 | 含义 |
| --- | --- |
| `kind` | 结构类型。目前工厂缓存中的值为 `StructureKind::Factory`。材料结构在 hover 显示时也会使用 `StructureKind::Material`。 |
| `positions` | 结构内所有方块坐标。 |
| `activity` | 活动性。`Active` 表示可移动；`Inactive` 表示固定。 |
| `freedom` | 自由度。目前只有 `All` 和 `None` 两档。`All` 表示六方向都可尝试移动；`None` 表示完全不能移动。 |

## 工厂结构建立规则

工厂结构通过六方向相邻关系建立。构建流程在 `FactoryStructureState::from_world`：

1. 遍历所有工厂方块。
2. 从未处理的工厂方块开始 BFS。
3. 六方向相邻且也是工厂方块的格子加入同一个结构。
4. 记录结构列表和 `structure_by_pos` 查询表。

活动性规则：

1. 如果一个工厂结构与任何场景方块相邻，则该结构为 `Inactive`。
2. 如果一个工厂结构与任何 `Inactive` 工厂结构相邻，则它也会传播为 `Inactive`。
3. 未被传播为固定的结构为 `Active`。

自由度规则：

- `Active` 工厂结构当前设置为 `StructureFreedom::All`。
- `Inactive` 工厂结构当前设置为 `StructureFreedom::None`。

后续如果要支持“某些方向固定、某些方向可移动”，可以把 `StructureFreedom` 从两档扩展为六方向 bitset 或方向集合。

## 更新时机

工厂结构连接性在编辑 / 放置阶段更新，而不是在模拟中因为相邻自动更新。

当前会重建工厂结构缓存的操作包括：

- 放置方块。
- 删除方块。
- 旋转方块。
- 切换方块形态。
- 移动编辑选择区域。
- 加载世界。
- 清空世界。
- 重置解法。
- 切换编辑 / 游玩相关流程。
- 开始模拟时也会重建一次，确保模拟快照使用最新结构。

模拟期间，工厂结构不会因为落地或移动后相邻而合并成新结构。结构移动时只更新已有结构的 `positions` 和 `structure_by_pos` 坐标映射。

## 材料结构处理

材料结构由 `material_structure(world, start)` 即时计算。

它不看六方向普通相邻关系，只看 `world.material_welds`：

1. 从起点材料方块开始。
2. 沿焊接关系查找相邻材料。
3. 得到当前焊接材料结构。

材料结构没有存入 `FactoryStructureState`。原因是材料结构会被焊接器在模拟期改变，用即时计算能直接反映最新焊接状态。

## 重力处理

重力阶段在 `mark_gravity_phase` 中产生移动计划：

- 材料结构：按焊接关系找到结构，能下落则生成向下移动。
- 工厂结构：从 `FactoryStructureState` 查询结构，只有 `Active` 且对应方向有自由度时才尝试下落。

重力阶段只生成 `StructureMove`，不立即修改世界。

`StructureMove` 的移动目标是结构，而不是单个方块。平移移动保存的是一整个 `HashSet<IVec3>` 结构坐标集合。

## 设备移动处理

设备移动阶段在 `mark_structure_movement_phase` 中处理：

| 设备行为 | 当前处理 |
| --- | --- |
| 传送带 / 反向传送带 | 源格是材料则移动材料结构；源格是工厂则移动工厂结构。 |
| 活塞 | 仅在通电时移动源格结构，并记录活塞动画 actor。 |
| 抬升器 | 在范围内寻找第一个材料结构或可移动工厂结构并向上移动。 |
| 旋转器 | 当前仍只旋转材料结构。 |

如果设备自身就在被检测到的工厂结构内，则不会对这个结构产生移动效果。这样传送带属于某个悬空工厂结构时，不会自己推动自己所在的结构。

重力移动和设备移动会先合并为结构级移动计划。平移移动会保存一个移动标记：

| 标记 | 来源 | 优先级 |
| --- | --- | --- |
| `Vertical` | 重力、抬升器 | 最高 |
| `Push` | 活塞、阻拦器、其他结构推进；旋转移动也按该级别处理 | 中 |
| `Conveyor` | 传送带 / 反向传送带 | 最低 |

合并规则是结构级的：如果两个移动计划影响同一个结构，高优先级移动不会被低优先级移动覆盖；同优先级或更高优先级的后来移动会替换已有移动。因此抬升器的向上移动可以覆盖重力下落，活塞推动不会被传送带覆盖；当一个结构同时受到多个传送带影响时，后收集到的传送带移动生效。合并前会先按当前世界状态扩展平移计划，所以被递归推动到的前方结构也会参与优先级冲突判断。

## 推动规则

实际移动执行在 `execute_structure_moves_with_pushers`。

平移移动会先调用 `expanded_move_structure`：

1. 从待移动结构开始。
2. 检查目标方向上的每个目标格。
3. 空格可以通过。
4. 如果目标格是材料结构，则把该焊接材料结构并入移动集合。
5. 如果目标格是活动工厂结构，则把该工厂结构并入移动集合。
6. 新并入的结构也会继续沿同方向检查前方阻挡，因此推动链可以递归扩展。
7. 如果目标格是场景、非活动工厂或其他不能推动的实体，则移动失败。
8. 扩展完成后再次做基础碰撞检查。

因此，一个材料结构或工厂结构可以推动前方多个可同向移动的结构。被推动对象可以是材料结构，也可以是活动工厂结构。非活动工厂结构相当于固定障碍。

移动成功后：

- 世界中的方块被平移。
- 材料焊接和面标记随被移动结构迁移。
- 工厂结构缓存中的结构坐标通过 `FactoryStructureState::move_positions` 同步平移。
- 渲染动画按每个移动方块生成。

## Hover 线框

运行期瞄准显示由 `HoverStructureBounds` 保存当前要画的结构包围盒：

```rust
pub struct HoverStructureBounds {
    pub bounds: Option<StructureBounds>,
}

pub struct StructureBounds {
    pub kind: StructureKind,
    pub min: IVec3,
    pub max: IVec3,
}
```

显示规则：

- 瞄准材料方块：任意时候显示材料结构包围盒。
- 瞄准工厂方块：只有按 `P` 打开结构调试模式时显示工厂结构包围盒。
- 瞄准场景方块：不显示。
- 瞄准完全无自由度的工厂结构：不显示。

包围盒使用结构内所有方块的最大包围盒。绘制使用 `Gizmos::cube`，中心通过 `grid_to_world(min/max)` 计算，避免半格偏移；尺寸会轻微外扩并重复绘制几层，让线框更明显。

## P 调试模式

按 `P` 切换工厂结构调试显示：

- 工厂方块渲染为实心方块。
- 活动结构显示为绿色。
- 非活动结构显示为红色。
- 工厂结构 hover 线框只在该模式打开时显示。

材料结构 hover 线框不依赖 P 模式。

## 当前限制

1. `StructureFreedom` 目前只有 `All` 和 `None`，还没有保存六个方向的独立自由度。
2. 工厂结构缓存只保存工厂结构；材料结构是即时计算，不在同一个结构列表中持久保存。
3. 旋转器目前仍只处理材料结构。
4. 移动冲突规则仍是顺序执行：一个移动占用源或目标后，后续冲突移动会被跳过。例外是设备向上移动会在执行前覆盖同结构的重力向下移动。
5. 模拟期不会自动合并新相邻的工厂结构；这是当前设计目标，但如果未来有能连接工厂结构的设备，需要显式更新 `FactoryStructureState`。

## 总结

当前结构系统的核心原则是：

- 工厂结构在编辑 / 放置阶段确定，模拟期保持结构身份稳定。
- 材料结构由焊接关系决定，使用时即时计算。
- 工厂结构有活动性和自由度；材料结构主要由焊接关系决定整体移动。
- 传送带、活塞、抬升器可以移动材料结构，也可以移动可动工厂结构。
- Hover 线框和 P 调试模式共享结构信息，但显示规则不同。
