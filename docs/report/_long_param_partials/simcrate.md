# Agent-simcrate

## 清单 4 处方（Agent-SimCrate / 仅 `crates/oif-sim`）

注册表约束：mark 命中字段必须 **C8 `SimMovementMarkCtx`**（plain ctx，非 Bevy）。未使用 C2/C3 名；不建议消息/SystemParam。

C8 权威字段（与 `mark_pusher_movement` ∩ `try_deform_action` 一致）：
`world`, `structures`, `suction`, `claimed_heads`, `motion_held`, `motion_tags`, `succeeded_deform`, `actuating_extend`, `actuating_retract`

---

### 1. `try_deform_action` — `simulation/movement/mark.rs`

| 字段 | 值 |
|---|---|
| **params** | **16**：`world`, `structures`, `suction`, `claimed_heads`, `pos`, `id`, `structure_id`, `forward`, `move_offset`, `animation`, `claim_head`, `motion_held`, `motion_tags`, `succeeded_deform`, `actuating_extend`, `actuating_retract` |
| **kind** | helper |
| **手段** | **1** 聚合 C8 |
| **共享簇ID[]** | `[C8]` |
| **local-only说明** | — |
| **落地形态** | `fn try_deform_action(ctx: &mut SimMovementMarkCtx, pos, id, structure_id, forward, move_offset, animation, claim_head) -> Option<StructureMove>` → **约 8 参** |
| **风险** | 低；仅改签名透传。`ctx.world` 保持 `&WorldBlocks`（与现一致） |
| **优先级** | **P0**（Wave E / C8 核） |

---

### 2. `mark_pusher_movement` — `simulation/movement/mark.rs`

| 字段 | 值 |
|---|---|
| **params** | **14**：`world`, `structures`, `pusher_state`, `pos`, `source`, `offset`, `desired_extended`, `claimed_heads`, `suction`, `motion_held`, `motion_tags`, `succeeded_deform`, `actuating_extend`, `actuating_retract` |
| **kind** | helper |
| **手段** | **1** 聚合 C8 |
| **共享簇ID[]** | `[C8]` |
| **local-only说明** | — |
| **落地形态** | `fn mark_pusher_movement(ctx: &mut SimMovementMarkCtx, pusher_state: &mut PusherState, pos, source, offset, desired_extended) -> Option<StructureMove>` → **约 6 参**；内部 4 处 `try_deform_action` 改传 `ctx` |
| **构造点** | 仅 `mark_structure_movement_phase`：回合局部 `claimed_heads` / `motion_*` / `succeeded_deform` / `actuating_*` + `world`/`structures`/`suction` 装入 C8 后下传（该入口本身 5 参，不进清单） |
| **风险** | 低–中；`pusher_state` **故意不进 C8**（仅本函数需要） |
| **优先级** | **P0** |

---

### 3. `trace_laser` — `simulation/behaviors.rs`

| 字段 | 值 |
|---|---|
| **params** | **10**：`world`, `origin`, `direction`, `range`, `destroy`, `beams`, `sparks`, `debris`, `hit_detectors`, `bounce_depth` |
| **kind** | helper（递归；由 `run_lasers` 驱动） |
| **手段** | **1 local-only**（**非 C8**；与 movement mark 无关） |
| **共享簇ID[]** | `[]` |
| **local-only说明** | 将同步写完的输出组收成**模块私有**累加器（勿上注册表名）：`beams` + `sparks` + `debris` + `hit_detectors`。签名约：`(world, origin, direction, range, destroy, out, bounce_depth)` → **7 参** |
| **风险** | 低；递归自调用同改即可 |
| **优先级** | **P2** |
| **禁** | 不得映射 C8；不得建议消息总线 |

---

### 4. `simulate_turn` — `simulation/core.rs`

| 字段 | 值 |
|---|---|
| **params** | **9**：`world`, `pending_generated`, `signal_cache`, `turn`, `structure_state`, `movement_influence`, `pusher_state`, `sim_log`, `stats` |
| **kind** | helper（crate 公开编排入口；`SimSession::simulate_next_turn` + tests） |
| **手段** | **3 accept** |
| **共享簇ID[]** | `[]` |
| **local-only说明** | 字段已由 `SimSession` 持有；再造「回合 Ctx」= 单函数专用类型，违反「不为单函数新建共享类型」。**禁止**把 Bevy 侧 `SessionStateParams`/`PlayingWorldParams`/`C3` 拉进 oif-sim |
| **风险** | 无（保持 API） |
| **优先级** | **P3** |

---

## 统计

| 项 | 数量 |
|---|---:|
| 本清单函数 | **4** |
| 手段1 → **C8** | **2**（`try_deform_action`, `mark_pusher_movement`） |
| 手段1 → **local-only** | **1**（`trace_laser`） |
| 手段3 → **accept** | **1**（`simulate_turn`） |
| 手段2 消息 | **0** |
| 命中注册表 | C8×2；C1–C7 **0** |
| 预期参降 | 16→~8；14→~6；10→~7；9 不变 |

---

## 缺口

| 候选 | 字段列表 | 是否扩注册表 |
|---|---|---|
| （无）C8 字段 | 与两 mark helper 交集已锁死；`pusher_state` / deform 标量留调用参 | **不扩** |
| 激光输出组 | `beams`, `sparks`, `debris`, `hit_detectors` | **不扩**（仅 `trace_laser` 调用树，&lt;3 独立函数）→ local-only |
| 回合会话组 | `world`, `pending_generated`, `signal_cache`, `structure_state`, `movement_influence`, `pusher_state`（± log/stats） | **不扩**；`accept` + 已有 `SimSession` 持有 |

**同义词/越界检查：** 未建议 C2/C3 名、未建议 Bevy SystemParam/消息、未自创平行共享簇名。