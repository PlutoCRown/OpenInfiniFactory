use crate::world::grid::WorldBlocks;
use glam::IVec3;
use std::collections::HashSet;

use super::structure_state::StructureState;
use super::structures::{gravity_moves, StructureMove};

pub(super) fn mark_gravity_phase(
    world: &WorldBlocks,
    structures: &mut StructureState,
    skip_factory_positions: &HashSet<IVec3>,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> Vec<StructureMove> {
    gravity_moves(
        world,
        structures,
        skip_factory_positions,
        hard_pusher_head_occupancy,
    )
}
