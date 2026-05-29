use crate::game::world::grid::WorldBlocks;

use super::structures::{factory_gravity_moves, material_gravity_moves, StructureMove};

pub(super) fn mark_gravity_phase(world: &WorldBlocks) -> Vec<StructureMove> {
    let mut moves = material_gravity_moves(world);
    moves.extend(factory_gravity_moves(world));
    moves
}
