# 测试用例报告

本文档说明当前项目里**所有自动化测试**各自在防什么问题。测试分两层：

| 层级 | 运行方式 | 目录 / 入口 |
| --- | --- | --- |
| **E2E（HTTP）** | `cd e2e && bun test` | `e2e/fixtures/` + `e2e/src/blocks.test.ts` |
| **Rust 单元测试** | `cargo test` | 各模块 `#[cfg(test)]` |

E2E 通过无头 `oif-debug-http` 驱动 `SimCoreWorld`，不启动窗口。详见 [architecture.md](./architecture.md) 与 `.cursor/skills/sim-debug-http/`。

---

## E2E 基础设施（`blocks.test.ts`）

| 用例 | 解决的问题 |
| --- | --- |
| `lists all registered block kinds` | 方块注册表与 HTTP `/blockKinds` 未断裂；至少 32 种 `BlockKind` 可查询 |
| `runs every block placement fixture` | 全部方块放置 smoke：占格规则、层叠规则、fixture 加载链路 |
| `runs simulation fixtures` | 下面 4 个 `fixtures/sim/` 行为用例整体可跑通 |
| `runN advances turn counter` | HTTP `/runN` 与模拟回合计数一致（推进 3 回合 → `turn === 3`） |

**说明**：`sim/test_pusher_platform_stuck.json` 目前**未**列入 `blocks.test.ts` 的 sim 列表，需手动 `runFixture` 或后续加入 CI。

---

## E2E：方块放置用例（`e2e/fixtures/blocks/*.json`）

共 **33 个**，由 `e2e/scripts/generate-fixtures.ts` 生成（`bun run generate-fixtures`）。每个文件对应一种 `BlockKind`。

###  collectively 解决什么问题

1. **注册表完整**：每种方块都能在调试核心里被 `place_block` 放置，不会拼写/层错误。
2. **占格与层规则**：场景 / 工厂 / 系统 / 材料 / 虚拟各层的最小合法摆法被固定下来，改 `WorldBlocks::can_place_*` 时会批量失败。
3. **虚拟 marker 生成**：父方块放置后，模拟 1 回合能生成预期虚拟格。

### 按类型的具体意图

| 类型 | 方块 | setup 要点 | 断言 |
| --- | --- | --- | --- |
| 场景 | Grass, Stone, Dirt, Planks | 仅自身一格 | 该格存在且 `layer: scene` |
| 工厂 | Platform, Welder, Conveyor, … | 下方垫 Stone | 工厂格存在且 `layer: factory` |
| 系统 | Generator, Goal, Stamper, … | 单独放置 | `layer: system` |
| 材料 | Material, IronMaterial, CopperMaterial | Stone + Platform + 材料 | `layer: material` |
| 虚拟 | WeldPoint, BlockerHead, DrillHead | Stone + 父方块（Welder/Blocker/Drill） | 跑 1 回合后前方出现对应虚拟格 |

虚拟父方块映射：`WeldPoint←Welder`，`BlockerHead←Blocker`，`DrillHead←Drill`。

---

## E2E：模拟行为用例（`e2e/fixtures/sim/`）

需要跑回合、断言结构或信号结果的用例。

### `welder_weld_point.json`

| 项 | 内容 |
| --- | --- |
| **场景** | Stone 上放 Welder（朝 North） |
| **断言** | 1 回合后 `(0,1,-1)` 生成 `WeldPoint`（virtual） |
| **解决的问题** | 焊接器静态 marker 阶段是否在模拟开始后正确生成焊点 |

### `wire_detector_power.json`

| 项 | 内容 |
| --- | --- |
| **场景** | Generator → Wire → Detector 串联 |
| **断言** | 1 回合后 Detector 仍在位（工厂层）；配合信号阶段应被通电 |
| **解决的问题** | 电线网络 + 生成器 → 探测器供电链路的基本 smoke（与 InfiniFactory 差异：探测器只认平台/材料，见 Rust 测试） |

### `opposing_pushers_shared_head.json`

