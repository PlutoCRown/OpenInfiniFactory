# 两阶段存档实现报告

## 存档类型

当前存档分为两种：

| 类型 | 语义 | 保存内容 |
| --- | --- | --- |
| `Puzzle` | 编辑阶段存档 | 场景方块，以及编辑模式可放置的系统方块。不会保存工厂方块、材料方块、焊接关系或材料面标记。 |
| `Solution` | 游玩阶段存档 | 内置一份 `Puzzle` 快照，再额外保存当前解法里的所有工厂方块。不会保存材料方块、焊接关系或材料面标记。 |

旧版 RON 存档会按 `Puzzle` 加载。加载时只保留场景方块和系统层方块，旧存档里的材料和工厂方块不会进入 puzzle 层。

## 读写流程

| 操作 | 当前行为 |
| --- | --- |
| 新建世界 | 创建 `Puzzle`。 |
| 从主菜单加载存档 | 按编辑模式打开；如果文件是 `Puzzle`，继续作为 `Puzzle` 编辑。 |
| 从暂停菜单加载 `Puzzle` | 按游玩模式打开，并把它转成当前 `Solution` 上下文，`Solution` 内置该 puzzle 的快照。 |
| 从暂停菜单加载 `Solution` | 先加载内置 puzzle 快照，再叠加工厂方块层。 |
| 编辑模式保存 | 保存为 `Puzzle`。 |
| 游玩模式保存 | 保存为 `Solution`。如果当前 solution 来自 puzzle，会把 puzzle 快照写入 solution。 |
| Solution reset | 清空当前工厂方块层，恢复到 solution 内置的 puzzle 快照。 |
| Solution 切回编辑模式 | 先进入确认界面，可以保存当前 solution 后编辑，也可以不保存直接编辑，或者取消。进入编辑后恢复到 puzzle 快照，并清空当前存档名，避免后续保存 puzzle 覆盖已有 solution。 |

## 关键代码位置

| 文件 | 责任 |
| --- | --- |
| `src/shared/save.rs` | 定义 `SaveKind`、`SaveFileKind`、`WorldLayer`、`LoadedSave`，实现 puzzle/solution 的序列化和加载叠层。 |
| `src/game/state.rs` | `SolutionState` 保存当前 solution 内置的 puzzle 快照；`SimulationState` 保存模拟开始快照。 |
| `src/game/systems/menus.rs` | 接入新建、读取、保存、切换编辑确认、solution reset。 |
| `src/game/systems/simulation_controls.rs` | 模拟开始时记录完整世界快照，回滚时恢复快照。 |
| `src/game/ui/layout.rs` / `src/game/ui/systems.rs` | 暂停菜单增加 solution reset 和切回编辑前的保存确认按钮，并按模式控制可见性。 |

## 模拟回滚修复

之前 `reset_simulation` 只删除材料方块和生成标记，所以模拟期间下落的悬空工厂方块不会回到开始位置。

现在逻辑改为：

1. 第一次开始模拟时，把完整 `WorldBlocks` 克隆到 `SimulationState::start_snapshot`。
2. 回滚时，如果存在快照，则直接恢复该快照。
3. 如果没有快照，则兜底删除材料和生成标记。

这样工厂方块在模拟期间被重力、传送带或其它机制移动后，停止/回滚会恢复到模拟开始前的位置。

## 剩余注意点

- 暂停菜单确认界面目前复用同一个 `PausePanel`，通过按钮可见性切换普通暂停和“保存 solution 后编辑”的确认状态。
- 旧存档兼容路径会保留旧系统方块设置，但只对仍存在于 puzzle 层的系统方块生效。
- 新格式仍写在原来的 `saves/*.ron` 路径下，列表 UI 暂时不区分展示 puzzle / solution 类型。
