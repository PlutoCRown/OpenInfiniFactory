# 结构运动分析报告

本文分析当前结构在模拟中能受到的运动类型，以及这些运动发生冲突时的优先级。相关代码主要在：

- `src/game/simulation/runtime.rs`
- `src/game/simulation/gravity.rs`
- `src/game/simulation/movement.rs`
- `src/game/simulation/structures.rs`
- `src/game/simulation/behaviors.rs`
- `src/game/world/blocks.rs`

## 结论概览

当前结构在主移动阶段可以受到 6 种运动影响：

| 运动 | 来源方块 / 系统 | 作用对象 | 运动形式 | 类型优先级 |
| --- | --- | --- | --- | --- |
| 重力下落 | 重力系统 | 材料结构、活动工厂结构 | 向下平移 | 最高类型，`Vertical` |
| 抬升 | Lifter | 材料结构、可推动工厂结构 | 向上平移 | 最高类型，`Vertical` |
| 顺向传送带 | Conveyor | 材料结构、可推动工厂结构 | 水平平移 | 最低，`Conveyor` |
| 反向传送带 | ReverseConveyor | 材料结构、可推动工厂结构 | 水平平移 | 最低，`Conveyor` |
| 旋转 | Rotator / CounterRotator | 材料结构 | 绕设备位置水平旋转 | 中高，`Rotate` |
| 动力平移 | Pusher / Blocker | 材料结构、可推动工厂结构 | 沿朝向平移或回拉 | 中，`Push` |

另外，行为阶段还有一种额外位移：

| 运动 | 来源方块 | 作用对象 | 说明 |
| --- | --- | --- | --- |
| 传送 | TeleportEntrance / TeleportExit | 材料结构 | 在行为阶段直接调用结构移动，不参与主移动阶段的优先级合并 |

因此，如果只问“主移动计划里结构能受到多少种运动”，答案是 6 种；如果把行为阶段的传送也算作结构位移，答案是 7 种。

## 回合中的运动阶段

`run_turn` 当前顺序和运动相关的部分如下：

1. 准备阶段：清理生成标记、放置到期生成材料、运行静态 marker 和焊接行为。
2. 信号阶段：刷新信号网络，得到 `powered_devices`。
3. 重力标记：`mark_gravity_phase` 先产生重力移动计划。
4. 设备移动标记：`mark_structure_movement_phase` 收集传送带、抬升器、旋转器、活塞、阻拦器等设备移动。
5. 移动合并：`merge_structure_movement_plan` 把重力移动和设备移动按优先级合并。
6. 移动执行：`execute_structure_moves_with_pushers` 统一执行平移和旋转。
7. 行为阶段：钻孔、标记、转换、传送等行为在主移动后执行，其中传送会额外移动材料结构。

也就是说，重力和设备运动会进入同一个主移动计划；传送门位移不在这个计划里，它发生得更晚。

## 运动类型明细

### 1. 重力下落

重力由 `mark_gravity_phase` 生成，分两类：

- 材料结构：从材料方块出发，沿 `material_welds` 找到焊接材料结构，然后尝试整体向 `IVec3::NEG_Y` 移动。
- 工厂结构：从 `FactoryStructureState` 查询可下落结构，只有 `FactoryActivity::Active` 且自由度允许时才会下落。

重力移动使用 `MovementMark::Vertical`，类型优先级最高。不过它没有设备 source，第一层 `influence_count` 会是 `u32::MAX`，所以它不是绝对不可覆盖的移动；同结构冲突时，有 source 的设备移动可能在第一层优先级上打败重力。

### 2. 抬升

Lifter 的 `MovementRule::Lift { range: 5 }` 会向上搜索 1 到 5 格内第一个可移动对象：

- 如果找到材料，移动整个焊接材料结构。
- 如果找到可推动工厂结构，移动整个工厂结构。

抬升方向是 `IVec3::Y`，也使用 `MovementMark::Vertical`。因此它和重力同属最高运动类型；同时它有设备 source，会参与影响次数比较。

### 3. 顺向传送带

Conveyor 的规则是：

```rust
MovementRule::Translate {
    source: IVec3::Y,
    offset: facing.forward_ivec3(),
}
```

它读取自身上方的结构，并沿自身朝向水平推动。标记是 `MovementMark::Conveyor`，主优先级最低。

如果上方没有可移动结构，但目标方向相邻位置有占用，代码还会尝试从传送带后方找结构并向反方向移动。这是 `mark_conveyor_movement` 中的 fallback，用来处理传送带被目标侧阻挡时的反向牵连场景。

### 4. 反向传送带

ReverseConveyor 的规则是：

```rust
MovementRule::Translate {
    source: IVec3::NEG_Y,
    offset: -facing.forward_ivec3(),
}
```

它读取自身下方的结构，并沿朝向反方向水平推动。它和普通传送带一样使用 `MovementMark::Conveyor`，所以优先级也最低。

### 5. 旋转

Rotator 和 CounterRotator 使用 `MovementRule::Rotate`：

- Rotator：`clockwise: true`
- CounterRotator：`clockwise: false`

当前旋转只作用于设备上方一格的材料结构，不作用于工厂结构。旋转会绕设备位置在水平面内改变结构坐标，并同步旋转材料方块朝向、焊接关系和面标记。

