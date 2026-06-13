use bevy::platform::time::Instant;
use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::game::world::animation::{BlockAnimation, BlockAnimationKind, PusherAnimation};
use crate::game::world::grid::WorldBlocks;

use super::behaviors::{material_source_generation, run_material_behavior_phase, LaserBeam};
use super::markers::{run_powered_marker_phase, run_static_marker_phase};
use super::movement::PusherState;
use super::movement_plan::{collect_movement_plan, execute_movement_plan};
use super::runtime::{PendingGeneratedMaterials, SignalNetworkCache, SimulationStepStats};
use super::structures::MovementInfluenceCache;
use super::SimulationWorlds;

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
    worlds: &mut SimulationWorlds,
    pending_generated: &mut PendingGeneratedMaterials,
    signal_cache: &mut SignalNetworkCache,
    turn: u64,
    animation_duration: f32,
    pusher_state: &mut PusherState,
    movement_influence: &mut MovementInfluenceCache,
    mut sim_log: Option<&mut crate::sim_core::SimulationDebugLog>,
    stats: Option<&mut SimulationStepStats>,
) -> TurnOutput {
    let total_start = Instant::now();
    let mut mark = total_start;
    let mut sample = SimulationStepStats::default();

    if let Some(sim_log) = sim_log.as_mut() {
        sim_log.log(turn, "turn begin");
    }

    worlds.turn.clear_generated_markers();
    let generated_animations =
        place_ready_generated_materials(&mut worlds.turn, pending_generated, turn);
    sample.prep_ms = mark_elapsed_ms(&mut mark);

    signal_cache.refresh(&worlds.turn);
    let powered_components = signal_cache.powered_components(&worlds.turn);
    let powered_devices = signal_cache.powered_devices(&powered_components);
    let render_powered_wires = signal_cache.powered_wires(&powered_components);
    sample.signal_ms = mark_elapsed_ms(&mut mark);

    let movement_plan = collect_movement_plan(
        &worlds.turn,
        &worlds.solution,
        &mut worlds.turn_structures,
        &worlds.solution_structures,
        &worlds.factory_registry,
        &powered_devices,
        pusher_state,
        movement_influence,
    );
    sample.movement_mark_ms = mark_elapsed_ms(&mut mark);

    let mut realtime = worlds.turn.clone();
    let movement_output = execute_movement_plan(
        &movement_plan,
        &mut realtime,
        &mut worlds.turn_structures,
        &mut worlds.factory_registry,
        pusher_state,
        movement_influence,
    );
    worlds.turn = realtime;
    sample.movement_execute_ms = mark_elapsed_ms(&mut mark);

    let mut animations = movement_output.animations;
    merge_generated_animations(&mut animations, generated_animations);
    let pusher_animations = movement_output
        .pusher_animations
        .into_iter()
        .map(|(pos, mut animation)| {
            animation.duration = animation_duration;
            (pos, animation)
        })
        .collect::<HashMap<_, _>>();

    run_static_marker_phase(&mut worlds.turn);
    run_powered_marker_phase(&mut worlds.turn, &powered_devices, pusher_state);
    sample.marker_after_move_ms = mark_elapsed_ms(&mut mark);

    let behavior_effects = run_material_behavior_phase(
        &mut worlds.turn,
        &powered_devices,
        &mut worlds.turn_structures,
    );
    worlds
        .turn_structures
        .refresh_material_structures(&worlds.turn);

    prepare_upcoming_generation(&worlds.turn, pending_generated, turn + 1);
    sample.behavior_ms = mark_elapsed_ms(&mut mark);

    signal_cache.refresh(&worlds.turn);
    sample.signal_refresh_ms = mark_elapsed_ms(&mut mark);
    sample.total_ms = total_start.elapsed().as_secs_f64() * 1000.0;
    sample.has_sample = true;

    if let Some(sim_log) = sim_log.as_mut() {
        sim_log.log(turn, format!("turn end: {:.2} ms", sample.total_ms));
    }
    if let Some(stats) = stats {
        *stats = sample.clone();
    }

    TurnOutput {
        turn,
        animations,
        pusher_animations,
        render_powered_wires,
        weld_sparks: behavior_effects.weld_sparks,
        behavior_sparks: behavior_effects.sparks,
        laser_beams: behavior_effects.laser_beams,
        acceptance_sparks: behavior_effects.acceptance_sparks,
        stats: sample,
    }
}

pub fn prepare_upcoming_generation(
    world: &WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    ready_turn: u64,
) {
    let blocked_generation: HashSet<IVec3> = pending_generated.pending_keys().collect();
    let generated = material_source_generation(world, ready_turn, &blocked_generation);
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
    for pos in ready {
        let Some(block) = pending_generated.take_pending_block(pos) else {
            continue;
        };
        if world.can_place_platform_at(pos) {
            world.insert(pos, block);
            animations.insert(
                pos,
                BlockAnimation {
                    from_pos: pos,
                    to_pos: pos,
                    from_facing: block.facing,
                    to_facing: block.facing,
                    kind: BlockAnimationKind::SpawnScale,
                    duration: None,
                    progress: None,
                },
            );
        }
    }
    animations
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
