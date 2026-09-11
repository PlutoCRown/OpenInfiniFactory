use std::collections::{HashMap, HashSet};
use std::time::Instant;

use glam::IVec3;

use crate::world::grid::WorldBlocks;

use super::behaviors::{
    BreakDebris, LaserBeam, apply_pending_paints, apply_pending_stamps, probe_lasers,
    run_material_acceptance_phase, run_material_conversion_phase, run_material_destroy_phase,
    run_material_label_phase, run_material_teleport_phase, run_weld_behavior_phase,
};
use super::gravity::mark_gravity_phase;
use super::markers::run_static_marker_phase;
use super::motion::{BlockMotion, PusherMotion};
use super::movement::{PusherState, mark_structure_movement_phase};
use super::pending::{PendingDestroyReason, PendingTurnEffects};
use super::signals::SignalNetworkCache;
use super::stats::SimulationStepStats;
use super::structure_state::StructureState;
use super::structures::{
    MovementHistory, StructureMove, apply_fragile_shatter_before_execute, arbitrate_movement_plan,
    execute_structure_moves_with_pushers, merge_structure_movement_plan,
};
use super::suction::SuctionLinks;

/// 回合表现事件相对运动动画所处的阶段。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationPhase {
    TurnStart,
    AfterMotion,
}

/// 单回合模拟输出：运动 DTO、通电表现与行为火花
#[derive(Clone)]
pub struct TurnOutput {
    pub turn: u64,
    pub animations: HashMap<IVec3, BlockMotion>,
    pub pusher_animations: HashMap<IVec3, PusherMotion>,
    pub powered_wires: HashSet<IVec3>,
    /// 本回合通电的用电器格（抬升器顶盘熄灭等）
    pub powered_devices: HashSet<IVec3>,
    /// 成功焊接的焊点对（本回合运动结束后，在两端格心连线中点播扩散粒子）
    pub weld_sparks: Vec<(IVec3, IVec3)>,
    /// 本回合成功落地的传送（源口、目标口、方块 id）；表现层闪烁并瞬时挪实体，不走移动动画
    pub teleport_flashes: Vec<(IVec3, IVec3, crate::blocks::BlockId)>,
    /// 激光打镜等非破坏火花
    pub behavior_sparks: Vec<IVec3>,
    /// 钻头/激光毁掉的材料碎片
    pub break_debris: Vec<BreakDebris>,
    pub laser_beams: Vec<LaserBeam>,
    /// 上一回合挂起、本回合开头落地的验收销毁：带材料种类，表现层采样贴图
    pub acceptance_sparks: Vec<BreakDebris>,
    pub stats: SimulationStepStats,
}

impl TurnOutput {
    /// 把语义阶段统一换算成表现延迟，避免粒子与声音各自猜测播放时机。
    pub fn presentation_delay(&self, phase: PresentationPhase, animation_duration: f32) -> f32 {
        match phase {
            PresentationPhase::TurnStart => 0.0,
            PresentationPhase::AfterMotion
                if !self.animations.is_empty()
                    || self
                        .pusher_animations
                        .values()
                        .any(|motion| motion.from_extension != motion.to_extension) =>
            {
                animation_duration
            }
            PresentationPhase::AfterMotion => 0.0,
        }
    }
}

use crate::session::SimulationDebugLog;
use bevy_ecs::prelude::*;

/// 一次回合的短生命周期工作区；仅保存阶段之间需要传递的数据。
#[derive(Resource)]
struct TurnWork {
    turn: u64,
    total_start: Instant,
    mark: Instant,
    sample: SimulationStepStats,
    log: Option<SimulationDebugLog>,
    break_debris: Vec<BreakDebris>,
    acceptance_sparks: Vec<BreakDebris>,
    laser_devices: HashSet<IVec3>,
    powered_devices: HashSet<IVec3>,
    powered_wire_ids: HashSet<crate::blocks::BlockId>,
    powered_device_ids: HashSet<crate::blocks::BlockId>,
    laser_beams: Vec<LaserBeam>,
    laser_probe_sparks: Vec<IVec3>,
    animations: HashMap<IVec3, BlockMotion>,
    pusher_animations: HashMap<IVec3, PusherMotion>,
    fragile_debris: Vec<(IVec3, crate::blocks::BlockKind)>,
    output: Option<TurnOutput>,
}

/// 可复用的回合执行器；GUI 和无头会话使用相同的 ECS 系统与顺序。
#[derive(Resource)]
pub struct TurnRunner {
    world: World,
    schedule: Schedule,
}