旋转没有 `MovementMark`，但在优先级比较中单独排在 `Vertical` 后、`Push` 前。

### 6. 动力平移

Pusher 和 Blocker 都使用 `MovementRule::PoweredTranslate`，方向是自身朝向：

- Pusher：通电时希望伸出，不通电时希望收回。
- Blocker：逻辑相反，不通电时希望伸出，通电时希望收回。

动力平移使用 `MovementMark::Push`。伸出时会推动前方结构；收回时只有在之前绑定了前方结构的情况下，才会尝试把前方结构拉回。Pusher / Blocker 还会维护 `PusherState`，用于记录伸出状态、前方绑定关系和硬头部占用。

## 推动链

平移运动在合并前和执行时都会通过 `expanded_move_structure` 扩展：

1. 先从原始结构开始。
2. 检查目标方向上的阻挡。
3. 空格可以通过。
4. 遇到材料结构时，把整个焊接材料结构并入移动集合。
5. 遇到可推动工厂结构时，把整个工厂结构并入移动集合。
6. 新并入的结构继续检查前方阻挡，因此可以形成推动链。
7. 遇到场景、不可推动工厂结构、越界或其他不可进入目标时，移动失败。

所以“一个结构受到某种运动”不只包括直接被设备作用，也包括被前方或后方的移动链间接带动。

## 主优先级规则

移动合并使用 `movement_priority_key`：

```rust
(
    influence_count,
    movement_kind_priority,
    conveyor_source_priority,
)
```

排序越小，优先级越高。

### 第一层：设备影响次数

有 source 的设备移动会被 `MovementInfluenceCache` 记录执行次数。同一个 source 对同一个结构连续成功影响的次数越少，优先级越高。

这意味着多个设备长期争夺同一个结构时，系统会优先考虑影响次数较少的设备，避免同一个 source 永远压制其他 source。

没有 source 的移动，例如重力，`influence_count` 是 `u32::MAX`。不过重力计划先进入 `planned_moves`，设备移动只有在自己能打败已有移动时才会替换它；同结构冲突时，设备因为第一层计数更小，可能替换重力。实际想判断“重力一定最高”时不能只看 `movement_kind_priority`，还要看它是否已经在计划中、以及 challenger 的 source 计数。

### 第二层：运动种类

当第一层相同时，运动种类优先级是：

| 排名 | 类型 | 代码值 |
| --- | --- | --- |
| 1 | `Vertical` 平移，包括重力和抬升 | `0` |
| 2 | 旋转 | `1` |
| 3 | `Push` 平移，包括 Pusher / Blocker | `2` |
| 4 | `Conveyor` 平移，包括 Conveyor / ReverseConveyor | `3` |

这个顺序只在第一层影响次数相同的时候决定胜负。

### 第三层：传送带 source 坐标

如果两个移动都是传送带，并且前两层也相同，会比较 `ConveyorSourcePriority`：

```rust
positive_x: -source.x,
negative_x: source.x,
positive_y: -source.y,
negative_y: source.y,
positive_z: -source.z,
negative_z: source.z,
```

也就是按 source 坐标生成一个稳定排序键，让多个传送带冲突时结果可重复，而不是依赖 HashMap 遍历顺序。

## 合并时的替换规则

`merge_structure_movement_plan` 的核心规则是：

1. 先扩展已有计划和设备计划，让推动链参与冲突判断。
2. 设备移动按优先级排序。
3. 如果新设备移动和已有移动影响的结构重叠，并且已有移动优先级更高或相同，新移动被丢弃。
4. 如果新设备移动优先级更高或相同，会移除被它打败的已有移动，再加入新移动。

这里“重叠”看的是移动结构集合是否有共同坐标，而不是只看最初被设备直接作用的方块。

## 执行时的二次冲突

即使合并阶段通过了，执行阶段仍会按顺序检查：

- 如果移动的源结构中已有方块被之前移动处理过，跳过。
- 平移会再次扩展推动链，并检查碰撞。
- 旋转会再次检查目标位置是否可放置。

因此最终结果由“合并优先级”和“执行时世界状态”共同决定。优先级决定谁先进入计划，执行检查决定它在当时世界状态下是否真的能完成。

## 传送门位移

TeleportEntrance / TeleportExit 在 `run_material_teleport_phase` 中处理：

- 只处理入口位置上的材料结构。
- 需要入口配置了成对出口，且出口位置确实是 TeleportExit。
- 计算 `exit - entrance` 作为整体 offset。
- 直接调用 `execute_structure_moves` 移动材料结构。

这类位移发生在行为阶段，不参与 `merge_structure_movement_plan`，也不和重力、传送带、抬升、旋转、活塞使用同一套优先级合并。

## 当前限制

1. 旋转器当前只旋转材料结构，不能旋转工厂结构。
2. 传送门只传送材料结构，不处理工厂结构。
3. 优先级的第一层是设备影响次数，不是纯粹的运动类型枚举；因此“Vertical 永远最高”不是完整描述。
4. 工厂结构是否可移动取决于 `FactoryStructureState` 中的 `activity`、`freedom` 和 `pushable`；材料结构则取决于焊接关系和碰撞。
5. 行为阶段传送是额外移动，不参与主移动计划的全局冲突解析。