| 项 | 内容 |
| --- | --- |
| **场景** | 左右两个 Pusher 相向，中间 Platform；Generator+Wire 给两侧供电 |
| **断言** | 1 回合后中间格变为 `BlockerHead`（factory） |
| **解决的问题** | **对向活塞共占杆头格**时，不能两杆重叠；应合并为阻拦头占用，避免占格冲突 |

### `conveyor_blocked_by_pusher_head.json`

| 项 | 内容 |
| --- | --- |
| **场景** | 传送带向东推材料；前方格上有已伸出的 Pusher（West）杆头占格 |
| **断言** | 1 回合后材料仍在 `(0,1,0)`，未被推入杆头格 |
| **解决的问题** | **传送带运动**必须 respect **活塞/阻拦器杆头硬占格**；材料不能穿进 head cell |

### `test_pusher_platform_stuck.json`

| 项 | 内容 |
| --- | --- |
| **来源** | 从存档 `solution_3`（puzzle `test_pusher`）裁剪区域 `(-14,0,-13)..(-10,3,-11)` |
| **场景** | 锚在石头上的工厂结构 + 探测器/电线 + 活塞；活塞前方有独立 Platform |
| **断言** | 2 回合后前方 Platform 被推到 `(4,2,1)`（本地坐标） |
| **解决的问题** | 回归：**场景锚定的大结构**与**活塞前方可推子结构**连通时，活塞应能推动前沿子集，而不是因整结构 `pushable=false` 只伸杆不推块（`pusher_target_structure` 子集 pushable 修复） |

---

## Rust 单元测试

### `src/shared/save.rs`

| 测试 | 解决的问题 |
| --- | --- |
| `puzzle_layer_round_trips_edit_system_blocks_and_settings` | Puzzle 存档往返：系统方块 + Generator/Goal/Stamper 等 **block_settings** 不丢 |
| `hotbar_round_trips_for_puzzle_and_solution` | Puzzle / Solution 的 hotbar 序列化一致 |
| `solution_loads_factory_blocks_from_puzzle_reference` | Solution 只存 `puzzle_id` + 工厂层时，加载能合并对应 Puzzle 场景 |

### `src/game/world/grid.rs`

| 测试 | 解决的问题 |
| --- | --- |
| `factory_cannot_overlap_system_block` | 工厂方块不能与系统方块同格（材料可同格） |
| `system_block_cannot_overlap_factory` | 反向：已有工厂时不能放系统方块 |
| `factory_cannot_overlap_generated_marker` | 工厂线（如 Wire）不能占虚拟 marker 格（DrillHead 等） |

### `src/game/simulation/signals.rs`

| 测试 | 解决的问题 |
| --- | --- |
| `detector_detects_material_and_platform_only_among_factory_blocks` | 方块探测器**只**对 Platform / 材料通电；对 Conveyor 等其它工厂块不应形成 powered network（与原版 InfiniFactory 行为差异） |

### `src/game/simulation/structure_state.rs`

| 测试 | 解决的问题 |
| --- | --- |
| `rebuild_for_simulation_groups_factory_and_material_separately` | 工厂连通块与焊接材料结构**分图**建组 |
| `rebuild_for_simulation_groups_connected_acceptors` | Goal 等验收器：相邻连通 vs 悬空分别成组 |
| `gravity_support_cache_survives_lookup_after_recorded` | 记录重力支撑后 `gravity_support_valid` 可正确查询 |
| `pusher_target_structure_allows_front_subset_when_whole_structure_is_scene_anchored` | 整结构 scene-anchored 不可推时，活塞**前方子集**仍可作为推动目标 |
| `pusher_target_structure_rejects_scene_anchored_subset` | 活塞前方子集若自身仍贴场景（锚定），则不可推 |

### `src/game/simulation/movement_plan.rs`

| 测试 | 解决的问题 |
| --- | --- |
| `opposing_pushers_only_one_occupies_shared_head_cell` | 与 E2E `opposing_pushers_shared_head` 同问题：对向活塞杆头只占中间一格（单元级运动计划） |
| `conveyor_stops_when_forward_cell_is_pusher_head` | 与 E2E `conveyor_blocked_by_pusher_head` 同问题：杆头阻挡传送带（单元级） |

