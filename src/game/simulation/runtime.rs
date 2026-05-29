use std::collections::HashMap;
use std::time::Instant;

use bevy::prelude::*;

use crate::game::state::{BuilderMode, SimulationState};
use crate::game::world::animation::{
    pair_block_animations, AnimationTiming, SIMULATION_TURN_SECONDS,
};
use crate::game::world::blocks::BlockKind;
use crate::game::world::direction::Facing;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    despawn_world, rebuild_world_with_timed_animations, BlockEntity, WorldRenderAssets,
};

use super::behaviors::run_material_behavior_phase;
use super::gravity::mark_gravity_phase;
use super::markers::{run_powered_marker_phase, run_static_marker_phase};
use super::movement::mark_material_movement_phase;
pub use super::signals::SignalNetworkCache;
use super::structures::execute_structure_moves;

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
    signal_cache: &mut SignalNetworkCache,
    turn: u64,
    commands: &mut Commands,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
    animation_duration: f32,
    stats: &mut SimulationStepStats,
) {
    let total_start = Instant::now();
    let mut mark = total_start;
    let mut sample = SimulationStepStats::default();

    let before = animation_snapshot(world);
    world.clear_generated_markers();
    sample.prep_ms = mark_elapsed_ms(&mut mark);

    let mut movement_plan = mark_gravity_phase(world);
    sample.gravity_ms = mark_elapsed_ms(&mut mark);

    signal_cache.refresh(world);
    let powered_components = signal_cache.powered_components(world);
    let powered_devices = signal_cache.powered_devices(&powered_components);
    sample.signal_ms = mark_elapsed_ms(&mut mark);

    run_powered_marker_phase(world, &powered_devices);
    sample.marker_before_move_ms = mark_elapsed_ms(&mut mark);

    movement_plan.extend(mark_material_movement_phase(world, &powered_devices));
    sample.movement_mark_ms = mark_elapsed_ms(&mut mark);

    execute_structure_moves(world, movement_plan);
    sample.movement_execute_ms = mark_elapsed_ms(&mut mark);

    run_static_marker_phase(world);
    run_powered_marker_phase(world, &powered_devices);
    sample.marker_after_move_ms = mark_elapsed_ms(&mut mark);

    run_material_behavior_phase(world, turn, &powered_devices);
    sample.behavior_ms = mark_elapsed_ms(&mut mark);

    signal_cache.refresh(world);
    sample.signal_refresh_ms = mark_elapsed_ms(&mut mark);

    let animations = pair_block_animations(&before, &animation_snapshot(world));
    despawn_world(commands, block_entities);
    rebuild_world_with_timed_animations(
        commands,
        world,
        render_assets,
        &animations,
        AnimationTiming::simulation(animation_duration),
    );
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
    mut signal_cache: ResMut<SignalNetworkCache>,
    mut sim_stats: ResMut<SimulationStepStats>,
    mut commands: Commands,
    block_entities: Query<Entity, With<BlockEntity>>,
    render_assets: Res<WorldRenderAssets>,
) {
    if *builder_mode != BuilderMode::Play || !simulation.running {
        return;
    }

    simulation.accumulator += time.delta_seconds() * simulation.speed / SIMULATION_TURN_SECONDS;
    while simulation.accumulator >= 1.0 {
        simulation.turn += 1;
        simulation.accumulator -= 1.0;
        run_turn(
            &mut world,
            &mut signal_cache,
            simulation.turn,
            &mut commands,
            &block_entities,
            &render_assets,
            SIMULATION_TURN_SECONDS / simulation.speed.max(0.001),
            &mut sim_stats,
        );
    }
}

fn mark_elapsed_ms(mark: &mut Instant) -> f64 {
    let now = Instant::now();
    let elapsed = now.saturating_duration_since(*mark).as_secs_f64() * 1000.0;
    *mark = now;
    elapsed
}

fn animation_snapshot(world: &WorldBlocks) -> HashMap<IVec3, (BlockKind, Facing)> {
    world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            (!block.kind.is_generated_marker()).then_some((*pos, (block.kind, block.facing)))
        })
        .collect()
}
