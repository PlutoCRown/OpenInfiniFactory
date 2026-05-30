use crate::game::world::grid::WorldBlocks;

use super::factory_activity::FactoryStructureState;
use super::structures::{factory_gravity_moves, material_gravity_moves, StructureMove};

pub(super) fn mark_gravity_phase(
    world: &WorldBlocks,
    factory_structures: &FactoryStructureState,
) -> Vec<StructureMove> {
    let mut moves = material_gravity_moves(world, factory_structures);
    moves.extend(factory_gravity_moves(world, factory_structures));
    moves
}