impl Default for TurnRunner {
    fn default() -> Self {
        let mut schedule = Schedule::default();
        schedule.add_systems((prepare_turn, resolve_signals, move_structures, finish_turn).chain());
        Self {
            world: World::new(),
            schedule,
        }
    }
}

impl TurnRunner {
    /// 原子推进回合：移动资源所有权，不复制世界；完成后归还宿主。
    pub fn run(
        &mut self,
        world: &mut WorldBlocks,
        pending_effects: &mut PendingTurnEffects,
        signal_cache: &mut SignalNetworkCache,
        turn: u64,
        structure_state: &mut StructureState,
        movement_history: &mut MovementHistory,
        pusher_state: &mut PusherState,
        mut sim_log: Option<&mut SimulationDebugLog>,
        stats: Option<&mut SimulationStepStats>,
    ) -> TurnOutput {
        let now = Instant::now();
        self.world.insert_resource(std::mem::take(world));
        self.world.insert_resource(std::mem::take(pending_effects));
        self.world.insert_resource(std::mem::take(signal_cache));
        self.world.insert_resource(std::mem::take(structure_state));
        self.world.insert_resource(std::mem::take(movement_history));
        self.world.insert_resource(std::mem::take(pusher_state));
        self.world.insert_resource(TurnWork {
            turn,
            total_start: now,
            mark: now,
            sample: SimulationStepStats::default(),
            log: sim_log.as_deref_mut().map(std::mem::take),
            break_debris: Vec::new(),
            acceptance_sparks: Vec::new(),
            laser_devices: HashSet::new(),
            powered_devices: HashSet::new(),
            powered_wire_ids: HashSet::new(),
            powered_device_ids: HashSet::new(),
            laser_beams: Vec::new(),
            laser_probe_sparks: Vec::new(),
            animations: HashMap::new(),
            pusher_animations: HashMap::new(),
            fragile_debris: Vec::new(),
            output: None,
        });
        self.schedule.run(&mut self.world);
        *world = self.world.remove_resource::<WorldBlocks>().unwrap();
        *pending_effects = self.world.remove_resource::<PendingTurnEffects>().unwrap();
        *signal_cache = self.world.remove_resource::<SignalNetworkCache>().unwrap();
        *structure_state = self.world.remove_resource::<StructureState>().unwrap();
        *movement_history = self.world.remove_resource::<MovementHistory>().unwrap();
        *pusher_state = self.world.remove_resource::<PusherState>().unwrap();
        let work = self.world.remove_resource::<TurnWork>().unwrap();
        if let Some(log) = sim_log {
            *log = work.log.unwrap();
        }
        let output = work.output.expect("turn schedule must produce its output");
        if let Some(stats) = stats {
            *stats = output.stats.clone();
        }
        output
    }
}

/// 单次调用入口；连续会话应保留 TurnRunner，复用已经初始化的系统。
pub fn simulate_turn(
    world: &mut WorldBlocks,
    pending_effects: &mut PendingTurnEffects,
    signal_cache: &mut SignalNetworkCache,
    turn: u64,
    structure_state: &mut StructureState,
    movement_history: &mut MovementHistory,
    pusher_state: &mut PusherState,
    sim_log: Option<&mut SimulationDebugLog>,
    stats: Option<&mut SimulationStepStats>,
) -> TurnOutput {
    TurnRunner::default().run(
        world,
        pending_effects,
        signal_cache,
        turn,
        structure_state,
        movement_history,
        pusher_state,
        sim_log,
        stats,
    )
}

