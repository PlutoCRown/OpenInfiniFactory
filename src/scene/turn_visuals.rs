use bevy::prelude::*;

use crate::game::systems::debug::DebugState;
use crate::game::world::animation::AnimationTiming;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    despawn_world, rebuild_world_with_runtime_animations_for_debug_state, spawn_acceptance_sparks,
    spawn_laser_beams, spawn_weld_sparks, BlockEntity, WorldRenderAssets,
};
use crate::sim_core::TurnOutput;

pub fn apply_turn_output(
    output: &TurnOutput,
    world: &WorldBlocks,
    animation_duration: f32,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &crate::game::simulation::structure_state::StructureState,
    stats: &mut crate::game::simulation::runtime::SimulationStepStats,
) {
    let render_start = bevy::platform::time::Instant::now();
    despawn_world(commands, block_entities);
    rebuild_world_with_runtime_animations_for_debug_state(
        commands,
        meshes,
        world,
        render_assets,
        &output.animations,
        &output.pusher_animations,
        AnimationTiming::simulation(animation_duration),
        debug,
        structure_state,
        &output.render_powered_wires,
    );
    spawn_weld_sparks(commands, render_assets, &output.weld_sparks);
    spawn_weld_sparks(commands, render_assets, &output.behavior_sparks);
    spawn_laser_beams(
        commands,
        render_assets,
        &output.laser_beams,
        animation_duration * 0.5,
    );
    spawn_acceptance_sparks(commands, render_assets, &output.acceptance_sparks);
    stats.render_rebuild_ms = render_start.elapsed().as_secs_f64() * 1000.0;
    stats.total_ms += stats.render_rebuild_ms;
}
