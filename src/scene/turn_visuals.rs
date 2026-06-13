use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::systems::debug::DebugState;
use crate::game::world::block_instance::MaterialBlockRegistry;
use crate::game::world::factory_registry::FactoryBlockRegistry;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{BlockEntity, WorldRenderAssets};
use crate::sim_core::TurnOutput;

use super::block_entities::BlockEntityTracker;
use super::incremental::{
    apply_turn_output_incremental, resync_world_rendering, snap_block_entities_to_world,
};

pub fn apply_turn_output(
    before: &WorldBlocks,
    after: &WorldBlocks,
    output: &TurnOutput,
    previous_powered_wires: &HashSet<IVec3>,
    presentation_duration: f32,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    tracker: &mut BlockEntityTracker,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &crate::game::simulation::structure_state::StructureState,
    factory_registry: &FactoryBlockRegistry,
    material_registry: &MaterialBlockRegistry,
    stats: &mut crate::game::simulation::runtime::SimulationStepStats,
) {
    apply_turn_output_incremental(
        before,
        after,
        output,
        previous_powered_wires,
        presentation_duration,
        commands,
        meshes,
        tracker,
        render_assets,
        debug,
        structure_state,
        factory_registry,
        material_registry,
        stats,
    );
}

pub fn apply_turn_output_resync(
    after: &WorldBlocks,
    output: &TurnOutput,
    presentation_duration: f32,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    blocks: &Query<(Entity, &BlockEntity)>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &crate::game::simulation::structure_state::StructureState,
    factory_registry: &FactoryBlockRegistry,
    material_registry: &MaterialBlockRegistry,
    stats: &mut crate::game::simulation::runtime::SimulationStepStats,
) {
    resync_world_rendering(
        after,
        output,
        presentation_duration,
        commands,
        meshes,
        blocks,
        render_assets,
        debug,
        structure_state,
        factory_registry,
        material_registry,
        stats,
    );
}

pub fn snap_visuals_to_world(
    world: &WorldBlocks,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    blocks: &Query<(Entity, &BlockEntity)>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &crate::game::simulation::structure_state::StructureState,
    factory_registry: &FactoryBlockRegistry,
    material_registry: &MaterialBlockRegistry,
) {
    snap_block_entities_to_world(
        world,
        commands,
        meshes,
        blocks,
        render_assets,
        debug,
        structure_state,
        factory_registry,
        material_registry,
    );
}
