use crate::game::world::grid::WorldBlocks;
use bevy::prelude::*;
use std::collections::HashSet;

use super::factory_activity::FactoryStructureState;
use super::structures::{factory_gravity_moves, material_gravity_moves, StructureMove};

pub(super) fn mark_gravity_phase(
    world: &WorldBlocks,
    factory_structures: &FactoryStructureState,
    skip_factory_positions: &HashSet<IVec3>,
) -> Vec<StructureMove> {
    let mut moves = material_gravity_moves(world, factory_structures);
    moves.extend(factory_gravity_moves(
        world,
        factory_structures,
        skip_factory_positions,
    ));
    moves
}