/// 回合准备系统：提交上一回合延后效果与本回合初始生成。
fn prepare_turn(
    mut world: ResMut<WorldBlocks>,
    mut pending_effects: ResMut<PendingTurnEffects>,
    mut structure_state: ResMut<StructureState>,
    mut work: ResMut<TurnWork>,
) {
    let world = &mut *world;
    let pending_effects = &mut *pending_effects;
    let structure_state = &mut *structure_state;
    let TurnWork {
        turn,
        mark,
        sample,
        log: sim_log,
        break_debris,
        acceptance_sparks,
        ..
    } = &mut *work;
    let turn = *turn;
    if let Some(sim_log) = sim_log.as_mut() {
        sim_log.log(turn, "turn begin");
    }

    // 回合初：只重建静态 marker；不做焊/传送/生成/销毁落地
    run_static_marker_phase(world);
    sample.prep_ms = mark_elapsed_ms(mark);

    // 上一回合挂起的钻头/验收销毁：此时上一回合移动动画已结束，再真正移除
    let mut structures_dirty = false;
    for (pos, kind, reason) in pending_effects.take_ready_destroyed(turn) {
        if !world.is_material_at(pos) {
            continue;
        }
        world.remove(&pos);
        structures_dirty = true;
        match reason {
            PendingDestroyReason::Drill => {
                break_debris.push(BreakDebris { pos, kind });
            }
            PendingDestroyReason::Accept => {
                acceptance_sparks.push(BreakDebris { pos, kind });
            }
        }
    }
    // 上一回合挂起的滚刷漆 / 印花：停稳后再附着
    if apply_pending_paints(world, pending_effects, turn) {
        structures_dirty = true;
    }
    if apply_pending_stamps(world, pending_effects, turn) {
        structures_dirty = true;
    }
    // 首回合在模拟内部提交初始生成，GUI 预览和无头会话使用同一判定。
    if turn == 1 {
        pending_effects.schedule_generation(world, turn, &HashSet::new());
    }
    // 上一回合调度的生成：须在重力前落地，否则会多悬一回合才下落
    if place_ready_generated_materials(world, pending_effects, turn) {
        structures_dirty = true;
    }
    if structures_dirty {
        structure_state.refresh_material_structures(world);
    }
}

/// 信号系统：光学探测后计算供电，并保存移动前的方块身份。
fn resolve_signals(
    mut world: ResMut<WorldBlocks>,
    mut signal_cache: ResMut<SignalNetworkCache>,
    mut work: ResMut<TurnWork>,
) {
    let world = &mut *world;
    let signal_cache = &mut *signal_cache;
    let TurnWork {
        turn,
        mark,
        sample,
        log: sim_log,
        ..
    } = &mut *work;
    let turn = *turn;
    // —— 阶段 1 信号：光学探测（不销毁）→ 二次供电 ——
    signal_cache.refresh(world);
    let laser_power = signal_cache.powered_components(world, &HashSet::new());
    let laser_devices = signal_cache.powered_devices(world, &laser_power);
    let (laser_beams, laser_hit_detectors, laser_probe_sparks) =
        probe_lasers(world, &laser_devices);
    let powered_components = signal_cache.powered_components(world, &laser_hit_detectors);
    let powered_devices = signal_cache.powered_devices(world, &powered_components);
    let powered_wires = signal_cache.powered_wires(world, &powered_components);
    // 通电判定在位移前；输出给表现层前再按 BlockId 映射到移动后坐标（否则反推带着电线走会晚一拍才亮）
    let powered_wire_ids: HashSet<crate::blocks::BlockId> = powered_wires
        .iter()
        .filter_map(|pos| world.blocks.get(pos).map(|block| block.id))
        .collect();
    let powered_device_ids: HashSet<crate::blocks::BlockId> = powered_devices
        .iter()
        .filter_map(|pos| world.blocks.get(pos).map(|block| block.id))
        .collect();
    sample.signal_ms = mark_elapsed_ms(mark);
    if let Some(sim_log) = sim_log.as_mut() {
        sim_log.log(
            turn,
            format!(
                "signals: {} powered networks, {} powered devices, {} laser-hit detectors",
                powered_components.len(),
                powered_devices.len(),
                laser_hit_detectors.len()
            ),
        );
        for pos in powered_devices.iter().copied().collect::<Vec<_>>() {
            if let Some(block) = world.blocks.get(&pos) {
                sim_log.log(
                    turn,
                    format!(
                        "  powered device at ({}, {}, {}) {:?}",
                        pos.x, pos.y, pos.z, block.kind
                    ),
                );
            }
        }
    }

    work.laser_devices = laser_devices;
    work.powered_devices = powered_devices;
    work.powered_wire_ids = powered_wire_ids;
    work.powered_device_ids = powered_device_ids;
    work.laser_beams = laser_beams;
    work.laser_probe_sparks = laser_probe_sparks;
}

