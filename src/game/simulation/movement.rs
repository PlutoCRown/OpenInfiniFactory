use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::world::blocks::{BlockKind, MovementRule};
use crate::game::world::grid::WorldBlocks;

use super::factory_activity::FactoryStructureState;
use super::structures::{material_structure, StructureMove};

pub(super) fn mark_structure_movement_phase(
    world: &WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    factory_structures: &FactoryStructureState,
) -> Vec<StructureMove> {
    let movers: Vec<(IVec3, BlockKind, MovementRule)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            block
                .kind
                .movement_rule(block.facing)
                .map(|mover| (*pos, block.kind, mover))
        })
        .collect();
    let mut moves = Vec::new();

    for (pos, kind, mover) in movers {
        match mover {
            MovementRule::Translate { source, offset } => {
                if let Some(movement) =
                    mark_structure_translate(world, factory_structures, pos, pos + source, offset)
                {
                    moves.push(movement);
                }
            }
            MovementRule::Lift { range } => {
                if let Some(movement) = mark_lift_structure(world, factory_structures, pos, range) {
                    moves.push(movement);
                }
            }
            MovementRule::Rotate { clockwise } => {
                if let Some(movement) = mark_rotate_material_structure(world, pos, clockwise) {
                    moves.push(movement);
                }
            }
            MovementRule::PoweredTranslate { source, offset } => {
                if powered_devices.contains(&pos) {
                    if let Some(movement) = mark_structure_translate(
                        world,
                        factory_structures,
                        pos,
                        pos + source,
                        offset,
                    ) {
                        if kind == BlockKind::Piston {
                            moves.push(movement.with_actor(pos));
                        } else {
                            moves.push(movement);
                        }
                    }
                }
            }
        }
    }
    moves
}

trait StructureMoveActorExt {
    fn with_actor(self, actor: IVec3) -> StructureMove;
}

impl StructureMoveActorExt for StructureMove {
    fn with_actor(self, actor: IVec3) -> StructureMove {
        match self {
            StructureMove::Translate {
                structure, offset, ..
            } => StructureMove::translate_by_actor(structure, offset, actor),
            movement => movement,
        }
    }
}

fn mark_structure_translate(
    world: &WorldBlocks,
    factory_structures: &FactoryStructureState,
    actor: IVec3,
    source: IVec3,
    offset: IVec3,
) -> Option<StructureMove> {
    if world.is_material_at(source) {
        return Some(StructureMove::translate(
            material_structure(world, source),
            offset,
        ));
    }

    if factory_structures.structure_contains(source, actor) {
        return None;
    }
    let structure = factory_structures.active_structure_at(source, offset)?;
    Some(StructureMove::translate(structure, offset))
}

fn mark_lift_structure(
    world: &WorldBlocks,
    factory_structures: &FactoryStructureState,
    pos: IVec3,
    range: i32,
) -> Option<StructureMove> {
    let source = (1..=range)
        .map(|height| pos + IVec3::Y * height)
        .find(|candidate| {
            world.is_material_at(*candidate)
                || factory_structures
                    .active_structure_at(*candidate, IVec3::Y)
                    .is_some()
        })?;

    mark_structure_translate(world, factory_structures, pos, source, IVec3::Y)
}

fn mark_rotate_material_structure(
    world: &WorldBlocks,
    pos: IVec3,
    clockwise: bool,
) -> Option<StructureMove> {
    let source = pos + IVec3::Y;
    if !world.is_material_at(source) {
        return None;
    }

    let structure = material_structure(world, source);
    Some(StructureMove::rotate(structure, pos, clockwise))
}
