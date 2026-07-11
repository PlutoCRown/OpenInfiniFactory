use bevy::platform::time::Instant;
use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::game::world::animation::{BlockAnimation, BlockAnimationKind, PusherAnimation};
use crate::game::world::grid::WorldBlocks;

use super::behaviors::{
    material_source_generation, run_laser_phase, run_material_behavior_phase,
    run_ready_material_teleports, run_weld_behavior_phase, LaserBeam,
};
use super::gravity::mark_gravity_phase;
use super::markers::run_static_marker_phase;
use super::movement::{mark_structure_movement_phase, PusherState};
use super::runtime::{PendingGeneratedMaterials, SignalNetworkCache, SimulationStepStats};
use super::structure_state::StructureState;
use super::structures::{
    execute_structure_moves_with_pushers, merge_structure_movement_plan, MovementInfluenceCache,
    StructureMove,
};

#[derive(Clone)]
pub struct TurnOutput {
    pub turn: u64,
    pub animations: HashMap<IVec3, BlockAnimation>,
    pub pusher_animations: HashMap<IVec3, PusherAnimation>,
    pub render_powered_wires: HashSet<IVec3>,
    pub weld_sparks: Vec<IVec3>,
    pub behavior_sparks: Vec<IVec3>,
    pub laser_beams: Vec<LaserBeam>,
    pub acceptance_sparks: Vec<IVec3>,
    pub stats: SimulationStepStats,
}

pub fn simulate_turn(
    world: &mut WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    signal_cache: &mut SignalNetworkCache,
    turn: u64,
    structure_state: &mut StructureState,
    movement_influence: &mut MovementInfluenceCache,
    pusher_state: &mut PusherState,
    mut sim_log: Option<&mut crate::sim_core::SimulationDebugLog>,
    stats: Option<&mut SimulationStepStats>,
) -> TurnOutput {
    let total_start = Instant::now();
    let mut mark = total_start;
    let mut sample = SimulationStepStats::default();

    if let Some(sim_log) = sim_log.as_mut() {
        sim_log.log(turn, "turn begin");
    }

    world.clear_generated_markers();
    let acceptance_sparks = remove_ready_destroyed_materials(world, pending_generated, turn);
    run_ready_material_teleports(world, pending_generated, turn);
    let generated_animations = place_ready_generated_materials(world, pending_generated, turn);
    run_static_marker_phase(world);
    let weld_sparks = run_weld_behavior_phase(world);
    // 材料结构随焊接更新；工厂连通在开局已按放置关系固定，运行时不因相邻合并
    structure_state.refresh_material_structures(world);
    sample.prep_ms = mark_elapsed_ms(&mut mark);

    // 先按平台/材料算电，通电激光本回合先发射；打中传感器工作面后再二次供电
    signal_cache.refresh(world);
    let laser_power = signal_cache.powered_components(world, &HashSet::new());
    let laser_devices = signal_cache.powered_devices(world, &laser_power);
    let (laser_effects, laser_hit_detectors) = run_laser_phase(world, &laser_devices);
    let powered_components = signal_cache.powered_components(world, &laser_hit_detectors);
    let powered_devices = signal_cache.powered_devices(world, &powered_components);
    let render_powered_wires = signal_cache.powered_wires(world, &powered_components);
    sample.signal_ms = mark_elapsed_ms(&mut mark);
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

    let actuating_devices = pusher_state.actuating_devices(world, &powered_devices);
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

    // 重力与设备都只做标记；空头伸出是 Push 零位移标签，执行时按优先级压过重力
    let hard_pusher_head_occupancy = pusher_state.hard_head_occupancy(world);
    let mut movement_plan = mark_gravity_phase(
        world,
        structure_state,
        &HashSet::new(),
        &hard_pusher_head_occupancy,
    );
    sample.gravity_ms = mark_elapsed_ms(&mut mark);
    if let Some(sim_log) = sim_log.as_mut() {
        log_movement_plan(turn, sim_log, world, "gravity", &movement_plan);
    }

    sample.marker_before_move_ms = mark_elapsed_ms(&mut mark);

    let device_movement_plan =
        mark_structure_movement_phase(world, &powered_devices, structure_state, pusher_state);
    if let Some(sim_log) = sim_log.as_mut() {
        log_movement_plan(turn, sim_log, world, "devices", &device_movement_plan);
    }
    movement_plan = merge_structure_movement_plan(
        movement_plan,
        device_movement_plan,
        movement_influence,
        structure_state,
        world,
    );
    if let Some(sim_log) = sim_log.as_mut() {
        log_movement_plan(turn, sim_log, world, "merged", &movement_plan);
    }
    sample.movement_mark_ms = mark_elapsed_ms(&mut mark);

    let (mut animations, pusher_animations) = execute_structure_moves_with_pushers(
        world,
        movement_plan,
        structure_state,
        movement_influence,
        &hard_pusher_head_occupancy,
    );
    // 粘头/空头推动只有执行成功才提交伸出/收回
    for (pos, animation) in &pusher_animations {
        if let Some(block) = world.blocks.get(pos) {
            pusher_state.set_extended(block.id, animation.to_extension > 0.5);
        }
    }
    merge_generated_animations(&mut animations, generated_animations);
    let mut pusher_animations = pusher_animations;
    for (pos, animation) in pusher_state.sustained_animations(world) {
        pusher_animations.entry(pos).or_insert(animation);
    }
    sample.movement_execute_ms = mark_elapsed_ms(&mut mark);

    run_static_marker_phase(world);
    sample.marker_after_move_ms = mark_elapsed_ms(&mut mark);

    let behavior_effects =
        run_material_behavior_phase(world, structure_state, pending_generated, turn + 1);
    structure_state.refresh_material_structures(world);

    prepare_upcoming_generation(
        world,
        pending_generated,
        turn + 1,
        &behavior_effects.accepted_acceptors,
    );
    sample.behavior_ms = mark_elapsed_ms(&mut mark);

    signal_cache.refresh(world);
    sample.signal_refresh_ms = mark_elapsed_ms(&mut mark);
    sample.total_ms = total_start.elapsed().as_secs_f64() * 1000.0;
    sample.has_sample = true;

    if let Some(sim_log) = sim_log.as_mut() {
        sim_log.log(turn, format!("turn end: {:.2} ms", sample.total_ms));
    }
    if let Some(stats) = stats {
        *stats = sample.clone();
    }

    let mut behavior_sparks = laser_effects.sparks;
    behavior_sparks.extend(behavior_effects.sparks);

    TurnOutput {
        turn,
        animations,
        pusher_animations,
        render_powered_wires,
        weld_sparks,
        behavior_sparks,
        laser_beams: laser_effects.laser_beams,
        acceptance_sparks,
        stats: sample,
    }
}