/// 运动系统：统一标记、仲裁并提交重力和设备驱动的位移。
fn move_structures(
    mut world: ResMut<WorldBlocks>,
    mut structure_state: ResMut<StructureState>,
    mut movement_history: ResMut<MovementHistory>,
    mut pusher_state: ResMut<PusherState>,
    mut work: ResMut<TurnWork>,
) {
    let world = &mut *world;
    let structure_state = &mut *structure_state;
    let movement_history = &mut *movement_history;
    let pusher_state = &mut *pusher_state;
    let TurnWork {
        turn,
        mark,
        sample,
        log: sim_log,
        powered_devices,
        ..
    } = &mut *work;
    let turn = *turn;
    let actuating_devices = pusher_state.actuating_devices(world, &powered_devices);
    let actuating_structure_ids: HashSet<_> = actuating_devices
        .iter()
        .filter_map(|pos| structure_state.id_at(*pos))
        .collect();
    for id in actuating_structure_ids {
        let needs_deform = structure_state.get(id).is_some_and(|structure| {
            structure.head_of.is_empty()
                && actuating_devices.iter().any(|pos| {
                    if structure_state.id_at(*pos) != Some(id) {
                        return false;
                    }
                    let Some(block) = world.blocks.get(pos) else {
                        return false;
                    };
                    let head = *pos + block.facing.forward_ivec3();
                    structure.activity == super::structure_state::FactoryActivity::Active
                        || world
                            .blocks
                            .get(&head)
                            .is_some_and(|head| head.kind == crate::blocks::BlockKind::PusherHead)
                        || (structure_state.id_at(head) == Some(id)
                            && !structure.scene_touching.contains(&head))
                })
        });
        if needs_deform {
            structure_state.rebuild_deform_for(world, id);
        }
    }
    if let Some(sim_log) = sim_log.as_mut() {
        for pos in actuating_devices.iter().copied().collect::<Vec<_>>() {
            sim_log.log(
                turn,
                format!(
                    "  actuating pusher/blocker at ({}, {}, {})",
                    pos.x, pos.y, pos.z
                ),
            );
        }
    }

    let suction = SuctionLinks::rebuild(world, structure_state, &powered_devices);

    // —— 阶段 2 运动标记：重力 + 通电设备，再 merge ——
    let hard_pusher_head_occupancy = PusherState::hard_head_occupancy(world);
    let mut movement_plan = mark_gravity_phase(
        world,
        structure_state,
        &HashSet::new(),
        &hard_pusher_head_occupancy,
        &suction,
    );
    sample.gravity_ms = mark_elapsed_ms(mark);
    if let Some(sim_log) = sim_log.as_mut() {
        log_movement_plan(turn, sim_log, world, "gravity", &movement_plan);
    }

    sample.marker_before_move_ms = mark_elapsed_ms(mark);

    let (device_movement_plan, conveyor_diag) = {
        structure_state.clear_turn_marks();
        mark_structure_movement_phase(
            world,
            &powered_devices,
            structure_state,
            pusher_state,
            &suction,
        )
    };
    if let Some(sim_log) = sim_log.as_mut() {
        let avg = if conveyor_diag.can_translate_calls == 0 {
            0.0
        } else {
            conveyor_diag.can_translate_ms / f64::from(conveyor_diag.can_translate_calls)
        };
        sim_log.log(
            turn,
            format!(
                "conveyor mark: attempts={} can_translate_calls={} ({:.2}ms, avg {:.3}ms/call) cache_hits={} emitted={} deduped={}",
                conveyor_diag.attempts,
                conveyor_diag.can_translate_calls,
                conveyor_diag.can_translate_ms,
                avg,
                conveyor_diag.cache_hits,
                conveyor_diag.emitted,
                conveyor_diag.deduped,
            ),
        );
        log_movement_plan(turn, sim_log, world, "devices", &device_movement_plan);
    }
    movement_plan = merge_structure_movement_plan(
        movement_plan,
        device_movement_plan,
        movement_history,
        structure_state,
        world,
    );
    movement_plan = arbitrate_movement_plan(world, structure_state, &suction, movement_plan);
    if let Some(sim_log) = sim_log.as_mut() {
        log_movement_plan(turn, sim_log, world, "merged", &movement_plan);
    }
    sample.movement_mark_ms = mark_elapsed_ms(mark);

    // —— 阶段 3a 脆弱碎裂：按运动计划移除冲突脆弱材料，再执行位姿 ——
    let fragile_debris =
        apply_fragile_shatter_before_execute(world, &mut movement_plan, structure_state);

    // —— 阶段 3b 执行运动：位姿/推杆，再重生静态 marker ——
    let (animations, pusher_animations, extension_commits) = execute_structure_moves_with_pushers(
        world,
        movement_plan,
        structure_state,
        movement_history,
        &hard_pusher_head_occupancy,
        &suction,
    );
    // 粘头/空头推动只有执行成功才提交伸出/收回（按 BlockId，避免互推后坐标过期）
    for (id, (pos, extended)) in extension_commits {
        pusher_state.set_extended(world, id, pos, extended);
    }
    let mut pusher_animations = pusher_animations;
    for (pos, animation) in pusher_state.sustained_animations(world) {
        pusher_animations.entry(pos).or_insert(animation);
    }
    sample.movement_execute_ms = mark_elapsed_ms(mark);

    run_static_marker_phase(world);
    sample.marker_after_move_ms = mark_elapsed_ms(mark);

    work.animations = animations;
    work.pusher_animations = pusher_animations;
    work.fragile_debris = fragile_debris;
}