### `src/game/simulation/structures.rs`（`movement_history_tests`）

| 测试 | 解决的问题 |
| --- | --- |
| `rotator_history_skips_repeat_mark_until_contact_ends` | 旋转器成功转动物体后，同配置重复标记应被忽略，避免一回合内重复转 |
| `conveyor_history_deprioritizes_stale_source` | 传送带「陈旧 source」候选应被降权，避免错误优先级导致反复假推 |

### `src/game/simulation/behaviors.rs`

材料行为与验收 / 传送，**无 pending 跨回合**的即时语义。

| 测试 | 解决的问题 |
| --- | --- |
| `teleport_moves_material_immediately` | 入口有材料、出口空 → 当回合传送到出口 |
| `teleport_moves_only_entrance_block_from_welded_structure` | 焊接结构中**只有入口格**传送，邻居不跟 |
| `teleport_waits_when_exit_is_occupied` | 出口占用则本回合不动 |
| `teleport_can_run_three_times_when_exit_clears_between` | 出口逐次清空可连续传送 |
| `teleport_retries_after_exit_clears` | 出口先占后空，下一回合重试成功 |
| `anchored_entrance_material_is_not_pushed_with_welded_neighbor` | 入口格贴验收器锚定时，与焊接邻居不可被一起平推 |
| `teleport_detaches_before_moving_to_exit` | 传送前断开与邻居的 weld |
| `teleport_does_not_move_unwelded_neighbor_on_entrance` | 未焊接邻居不随入口移动 |
| `acceptance_removes_matching_material_immediately` | Goal 匹配材料 → 当回合销毁 |
| `acceptance_ignores_wrong_material` | 材料种类不匹配 → 不验收 |
| `acceptance_requires_entire_connected_acceptor_structure` | 部分 Goal 连通组未满足时不验收 |
| `acceptance_requires_material_structure_without_extra_blocks` | 材料结构不能含多余未焊接块才验收 |
| `acceptance_removes_entire_welded_structure_immediately` | 匹配时整组焊接材料一起移除 |

### `src/game/systems/perf.rs`

| 测试 | 解决的问题 |
| --- | --- |
| `scope_order_matches_enum_variants` | 性能统计 `PerfScope` 枚举顺序与数组索引一致，防 refactor 错位 |

---

## 覆盖关系小结

```text
占格 / 存档          → grid tests + save tests + blocks/*
信号 / 探测器        → signals test + wire_detector_power.json
结构分组 / 重力      → structure_state tests
活塞推工厂子集       → pusher_target tests + test_pusher_platform_stuck.json
活塞杆头 / 对向      → movement_plan tests + opposing_* + conveyor_blocked_*
传送带历史           → structures movement_history tests
材料传送 / 验收      → behaviors tests（纯 Rust，无 E2E fixture）
旋转器 / 传送带优先级 → movement_history tests
HTTP 调试链路        → blocks.test.ts 基础设施
```

---

## 如何新增用例

| 需求 | 做法 |
| --- | --- |
| 新方块能放置 | `bun run generate-fixtures` 或改 `generate-fixtures.ts` |
| 新模拟行为 | 添加 `e2e/fixtures/sim/<case>.json`，并加入 `blocks.test.ts` sim 列表 |
| 从存档裁剪 | `cargo run --bin export_fixture -- ...`（见 sim-debug-http skill） |
| 纯逻辑、无 HTTP | 在对应模块 `#[cfg(test)]` 添加 Rust 测试 |

---

## 相关文档

- [simulation_turn_phases.md](./simulation_turn_phases.md) — 回合阶段顺序
- [structure_logic.md](./structure_logic.md) — 工厂结构 / 推动 / 锚定
- [blocks.md](./blocks.md) — 方块 trait 与分类
- [two_phase_save.md](./two_phase_save.md) — Puzzle / Solution 存档格式
