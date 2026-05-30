use std::collections::{HashMap, HashSet};
use std::time::Instant;

use bevy::prelude::*;

use crate::game::state::{BuilderMode, SimulationState};
use crate::game::systems::debug::DebugState;
use crate::game::world::animation::{
    AnimationTiming, BlockAnimation, BlockAnimationKind, SIMULATION_TURN_SECONDS,
};
use crate::game::world::blocks::BlockData;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    despawn_pending_generated_previews, despawn_world,
    rebuild_world_with_runtime_animations_for_debug_state, spawn_pending_generated_block,
    spawn_weld_sparks, BlockEntity, PendingGeneratedPreview, WorldRenderAssets,
};

use super::behaviors::{material_source_generation, run_material_behavior_phase};
use super::factory_activity::FactoryStructureState;
use super::gravity::mark_gravity_phase;
use super::markers::{run_powered_marker_phase, run_static_marker_phase};
use super::movement::mark_structure_movement_phase;
pub use super::signals::SignalNetworkCache;
use super::structures::{execute_structure_moves_with_pushers, merge_structure_movement_plan};

#[derive(Resource, Clone)]
pub struct SimulationStepStats {
    pub has_sample: bool,
    pub total_ms: f64,
    pub prep_ms: f64,
    pub gravity_ms: f64,
    pub signal_ms: f64,
    pub marker_before_move_ms: f64,
    pub movement_mark_ms: f64,
    pub movement_execute_ms: f64,
    pub marker_after_move_ms: f64,
    pub behavior_ms: f64,
    pub signal_refresh_ms: f64,
    pub render_rebuild_ms: f64,
}

#[derive(Resource, Default)]
pub struct PendingGeneratedMaterials {
    pending: HashMap<IVec3, PendingGeneratedMaterial>,
}