pub fn prepare_upcoming_generation(
    world: &WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    ready_turn: u64,
    accepted_acceptors: &HashSet<crate::game::blocks::AcceptorId>,
) {
    let blocked_generation: HashSet<IVec3> = pending_generated.pending_keys().collect();
    let generated =
        material_source_generation(world, ready_turn, &blocked_generation, accepted_acceptors);
    for generated in generated {
        pending_generated.insert_pending(generated.pos, generated.block, ready_turn);
    }
}

fn place_ready_generated_materials(
    world: &mut WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    turn: u64,
) -> HashMap<IVec3, BlockAnimation> {
    let ready = pending_generated.ready_pending_positions(turn);
    let mut animations = HashMap::new();
    let mut placed = Vec::new();
    for pos in ready {
        let Some(block) = pending_generated.take_pending_block(pos) else {
            continue;
        };
        if world.can_place_platform_at(pos) {
            world.insert(pos, block);
            let id = world.blocks.get(&pos).map(|b| b.id).unwrap_or(block.id);
            animations.insert(
                pos,
                BlockAnimation {
                    block_id: id,
                    from_pos: pos,
                    to_pos: pos,
                    from_facing: block.facing,
                    to_facing: block.facing,
                    kind: BlockAnimationKind::SpawnScale,
                    duration: None,
                    progress: None,
                },
            );
            placed.push(pos);
        }
    }
    // 同参相连生成器本回合同时生成的材料焊接为同一结构
    weld_co_generated_materials(world, &placed);
    animations
}

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

fn remove_ready_destroyed_materials(
    world: &mut WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    turn: u64,
) -> Vec<IVec3> {
    let ready = pending_generated.ready_destroyed_positions(turn);
    let mut acceptance_sparks = Vec::new();
    for pos in ready {
        pending_generated.remove_destroyed(pos);
        if pending_generated.take_acceptance_spark(pos).is_some() {
            acceptance_sparks.push(pos);
        }
        if world.is_material_at(pos) {
            world.remove(&pos);
        }
    }
    acceptance_sparks
}