/// 后处理系统：执行业务效果、刷新索引并生成本回合表现输出。
fn finish_turn(
    mut world: ResMut<WorldBlocks>,
    mut pending_effects: ResMut<PendingTurnEffects>,
    mut signal_cache: ResMut<SignalNetworkCache>,
    mut structure_state: ResMut<StructureState>,
    mut work: ResMut<TurnWork>,
) {
    let world = &mut *world;
    let pending_effects = &mut *pending_effects;
    let signal_cache = &mut *signal_cache;
    let structure_state = &mut *structure_state;
    let TurnWork {
        turn,
        mark,
        sample,
        log: sim_log,
        total_start,
        laser_devices,
        laser_probe_sparks,
        break_debris,
        fragile_debris,
        animations,
        pusher_animations,
        laser_beams,
        acceptance_sparks,
        powered_wire_ids,
        powered_device_ids,
        ..
    } = &mut *work;
    let turn = *turn;
    // —— 阶段 4 结构后处理（销毁 → 传送 → 转换 → 验收 → 生成 → 焊 → 漆/印花挂起）——
    let mut behavior_sparks = std::mem::take(laser_probe_sparks);
    break_debris.extend(
        std::mem::take(fragile_debris)
            .into_iter()
            .map(|(pos, kind)| BreakDebris { pos, kind }),
    );
    let had_materials = world.material_count > 0;
    // 材料销毁：钻头挂起至 turn+1；通电激光当场移除（与阶段 1 探测同一批激光设备）
    let (laser_destroy_sparks, laser_debris) = if had_materials || !laser_devices.is_empty() {
        run_material_destroy_phase(world, pending_effects, &laser_devices, turn + 1)
    } else {
        (Vec::new(), Vec::new())
    };
    behavior_sparks.extend(laser_destroy_sparks);
    break_debris.extend(laser_debris);

    // 传送：本回合当场落地（拆焊 + 搬到配对口），与焊接同拍
    let teleport_flashes = if had_materials && !world.system_blocks.is_empty() {
        run_material_teleport_phase(world)
    } else {
        Vec::new()
    };
    if had_materials && !world.system_blocks.is_empty() {
        run_material_conversion_phase(world);
    }

    // 验收：计数立刻生效，材料挂起至 turn+1 再删
    let accepted_acceptors = if had_materials && !structure_state.acceptor_structures().is_empty() {
        run_material_acceptance_phase(world, structure_state, pending_effects, turn)
    } else {
        HashSet::new()
    };

    // 只调度下一回合生成；落地已在回合初完成
    pending_effects.schedule_generation(world, turn + 1, &accepted_acceptors);

    let weld_sparks = if had_materials && !world.system_blocks.is_empty() {
        run_weld_behavior_phase(world)
    } else {
        Vec::new()
    };
    // 漆/印花：挂起至 turn+1，等本回合移动动画播完再附着
    if had_materials {
        run_material_label_phase(world, pending_effects, turn + 1);
    }
    if had_materials || world.material_count > 0 {
        structure_state.refresh_material_structures(world);
    }
    sample.behavior_ms = mark_elapsed_ms(mark);

    signal_cache.refresh(world);
    sample.signal_refresh_ms = mark_elapsed_ms(mark);
    sample.total_ms = total_start.elapsed().as_secs_f64() * 1000.0;
    sample.has_sample = true;

    if let Some(sim_log) = sim_log.as_mut() {
        sim_log.log(turn, format!("turn end: {:.2} ms", sample.total_ms));
    }

    // 表现用通电格：同一批通电 BlockId，落到本回合结束后的坐标
    let powered_wires: HashSet<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| powered_wire_ids.contains(&block.id).then_some(*pos))
        .collect();
    let powered_devices: HashSet<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| powered_device_ids.contains(&block.id).then_some(*pos))
        .collect();

    let output = TurnOutput {
        turn,
        animations: std::mem::take(animations),
        pusher_animations: std::mem::take(pusher_animations),
        powered_wires,
        powered_devices,
        weld_sparks,
        teleport_flashes,
        behavior_sparks,
        break_debris: std::mem::take(break_debris),
        laser_beams: std::mem::take(laser_beams),
        acceptance_sparks: std::mem::take(acceptance_sparks),
        stats: sample.clone(),
    };
    work.output = Some(output);
}