impl PendingGeneratedMaterials {
    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

struct PendingGeneratedMaterial {
    block: BlockData,
    ready_turn: u64,
}

impl Default for SimulationStepStats {
    fn default() -> Self {
        Self {
            has_sample: false,
            total_ms: 0.0,
            prep_ms: 0.0,
            gravity_ms: 0.0,
            signal_ms: 0.0,
            marker_before_move_ms: 0.0,
            movement_mark_ms: 0.0,
            movement_execute_ms: 0.0,
            marker_after_move_ms: 0.0,
            behavior_ms: 0.0,
            signal_refresh_ms: 0.0,
            render_rebuild_ms: 0.0,
        }
    }
}

pub fn run_turn(
    world: &mut WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    signal_cache: &mut SignalNetworkCache,
    turn: u64,
    commands: &mut Commands,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
    animation_duration: f32,
    debug: &DebugState,
    factory_structures: &mut FactoryStructureState,
    stats: &mut SimulationStepStats,
) {
    let total_start = Instant::now();
    let mut mark = total_start;
    let mut sample = SimulationStepStats::default();

    world.clear_generated_markers();
    let generated_animations = place_ready_generated_materials(world, pending_generated, turn);
    sample.prep_ms = mark_elapsed_ms(&mut mark);

    let mut movement_plan = mark_gravity_phase(world, factory_structures);
    sample.gravity_ms = mark_elapsed_ms(&mut mark);

    signal_cache.refresh(world);
    let powered_components = signal_cache.powered_components(world);
    let powered_devices = signal_cache.powered_devices(&powered_components);
    sample.signal_ms = mark_elapsed_ms(&mut mark);

    run_powered_marker_phase(world, &powered_devices);
    sample.marker_before_move_ms = mark_elapsed_ms(&mut mark);

    let device_movement_plan =
        mark_structure_movement_phase(world, &powered_devices, factory_structures);
    movement_plan = merge_structure_movement_plan(
        movement_plan,
        device_movement_plan,
        world,
        factory_structures,
    );
    sample.movement_mark_ms = mark_elapsed_ms(&mut mark);

    let (mut animations, pusher_animations) =
        execute_structure_moves_with_pushers(world, movement_plan, factory_structures);
    merge_generated_animations(&mut animations, generated_animations);
    let pusher_animations = pusher_animations
        .into_iter()
        .map(|(pos, mut animation)| {
            animation.duration = animation_duration;
            (pos, animation)
        })
        .collect();
    sample.movement_execute_ms = mark_elapsed_ms(&mut mark);

    run_static_marker_phase(world);
    run_powered_marker_phase(world, &powered_devices);
    sample.marker_after_move_ms = mark_elapsed_ms(&mut mark);

    let weld_sparks = run_material_behavior_phase(world, &powered_devices, factory_structures);

    prepare_upcoming_generation(world, pending_generated, turn + 1);
    sample.behavior_ms = mark_elapsed_ms(&mut mark);

    signal_cache.refresh(world);
    let render_powered_components = signal_cache.powered_components(world);
    let powered_wires = signal_cache.powered_wires(&render_powered_components);
    sample.signal_refresh_ms = mark_elapsed_ms(&mut mark);

    despawn_world(commands, block_entities);
    rebuild_world_with_runtime_animations_for_debug_state(
        commands,
        world,
        render_assets,
        &animations,
        &pusher_animations,
        AnimationTiming::simulation(animation_duration),
        debug,
        factory_structures,
        &powered_wires,
    );
    spawn_weld_sparks(commands, render_assets, &weld_sparks);
    sample.render_rebuild_ms = mark_elapsed_ms(&mut mark);
    sample.total_ms = total_start.elapsed().as_secs_f64() * 1000.0;
    sample.has_sample = true;
    *stats = sample;
}

pub fn tick_simulation(
    time: Res<Time>,
    builder_mode: Res<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut world: ResMut<WorldBlocks>,
    mut pending_generated: ResMut<PendingGeneratedMaterials>,
    mut signal_cache: ResMut<SignalNetworkCache>,
    mut sim_stats: ResMut<SimulationStepStats>,
    mut commands: Commands,
    block_entities: Query<Entity, With<BlockEntity>>,
    pending_previews: Query<Entity, With<PendingGeneratedPreview>>,
    render_assets: Res<WorldRenderAssets>,
    debug: Res<DebugState>,
    mut factory_structures: ResMut<FactoryStructureState>,
) {
    if *builder_mode != BuilderMode::Play || (!simulation.running && !simulation.step_requested) {
        prepare_upcoming_generation(&world, &mut pending_generated, simulation.turn + 1);
        refresh_pending_generated_previews(
            &mut commands,
            &pending_previews,
            &render_assets,
            &world,
            &pending_generated,
            simulation.turn,
            simulation.accumulator,
        );
        return;
    }

    if simulation.step_requested {
        simulation.step_requested = false;
        simulation.accumulator = 0.0;
        simulation.turn += 1;
        run_turn(
            &mut world,
            &mut pending_generated,
            &mut signal_cache,
            simulation.turn,
            &mut commands,
            &block_entities,
            &render_assets,
            SIMULATION_TURN_SECONDS,
            &debug,
            &mut factory_structures,
            &mut sim_stats,
        );
        prepare_upcoming_generation(&world, &mut pending_generated, simulation.turn + 1);
        refresh_pending_generated_previews(
            &mut commands,
            &pending_previews,
            &render_assets,
            &world,
            &pending_generated,
            simulation.turn,
            simulation.accumulator,
        );
        return;
    }

    simulation.accumulator += time.delta_secs() * simulation.speed / SIMULATION_TURN_SECONDS;
    while simulation.accumulator >= 1.0 {
        simulation.turn += 1;
        simulation.accumulator -= 1.0;
        run_turn(
            &mut world,
            &mut pending_generated,
            &mut signal_cache,
            simulation.turn,
            &mut commands,
            &block_entities,
            &render_assets,
            SIMULATION_TURN_SECONDS / simulation.speed.max(0.001),
            &debug,
            &mut factory_structures,
            &mut sim_stats,
        );
    }

    prepare_upcoming_generation(&world, &mut pending_generated, simulation.turn + 1);
    refresh_pending_generated_previews(
        &mut commands,
        &pending_previews,
        &render_assets,
        &world,
        &pending_generated,
        simulation.turn,
        simulation.accumulator,
    );
}

fn prepare_upcoming_generation(
    world: &WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    ready_turn: u64,
) {
    let blocked_generation: HashSet<IVec3> = pending_generated.pending.keys().copied().collect();
    let generated = material_source_generation(world, ready_turn, &blocked_generation);
    for generated in generated {
        pending_generated
            .pending
            .entry(generated.pos)
            .or_insert(PendingGeneratedMaterial {
                block: generated.block,
                ready_turn,
            });
    }
}

fn place_ready_generated_materials(
    world: &mut WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    turn: u64,
) -> HashMap<IVec3, BlockAnimation> {
    let mut ready = Vec::new();
    for (pos, pending) in &pending_generated.pending {
        if pending.ready_turn <= turn {
            ready.push(*pos);
        }
    }

    let mut animations = HashMap::new();
    for pos in ready {
        let Some(pending) = pending_generated.pending.remove(&pos) else {
            continue;
        };
        if world.can_place_platform_at(pos) {
            world.insert(pos, pending.block);
            animations.insert(
                pos,
                BlockAnimation {
                    from_pos: pos,
                    to_pos: pos,
                    from_facing: pending.block.facing,
                    to_facing: pending.block.facing,
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

fn refresh_pending_generated_previews(
    commands: &mut Commands,
    pending_previews: &Query<Entity, With<PendingGeneratedPreview>>,
    render_assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pending_generated: &PendingGeneratedMaterials,
    turn: u64,
    accumulator: f32,
) {
    despawn_pending_generated_previews(commands, pending_previews);
    spawn_pending_generated_previews(
        commands,
        render_assets,
        world,
        pending_generated,
        turn,
        accumulator,
    );
}

fn spawn_pending_generated_previews(
    commands: &mut Commands,
    render_assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pending_generated: &PendingGeneratedMaterials,
    turn: u64,
    accumulator: f32,
) {
    for (pos, pending) in &pending_generated.pending {
        let progress = if pending.ready_turn <= turn {
            1.0
        } else if pending.ready_turn == turn + 1 {
            accumulator
        } else {
            0.0
        }
        .clamp(0.0, 1.0);

        spawn_pending_generated_block(
            commands,
            render_assets,
            world,
            *pos,
            pending.block,
            Some(BlockAnimation {
                from_pos: *pos,
                to_pos: *pos,
                from_facing: pending.block.facing,
                to_facing: pending.block.facing,
                kind: BlockAnimationKind::SpawnScale,
                duration: Some(SIMULATION_TURN_SECONDS),
                progress: Some(progress),
            }),
            AnimationTiming::simulation(SIMULATION_TURN_SECONDS),
        );
    }
}

fn mark_elapsed_ms(mark: &mut Instant) -> f64 {
    let now = Instant::now();
    let elapsed = now.saturating_duration_since(*mark).as_secs_f64() * 1000.0;
    *mark = now;
    elapsed
}