fn merge_generated_animations(
    animations: &mut HashMap<IVec3, BlockAnimation>,
    generated_animations: HashMap<IVec3, BlockAnimation>,
) {
    for (generated_pos, generated_animation) in generated_animations {
        let moved_target = animations.iter().find_map(|(target, animation)| {
            (animation.from_pos == generated_pos).then_some(*target)
        });
        if moved_target.is_none() {
            animations.insert(generated_pos, generated_animation);
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
    sim_log: &mut crate::sim_core::SimulationDebugLog,
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
                actor,
                mark,
                source,
                ..
            } => {
                sim_log.log(
                    turn,
                    format!(
                        "  translate {} cell(s) by ({}, {}, {}) mark={mark:?} source={source:?} actor={actor:?}",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::blocks::{BlockData, BlockKind, MaterialKind};
    use crate::game::simulation::movement::PusherState;
    use crate::game::simulation::runtime::PendingGeneratedMaterials;
    use crate::game::world::direction::Facing;
    use crate::game::world::grid::{GeneratorMode, GeneratorSettings};

    #[test]
    fn co_generated_adjacent_materials_are_welded() {
        let mut world = WorldBlocks::default();
        let a = IVec3::ZERO;
        let b = IVec3::X;
        world.insert(a, BlockData::new(BlockKind::Generator, Facing::North));
        world.insert(b, BlockData::new(BlockKind::Generator, Facing::North));
        let settings = GeneratorSettings {
            mode: GeneratorMode::Period {
                period: 1,
                offset: 0,
            },
            material: MaterialKind::Basic,
        };
        world.set_generator_settings(a, settings);
        world.set_generator_settings(
            b,
            GeneratorSettings {
                mode: GeneratorMode::Period {
                    period: 1,
                    offset: 0,
                },
                material: MaterialKind::Iron,
            },
        );

        let mut pending = PendingGeneratedMaterials::default();
        prepare_upcoming_generation(&world, &mut pending, 1, &HashSet::new());
        let animations = place_ready_generated_materials(&mut world, &mut pending, 1);
        assert_eq!(animations.len(), 2);
        assert!(world.is_material_at(a));
        assert!(world.is_material_at(b));
        let id_a = world.blocks[&a].id;
        let id_b = world.blocks[&b].id;
        assert!(world
            .material_welds
            .contains(&crate::game::world::grid::MaterialWeld::new(id_a, id_b)));
    }

    #[test]
    fn floating_blocker_extends_once_then_falls_every_turn() {
        let mut world = WorldBlocks::default();
        world.insert(
            IVec3::new(0, 0, 0),
            BlockData::new(BlockKind::Stone, Facing::North),
        );
        world.insert(
            IVec3::new(0, 5, 0),
            BlockData::new(BlockKind::Blocker, Facing::East),
        );

        let mut pending = PendingGeneratedMaterials::default();
        let mut signal_cache = SignalNetworkCache::default();
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut influence = MovementInfluenceCache::default();
        let mut pusher_state = PusherState::rebuild_from_world(&world);

        let turn1 = simulate_turn(
            &mut world,
            &mut pending,
            &mut signal_cache,
            1,
            &mut structures,
            &mut influence,
            &mut pusher_state,
            None,
            None,
        );
        assert!(world.is_factory_at(IVec3::new(0, 5, 0)));
        assert!(pusher_state
            .sustained_animations(&world)
            .contains_key(&IVec3::new(0, 5, 0)));
        assert!(turn1
            .pusher_animations
            .get(&IVec3::new(0, 5, 0))
            .is_some_and(|a| a.from_extension < a.to_extension));

        let turn2 = simulate_turn(
            &mut world,
            &mut pending,
            &mut signal_cache,
            2,
            &mut structures,
            &mut influence,
            &mut pusher_state,
            None,
            None,
        );
        assert!(world.is_factory_at(IVec3::new(0, 4, 0)));
        assert!(!world.is_factory_at(IVec3::new(0, 5, 0)));
        assert!(pusher_state
            .sustained_animations(&world)
            .contains_key(&IVec3::new(0, 4, 0)));
        assert!(
            !turn2
                .pusher_animations
                .values()
                .any(|a| a.from_extension != a.to_extension),
            "already extended: must not replay extend while falling"
        );

        simulate_turn(
            &mut world,
            &mut pending,
            &mut signal_cache,
            3,
            &mut structures,
            &mut influence,
            &mut pusher_state,
            None,
            None,
        );
        assert!(world.is_factory_at(IVec3::new(0, 3, 0)));
        assert!(pusher_state
            .sustained_animations(&world)
            .contains_key(&IVec3::new(0, 3, 0)));
    }

    fn sim_world(
        world: WorldBlocks,
    ) -> (
        WorldBlocks,
        PendingGeneratedMaterials,
        SignalNetworkCache,
        StructureState,
        MovementInfluenceCache,
        PusherState,
    ) {
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let pusher_state = PusherState::rebuild_from_world(&world);
        (
            world,
            PendingGeneratedMaterials::default(),
            SignalNetworkCache::default(),
            structures,
            MovementInfluenceCache::default(),
            pusher_state,
        )
    }

    #[test]
    fn extending_head_blocks_falling_block_same_cell() {
        // 阻拦器伸出头优先于上方方块下落，二者不得重叠
        let mut world = WorldBlocks::default();
        world.insert(
            IVec3::new(0, 0, 0),
            BlockData::new(BlockKind::Stone, Facing::North),
        );
        world.insert(
            IVec3::new(0, 1, 0),
            BlockData::new(BlockKind::Blocker, Facing::East),
        );
        world.insert(
            IVec3::new(1, 2, 0),
            BlockData::new(BlockKind::Platform, Facing::North),
        );

        let (mut world, mut pending, mut signals, mut structures, mut influence, mut pushers) =
            sim_world(world);
        simulate_turn(
            &mut world,
            &mut pending,
            &mut signals,
            1,
            &mut structures,
            &mut influence,
            &mut pushers,
            None,
            None,
        );

        assert!(
            pushers
                .sustained_animations(&world)
                .contains_key(&IVec3::new(0, 1, 0)),
            "阻拦器应成功伸出"
        );
        assert!(
            world.is_factory_at(IVec3::new(1, 2, 0)),
            "上方平台不应落到头所在格"
        );
        assert!(
            !world.blocks.contains_key(&IVec3::new(1, 1, 0)),
            "头格不应被下落方块占用"
        );
    }

    #[test]
    fn failed_head_contest_falls_same_turn() {
        // 面对面争夺同一头格：胜者本回合伸出，败者本回合下落
        let mut world = WorldBlocks::default();
        world.insert(
            IVec3::new(0, 2, 0),
            BlockData::new(BlockKind::Blocker, Facing::East),
        );
        world.insert(
            IVec3::new(2, 2, 0),
            BlockData::new(BlockKind::Blocker, Facing::West),
        );

        let (mut world, mut pending, mut signals, mut structures, mut influence, mut pushers) =
            sim_world(world);
        simulate_turn(
            &mut world,
            &mut pending,
            &mut signals,
            1,
            &mut structures,
            &mut influence,
            &mut pushers,
            None,
            None,
        );

        let at_y2 = [IVec3::new(0, 2, 0), IVec3::new(2, 2, 0)]
            .iter()
            .filter(|pos| world.is_factory_at(**pos))
            .count();
        let at_y1 = [IVec3::new(0, 1, 0), IVec3::new(2, 1, 0)]
            .iter()
            .filter(|pos| world.is_factory_at(**pos))
            .count();
        assert_eq!(at_y2, 1, "胜者留在原高度并伸出");
        assert_eq!(at_y1, 1, "败者同回合下落");
        assert_eq!(
            pushers.hard_head_occupancy(&world).len(),
            1,
            "只有胜者头占位"
        );
    }

    #[test]
    fn extended_head_supports_another_extended_head() {
        // 伸出的头是实体，可垫住另一伸出结构的头
        let mut world = WorldBlocks::default();
        world.insert(
            IVec3::new(2, 0, 0),
            BlockData::new(BlockKind::Stone, Facing::North),
        );
        world.insert(
            IVec3::new(2, 1, 0),
            BlockData::new(BlockKind::Blocker, Facing::West),
        );
        world.insert(
            IVec3::new(0, 3, 0),
            BlockData::new(BlockKind::Blocker, Facing::East),
        );

        let (mut world, mut pending, mut signals, mut structures, mut influence, mut pushers) =
            sim_world(world);
        let support_id = world.blocks[&IVec3::new(2, 1, 0)].id;
        let falling_id = world.blocks[&IVec3::new(0, 3, 0)].id;
        pushers.set_extended(support_id, true);
        pushers.set_extended(falling_id, true);

        for turn in 1..=5 {
            simulate_turn(
                &mut world,
                &mut pending,
                &mut signals,
                turn,
                &mut structures,
                &mut influence,
                &mut pushers,
                None,
                None,
            );
        }

        assert!(
            world.is_factory_at(IVec3::new(0, 2, 0)),
            "下落阻拦器应被下方头垫住，身子停在 y=2"
        );
        assert!(
            !world.is_factory_at(IVec3::new(0, 1, 0)),
            "不应穿过下方活塞头继续下落"
        );
        assert!(world.is_factory_at(IVec3::new(2, 1, 0)));
    }
}