/// 落地 ready_turn 已到的生成材料，并焊接同参共生成块；有落地则返回 true
fn place_ready_generated_materials(
    world: &mut WorldBlocks,
    pending_effects: &mut PendingTurnEffects,
    turn: u64,
) -> bool {
    let ready = pending_effects.ready_pending_positions(turn);
    let mut placed = Vec::new();
    for pos in ready {
        let Some(block) = pending_effects.take_pending_block(pos) else {
            continue;
        };
        if world.can_place_platform_at(pos) {
            world.insert(pos, block);
            // 不播 SpawnScale：上一回合 pending 预览已播完生长，落地保持最终尺寸
            placed.push(pos);
        }
    }
    // 同参相连生成器本回合同时生成的材料焊接为同一结构
    weld_co_generated_materials(world, &placed);
    !placed.is_empty()
}

/// 同回合共生成且相邻的材料焊成一体
fn weld_co_generated_materials(world: &mut WorldBlocks, placed: &[IVec3]) {
    let offsets = [
        IVec3::X,
        IVec3::NEG_X,
        IVec3::Y,
        IVec3::NEG_Y,
        IVec3::Z,
        IVec3::NEG_Z,
    ];
    let placed_set: HashSet<IVec3> = placed.iter().copied().collect();
    for &pos in placed {
        let key = world.generator_settings(pos).trigger_key();
        for offset in offsets {
            let neighbor = pos + offset;
            if neighbor.x < pos.x
                || (neighbor.x == pos.x && neighbor.y < pos.y)
                || (neighbor.x == pos.x && neighbor.y == pos.y && neighbor.z <= pos.z)
            {
                continue;
            }
            if !placed_set.contains(&neighbor) || !world.is_material_at(neighbor) {
                continue;
            }
            if world.generator_settings(neighbor).trigger_key() == key {
                world.weld_materials(pos, neighbor);
            }
        }
    }
}

fn mark_elapsed_ms(mark: &mut Instant) -> f64 {
    let now = Instant::now();
    let elapsed = now.saturating_duration_since(*mark).as_secs_f64() * 1000.0;
    *mark = now;
    elapsed
}

fn log_movement_plan(
    turn: u64,
    sim_log: &mut crate::session::SimulationDebugLog,
    world: &WorldBlocks,
    label: &str,
    moves: &[StructureMove],
) {
    if moves.is_empty() {
        return;
    }
    sim_log.log(turn, format!("{label}: {} movement(s)", moves.len()));
    for movement in moves {
        match movement {
            StructureMove::Translate {
                structure,
                offset,
                actors,
                mark,
                source,
                ..
            } => {
                sim_log.log(
                    turn,
                    format!(
                        "  translate {} cell(s) by ({}, {}, {}) mark={mark:?} source={source:?} actors={actors:?}",
                        structure.len(),
                        offset.x,
                        offset.y,
                        offset.z,
                    ),
                );
                for pos in structure.iter().take(8) {
                    let kind = world
                        .blocks
                        .get(pos)
                        .map(|block| format!("{:?}", block.kind))
                        .unwrap_or_else(|| "?".into());
                    sim_log.log(
                        turn,
                        format!("    at ({}, {}, {}) {kind}", pos.x, pos.y, pos.z),
                    );
                }
                if structure.len() > 8 {
                    sim_log.log(turn, format!("    ... {} more", structure.len() - 8));
                }
            }
            StructureMove::Rotate {
                structure,
                pivot,
                clockwise,
                source,
                ..
            } => {
                sim_log.log(
                    turn,
                    format!(
                        "  rotate {} cell(s) pivot=({}, {}, {}) clockwise={clockwise} source={source:?}",
                        structure.len(),
                        pivot.x,
                        pivot.y,
                        pivot.z,
                    ),
                );
            }
        }
    }
}
