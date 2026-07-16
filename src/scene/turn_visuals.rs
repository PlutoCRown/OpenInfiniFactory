use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::systems::debug::DebugState;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::WorldRenderAssets;
use crate::sim_bridge::TurnOutput;

use super::entity_index::BlockEntityIndex;
use super::incremental::apply_turn_output_incremental;

pub fn apply_turn_output(
    before: &WorldBlocks,
    after: &WorldBlocks,
    output: &TurnOutput,
    previous_powered_wires: &HashSet<IVec3>,
    animation_duration: f32,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    index: &mut BlockEntityIndex,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &crate::game::simulation::structure_state::StructureState,
    stats: &mut crate::game::simulation::stats::SimulationStepStats,
) {
    apply_turn_output_incremental(
        before,
        after,
        output,
        previous_powered_wires,
        animation_duration,
        commands,
        meshes,
        index,
        render_assets,
        debug,
        structure_state,
        stats,
    );
}
