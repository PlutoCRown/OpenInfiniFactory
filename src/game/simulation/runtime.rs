use bevy::prelude::*;
use std::collections::HashMap;

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

pub fn run_turn(
    world: &mut WorldBlocks,
    signal_cache: &mut SignalNetworkCache,
    turn: u64,
    commands: &mut Commands,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
    animation_duration: f32,
) {
    let before = animation_snapshot(world);
    world.clear_generated_markers();

    let mut movement_plan = mark_gravity_phase(world);

    signal_cache.refresh(world);
    let powered_components = signal_cache.powered_components(world);
    let powered_devices = signal_cache.powered_devices(&powered_components);

    run_powered_marker_phase(world, &powered_devices);
    movement_plan.extend(mark_material_movement_phase(world, &powered_devices));
    execute_structure_moves(world, movement_plan);

    run_static_marker_phase(world);
    run_powered_marker_phase(world, &powered_devices);
    run_material_behavior_phase(world, turn, &powered_devices);

    signal_cache.refresh(world);

    let animations = pair_block_animations(&before, &animation_snapshot(world));
    despawn_world(commands, block_entities);
    rebuild_world_with_timed_animations(
        commands,
        world,
        render_assets,
        &animations,
        AnimationTiming::simulation(animation_duration),
    );
}

pub fn tick_simulation(
    time: Res<Time>,
    builder_mode: Res<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut world: ResMut<WorldBlocks>,
    mut signal_cache: ResMut<SignalNetworkCache>,
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
        );
    }
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
