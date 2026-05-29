use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::world::blocks::MaterialMover;
use crate::game::world::grid::WorldBlocks;

use super::structures::{material_structure, StructureMove};

pub(super) fn mark_material_movement_phase(
    world: &WorldBlocks,
    powered_devices: &HashSet<IVec3>,
) -> Vec<StructureMove> {
    let movers: Vec<(IVec3, MaterialMover)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            block
                .kind
                .material_mover(block.facing)
                .map(|mover| (*pos, mover))
        })
        .collect();
    let mut moves = Vec::new();

    for (pos, mover) in movers {
        match mover {
            MaterialMover::Conveyor { source, offset } => {
                if let Some(movement) = mark_material_translate(world, pos + source, offset) {
                    moves.push(movement);
                }
            }
            MaterialMover::Lifter => {
                if let Some(movement) = mark_lift_material_structure(world, pos) {
                    moves.push(movement);
                }
            }
            MaterialMover::Rotator { clockwise } => {
                if let Some(movement) = mark_rotate_material_structure(world, pos, clockwise) {
                    moves.push(movement);
                }
            }
            MaterialMover::Piston { source, offset } => {
                if powered_devices.contains(&pos) {
                    if let Some(movement) = mark_material_translate(world, pos + source, offset) {
                        moves.push(movement);
                    }
                }
            }
        }
    }
    moves
}

fn mark_material_translate(
    world: &WorldBlocks,
    source: IVec3,
    offset: IVec3,
) -> Option<StructureMove> {
    if !world.is_material_at(source) {
        return None;
    }

    let structure = material_structure(world, source);
    Some(StructureMove::translate(structure, offset))
}

fn mark_lift_material_structure(world: &WorldBlocks, pos: IVec3) -> Option<StructureMove> {
    let source = (1..=5)
        .map(|height| pos + IVec3::Y * height)
        .find(|candidate| world.is_material_at(*candidate))?;

    mark_material_translate(world, source, IVec3::Y)
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
