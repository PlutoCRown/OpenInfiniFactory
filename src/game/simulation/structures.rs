use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::game::world::blocks::BlockData;
use crate::game::world::direction::Facing;
use crate::game::world::grid::{MaterialFace, MaterialFaceMark, MaterialWeld, WorldBlocks};

use super::signal_offsets;

pub(super) fn material_gravity_moves(world: &WorldBlocks) -> Vec<StructureMove> {
    let mut materials: Vec<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| block.kind.is_material().then_some(*pos))
        .collect();
    materials.sort_by_key(|pos| pos.y);

    let mut handled = HashSet::new();
    let mut moves = Vec::new();
    for pos in materials {
        if handled.contains(&pos) || !world.is_material_at(pos) {
            continue;
        };

        let structure = material_structure(world, pos);
        handled.extend(structure.iter().copied());
        if can_move_structure(world, &structure, IVec3::NEG_Y) {
            moves.push(StructureMove::translate(structure, IVec3::NEG_Y));
        }
    }
    moves
}

pub(super) fn factory_gravity_moves(world: &WorldBlocks) -> Vec<StructureMove> {
    let mut factory_blocks: Vec<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| block.kind.is_factory().then_some(*pos))
        .collect();
    factory_blocks.sort_by_key(|pos| pos.y);

    let mut handled = HashSet::new();
    let mut moves = Vec::new();
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
            moves.push(StructureMove::translate(structure, IVec3::NEG_Y));
        }
    }
    moves
}

pub(super) enum StructureMove {
    Translate {
        structure: HashSet<IVec3>,
        offset: IVec3,
    },
    Rotate {
        structure: HashSet<IVec3>,
        pivot: IVec3,
        clockwise: bool,
    },
}

impl StructureMove {
    pub(super) fn translate(structure: HashSet<IVec3>, offset: IVec3) -> Self {
        Self::Translate { structure, offset }
    }

    pub(super) fn rotate(structure: HashSet<IVec3>, pivot: IVec3, clockwise: bool) -> Self {
        Self::Rotate {
            structure,
            pivot,
            clockwise,
        }
    }
}

pub(super) fn execute_structure_moves(world: &mut WorldBlocks, moves: Vec<StructureMove>) {
    let mut moved = HashSet::new();
    for movement in moves {
        match movement {
            StructureMove::Translate { structure, offset } => {
                if structure.iter().any(|pos| moved.contains(pos)) {
                    continue;
                }
                if can_move_structure(world, &structure, offset) {
                    moved.extend(structure.iter().copied());
                    move_structure(world, &structure, offset);
                    moved.extend(structure.into_iter().map(|pos| pos + offset));
                }
            }
            StructureMove::Rotate {
                structure,
                pivot,
                clockwise,
            } => {
                if structure.iter().any(|pos| moved.contains(pos)) {
                    continue;
                }
                if can_rotate_structure(world, &structure, pivot, clockwise) {
                    let targets: Vec<IVec3> = structure
                        .iter()
                        .map(|pos| rotate_pos_y(*pos, pivot, clockwise))
                        .collect();
                    moved.extend(structure.iter().copied());
                    rotate_structure(world, &structure, pivot, clockwise);
                    moved.extend(targets);
                }
            }
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
    let updated_marks = moved_face_marks(world, structure, |face| MaterialFace {
        pos: face.pos + offset,
        normal: face.normal,
    });
    let blocks: Vec<(IVec3, BlockData)> = structure
        .iter()
        .filter_map(|pos| world.remove(pos).map(|block| (*pos, block)))
        .collect();

    for (pos, block) in blocks {
        world.insert(pos + offset, block);
    }
    world.replace_material_welds(updated_welds);
    world.replace_material_face_marks(updated_marks);
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
    let updated_marks = moved_face_marks(world, structure, |face| MaterialFace {
        pos: rotate_pos_y(face.pos, pivot, clockwise),
        normal: rotate_offset_y(face.normal, clockwise),
    });
    let blocks: Vec<(IVec3, BlockData)> = structure
        .iter()
        .filter_map(|pos| world.remove(pos).map(|block| (*pos, block)))
        .collect();

    for (pos, mut block) in blocks {
        block.facing = rotate_facing(block.facing, clockwise);
        world.insert(rotate_pos_y(pos, pivot, clockwise), block);
    }
    world.replace_material_welds(updated_welds);
    world.replace_material_face_marks(updated_marks);
}

pub(super) fn rotate_pos_y(pos: IVec3, pivot: IVec3, clockwise: bool) -> IVec3 {
    let rel = pos - pivot;
    pivot + rotate_offset_y(rel, clockwise)
}

fn rotate_offset_y(offset: IVec3, clockwise: bool) -> IVec3 {
    if clockwise {
        IVec3::new(-offset.z, offset.y, offset.x)
    } else {
        IVec3::new(offset.z, offset.y, -offset.x)
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

fn moved_face_marks(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    transform: impl Fn(MaterialFace) -> MaterialFace,
) -> HashMap<MaterialFace, MaterialFaceMark> {
    world
        .material_face_marks
        .iter()
        .map(|(face, mark)| {
            let face = if structure.contains(&face.pos) {
                transform(*face)
            } else {
                *face
            };
            (face, *mark)
        })
        .collect()
}
