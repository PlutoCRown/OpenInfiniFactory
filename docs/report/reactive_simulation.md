# Reactive / Continue 模拟模式调研

本文对应 `todo.md` 的 S01：调研 Barrier 的 Reactive / Continue 模拟模式，并评估是否能把当前游戏改成回合 Reactive 模型。

## 调研边界

当前仓库内没有 Barrier 相关实现或文档。公开搜索也没有找到可核验的、与本项目语境匹配的 “Barrier Reactive / Continue simulation mode” 说明；命中的结果主要是无关的游戏道具、动物行为或其他领域的 reactive barrier 概念。

因此本文不把 Barrier 的具体规则当作已确认事实。下文的判断基于：

- 用户描述中提到的 Reactive / Continue 模拟方向。
- 当前项目的 `run_turn`、`tick_simulation` 和模拟阶段模型。
- 当前代码已经具备的离散回合、单步、连续运行和动画插值机制。

## 当前项目的模拟模型

当前模拟入口是 `src/game/simulation/runtime.rs`：

- `tick_simulation` 每帧运行，但只有在 Play 模式且满足条件时才推进模拟。
- 玩家请求单步时，立即推进一个回合。
- 模拟运行中，`accumulator` 达到 1.0 时推进一个或多个回合。
- `run_turn` 是完整回合，负责世界状态变更、移动计划合并、行为执行和渲染重建。

当前 `SimulationState` 包含：

- `running`：是否连续运行。
- `step_requested`：是否请求单步。
- `speed`：连续运行速度。
- `turn`：当前回合数。
- `accumulator`：帧时间累积到回合的进度。
- `start_snapshot` / `start_factory_structures`：模拟开始快照，用于回滚或作者态读取。

这说明当前项目已经是离散回合模拟，不是连续物理模拟。所谓 Continue 更接近当前 `running + accumulator`；所谓 Reactive 可以理解为“输入或世界事件触发回合推进 / 计划刷新，而不是只靠模式切换和固定播放状态”。

## Reactive 模式的可行定义

建议把 Reactive 定义为：

1. 世界仍然只在离散回合边界发生逻辑变化。
2. 输入、开关、生成器、玩家交互等事件可以请求下一次回合求值。
3. 每次求值都先生成下一回合计划，再在动画期展示该计划的执行结果。
4. 如果没有事件、没有运行状态、没有 pending 生成物、没有活动设备，则世界可以保持静止。

这个定义比“每帧实时模拟所有东西”更适合当前代码，因为它保留了 `run_turn` 的确定性和当前移动冲突规则。

## Continue 模式的可行定义

建议把 Continue 定义为：

1. 在 Reactive 模型上持续自动请求下一回合。
2. 仍然通过 `accumulator` 控制速度。
3. 动画和 UI 显示可以持续插值，但逻辑状态只在回合边界变化。

这与当前 `simulation.running` 基本一致。因此 Continue 不需要重写核心模型，只需要在命名和状态组织上更清晰。

## 是否适合把整个场景改成回合 Reactive

适合，但不应该一次性把整个项目改成新模式。

当前最稳妥的路线是保留 `run_turn` 作为唯一权威逻辑推进函数，把 Reactive 做成一层调度策略：

- Step：用户手动请求一次 `run_turn`。
- Continue：`accumulator` 自动请求 `run_turn`。
- Reactive：世界事件、玩家交互或设备状态变化请求 `run_turn`。
- Dynamic：后续动态模式可以组合 Reactive 和 Continue，允许玩家边放置、边触发、边实时推进。

这样能避免把模拟行为散落回每帧系统里。

## 需要调整的系统

### 1. SimulationState 状态命名

可以把当前布尔组合扩展为显式运行策略：

```rust
pub enum SimulationAdvanceMode {
    Paused,
    StepRequested,
    Continue,
    Reactive,
}
```

短期不必马上替换；可以先增加内部辅助函数：

- `should_advance_this_frame`
- `request_reactive_step`
- `advance_one_turn`

### 2. 下一回合计划缓存

P 模式 Moment 列表需要“下一回合计划比动画早一帧”。Reactive 化时建议拆出：

- `prepare_turn_plan(world, ...) -> TurnPlan`
- `execute_turn_plan(world, TurnPlan, ...)`

短期可以先只缓存调试所需摘要，不必把整个移动执行重构掉。

### 3. 输入事件触发

Reactive 模式需要定义哪些事件会请求回合：

- 开关状态变化。
- 玩家点击传送器。
- 生成器周期到达。
- 放置 / 删除 / 旋转会影响模拟的方块。
- 信号网络状态变化。

事件只应该请求回合，不应该直接修改模拟结果。

### 4. 动态模式隔离

如果后续实现动态模式，它应独立于现有 Puzzle / Solution 存档：

- 独立存档类型。
- 独立 BuilderMode / GameMode 或运行策略。
- 不复用会污染解法的模拟快照。

Reactive 模型可以作为动态模式底层，但动态模式不应该反过来污染普通 Play 模式。

## 风险

1. 如果把 Reactive 理解为“每帧实时执行逻辑”，会破坏当前回合制确定性。
2. 如果输入事件直接改世界，而不是请求 `run_turn`，会绕过移动冲突和行为阶段。
3. 如果下一回合计划缓存与实际执行逻辑分叉，会出现 UI 显示和真实移动不一致。
4. 如果 Dynamic 模式复用 Puzzle / Solution 存档，可能导致存档语义混乱。

## 建议迁移路径

1. 保留 `run_turn`，先提取“是否推进回合”的调度判断。
2. 为调试 UI 增加只读的下一回合运动摘要缓存。
3. 把 Continue 明确为自动推进策略，而不是散落在 `running` 和 `accumulator` 的隐含语义里。
4. 增加 Reactive step request 通道，让事件能请求下一回合。
5. 在动态模式分支中使用 Reactive + Continue 组合，而不是修改普通 Play 模式语义。
6. 等动态模式稳定后，再决定是否把普通 Play 也迁移到统一的 advance mode 枚举。

## 结论

当前项目适合采用“回合 Reactive”模型，但实现上应把它作为 `run_turn` 之上的调度层，而不是重写核心模拟。

Continue 可以直接映射到当前连续运行逻辑；Reactive 应理解为事件驱动的回合请求。这样既能支持后续动态模式，也能保留当前结构移动、信号、行为阶段的确定性。

