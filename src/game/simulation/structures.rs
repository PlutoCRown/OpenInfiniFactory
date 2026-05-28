use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

use crate::game::world::blocks::BlockData;
use crate::game::world::direction::Facing;
use crate::game::world::grid::{MaterialWeld, WorldBlocks};

use super::signal_offsets;

pub(super) fn apply_material_gravity(world: &mut WorldBlocks) {
    let mut materials: Vec<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| block.kind.is_material().then_some(*pos))
        .collect();
    materials.sort_by_key(|pos| pos.y);

    let mut handled = HashSet::new();
    for pos in materials {
        if handled.contains(&pos) || !world.is_material_at(pos) {
            continue;
        };

        let structure = material_structure(world, pos);
        handled.extend(structure.iter().copied());
        if can_move_structure(world, &structure, IVec3::NEG_Y) {
            move_structure(world, &structure, IVec3::NEG_Y);
        }
    }
}

pub(super) fn apply_factory_gravity(world: &mut WorldBlocks) {
    let mut factory_blocks: Vec<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| block.kind.is_factory().then_some(*pos))
        .collect();
    factory_blocks.sort_by_key(|pos| pos.y);

    let mut handled = HashSet::new();
    for pos in factory_blocks {
        if handled.contains(&pos) || !world.is_factory_at(pos) {
            continue;
        };

        let structure = factory_structure(world, pos);
        handled.extend(structure.iter().copied());
        if factory_structure_is_anchored(world, &structure) {
            continue;
        }
        if can_move_structure(world, &structure, IVec3::NEG_Y) {
            move_block_structure(world, &structure, IVec3::NEG_Y);
        }
    }
}

pub(super) fn material_structure(world: &WorldBlocks, start: IVec3) -> HashSet<IVec3> {
    let mut structure = HashSet::new();
    let mut queue = VecDeque::from([start]);
    structure.insert(start);

    while let Some(pos) = queue.pop_front() {
        for neighbor in welded_neighbors(world, pos) {
            if structure.contains(&neighbor) {
                continue;
            }
            structure.insert(neighbor);
            queue.push_back(neighbor);
        }
    }

    structure
}

fn factory_structure(world: &WorldBlocks, start: IVec3) -> HashSet<IVec3> {
    let mut structure = HashSet::new();
    let mut queue = VecDeque::from([start]);
    structure.insert(start);

    while let Some(pos) = queue.pop_front() {
        for offset in signal_offsets() {
            let neighbor = pos + offset;
            if structure.contains(&neighbor) || !world.is_factory_at(neighbor) {
                continue;
            }
            structure.insert(neighbor);
            queue.push_back(neighbor);
        }
    }

    structure
}

fn factory_structure_is_anchored(world: &WorldBlocks, structure: &HashSet<IVec3>) -> bool {
    structure.iter().any(|pos| {
        signal_offsets()
            .into_iter()
            .any(|offset| world.is_scene_at(*pos + offset))
    })
}

fn welded_neighbors(world: &WorldBlocks, pos: IVec3) -> Vec<IVec3> {
    world
        .material_welds
        .iter()
        .filter_map(|weld| weld.other(pos))
        .filter(|neighbor| world.is_material_at(*neighbor))
        .collect()
}

pub(super) fn can_move_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    offset: IVec3,
) -> bool {
    structure.iter().all(|pos| {
        let target = *pos + offset;
        target.y >= 0 && (structure.contains(&target) || world.can_place_solid_at(target))
    })
}

pub(super) fn move_structure(world: &mut WorldBlocks, structure: &HashSet<IVec3>, offset: IVec3) {
    let updated_welds = moved_welds(world, structure, |pos| pos + offset);
    let blocks: Vec<(IVec3, BlockData)> = structure
        .iter()
        .filter_map(|pos| world.remove(pos).map(|block| (*pos, block)))
        .collect();

    for (pos, block) in blocks {
        world.insert(pos + offset, block);
    }
    world.replace_material_welds(updated_welds);
}

fn move_block_structure(world: &mut WorldBlocks, structure: &HashSet<IVec3>, offset: IVec3) {
    let blocks: Vec<(IVec3, BlockData)> = structure
        .iter()
        .filter_map(|pos| world.remove(pos).map(|block| (*pos, block)))
        .collect();

    for (pos, block) in blocks {
        world.insert(pos + offset, block);
    }
}

pub(super) fn can_rotate_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    pivot: IVec3,
    clockwise: bool,
) -> bool {
    structure.iter().all(|pos| {
        let target = rotate_pos_y(*pos, pivot, clockwise);
        target.y >= 0 && (structure.contains(&target) || world.can_place_solid_at(target))
    })
}

pub(super) fn rotate_structure(
    world: &mut WorldBlocks,
    structure: &HashSet<IVec3>,
    pivot: IVec3,
    clockwise: bool,
) {
    let updated_welds = moved_welds(world, structure, |pos| rotate_pos_y(pos, pivot, clockwise));
    let blocks: Vec<(IVec3, BlockData)> = structure
        .iter()
        .filter_map(|pos| world.remove(pos).map(|block| (*pos, block)))
        .collect();

    for (pos, mut block) in blocks {
        block.facing = rotate_facing(block.facing, clockwise);
        world.insert(rotate_pos_y(pos, pivot, clockwise), block);
    }
    world.replace_material_welds(updated_welds);
}

fn rotate_pos_y(pos: IVec3, pivot: IVec3, clockwise: bool) -> IVec3 {
    let rel = pos - pivot;
    if clockwise {
        pivot + IVec3::new(-rel.z, rel.y, rel.x)
    } else {
        pivot + IVec3::new(rel.z, rel.y, -rel.x)
    }
}

fn rotate_facing(facing: Facing, clockwise: bool) -> Facing {
    if clockwise {
        facing.rotate()
    } else {
        facing.rotate_counter()
    }
}

fn moved_welds(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    transform: impl Fn(IVec3) -> IVec3,
) -> HashSet<MaterialWeld> {
    world
        .material_welds
        .iter()
        .filter_map(|weld| {
            let a = if structure.contains(&weld.a) {
                transform(weld.a)
            } else {
                weld.a
            };
            let b = if structure.contains(&weld.b) {
                transform(weld.b)
            } else {
                weld.b
            };
            (a != b).then_some(MaterialWeld::new(a, b))
        })
        .collect()
}
