use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::game::blocks::{BlockKind, MovementRule};
use crate::game::world::animation::PusherAnimation;
use crate::game::world::grid::WorldBlocks;

use super::structure_state::StructureState;
use super::structures::{
    can_translate_structure, MovementMark, PusherActor, PusherAnimationKind, StructureMove,
};

#[derive(Resource, Default, Clone)]
pub struct PusherState {
    entries: HashMap<IVec3, PusherStateEntry>,
}

#[derive(Clone, Copy)]
struct PusherStateEntry {
    extended: bool,
    bound_front: bool,
}

impl PusherState {
    pub fn rebuild_from_world(world: &WorldBlocks) -> Self {
        let entries = world
            .blocks
            .iter()
            .filter_map(|(pos, block)| {
                matches!(block.kind, BlockKind::Pusher | BlockKind::Blocker).then_some((
                    *pos,
                    PusherStateEntry {
                        extended: false,
                        bound_front: world.is_factory_at(*pos + block.facing.forward_ivec3()),
                    },
                ))
            })
            .collect();
        Self { entries }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn sustained_animations(&self) -> std::collections::HashMap<IVec3, PusherAnimation> {
        self.entries
            .iter()
            .filter_map(|(pos, entry)| {
                entry.extended.then_some((
                    *pos,
                    PusherAnimation {
                        duration: 0.0,
                        from_extension: 1.0,
                        to_extension: 1.0,
                    },
                ))
            })
            .collect()
    }

    pub(super) fn actuating_devices(
        &self,
        world: &WorldBlocks,
        powered_devices: &HashSet<IVec3>,
    ) -> HashSet<IVec3> {
        world
            .blocks
            .iter()
            .filter_map(|(pos, block)| {
                let desired_extended = match block.kind {
                    BlockKind::Pusher => powered_devices.contains(pos),
                    BlockKind::Blocker => !powered_devices.contains(pos),
                    _ => return None,
                };
                let current_extended = self
                    .entries
                    .get(pos)
                    .map(|entry| entry.extended)
                    .unwrap_or(false);
                (desired_extended != current_extended).then_some(*pos)
            })
            .collect()
    }

    pub fn extended_head_positions(&self, world: &WorldBlocks) -> HashSet<IVec3> {
        self.hard_head_occupancy(world)
    }

    pub(super) fn hard_head_occupancy(&self, world: &WorldBlocks) -> HashSet<IVec3> {
        self.entries
            .iter()
            .filter_map(|(pos, entry)| {
                if !entry.extended {
                    return None;
                }
                let block = world.blocks.get(pos)?;
                matches!(block.kind, BlockKind::Pusher | BlockKind::Blocker)
                    .then_some(*pos + block.facing.forward_ivec3())
            })
            .collect()
    }
}

pub fn blocker_animations(
    world: &WorldBlocks,
    powered_devices: &HashSet<IVec3>,
) -> std::collections::HashMap<IVec3, PusherAnimation> {
    world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            (block.kind == BlockKind::Blocker).then_some((
                *pos,
                PusherAnimation {
                    duration: 0.0,
                    from_extension: if powered_devices.contains(pos) {
                        1.0
                    } else {
                        0.0
                    },
                    to_extension: if powered_devices.contains(pos) {
                        0.0
                    } else {
                        1.0
                    },
                },
            ))
        })
        .collect()
}

pub(super) fn mark_structure_movement_phase(
    world: &WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    structures: &StructureState,
    pusher_state: &mut PusherState,
) -> (Vec<StructureMove>, HashMap<IVec3, PusherAnimation>) {
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
    let mut bare_pusher_animations = HashMap::new();

    for (pos, kind, mover) in movers {
        match mover {
            MovementRule::Translate { source, offset } => {
                if let Some(movement) =
                    mark_conveyor_movement(world, structures, pos, source, offset)
                {
                    moves.push(movement.with_source(pos));
                }
            }
            MovementRule::Lift { range } => {
                if let Some(movement) = mark_lift_structure(world, structures, pos, range) {
                    moves.push(movement.with_source(pos));
                }
            }
            MovementRule::Rotate { clockwise } => {
                if let Some(movement) = mark_rotate_material_structure(structures, pos, clockwise) {
                    moves.push(movement.with_source(pos));
                }
            }
            MovementRule::PoweredTranslate { source, offset } => {
                if matches!(kind, BlockKind::Pusher | BlockKind::Blocker) {
                    let desired_extended = if kind == BlockKind::Pusher {
                        powered_devices.contains(&pos)
                    } else {
                        !powered_devices.contains(&pos)
                    };
                    if let Some(movement) = mark_pusher_movement(
                        world,
                        structures,
                        pusher_state,
                        pos,
                        source,
                        offset,
                        desired_extended,
                        &mut bare_pusher_animations,
                    ) {
                        moves.push(movement);
                    }
                } else if powered_devices.contains(&pos) {
                    if let Some(movement) = mark_structure_translate(
                        world,
                        structures,
                        pos,
                        pos + source,
                        offset,
                        MovementMark::Push,
                    ) {
                        moves.push(movement.with_source(pos));
                    }
                }
            }
        }
    }
    (moves, bare_pusher_animations)
}

fn mark_conveyor_movement(
    world: &WorldBlocks,
    structures: &StructureState,
    pos: IVec3,
    source: IVec3,
    offset: IVec3,
) -> Option<StructureMove> {
    let target = pos + source;
    if let Some(movement) = mark_structure_translate(
        world,
        structures,
        pos,
        target,
        offset,
        MovementMark::Conveyor,
    ) {
        if can_translate_structure(world, movement.structure(), offset, structures) {
            return Some(movement);
        }
    } else if !world.is_occupied(target) {
        return None;
    }

    let structure = structures.active_structure_at(pos, -offset)?;
    if !can_translate_structure(world, &structure, -offset, structures) {
        return None;
    }
    Some(StructureMove::translate_marked(
        structure,
        -offset,
        MovementMark::Conveyor,
    ))
}

fn mark_pusher_movement(
    world: &WorldBlocks,
    structures: &StructureState,
    pusher_state: &mut PusherState,
    pos: IVec3,
    source: IVec3,
    offset: IVec3,
    desired_extended: bool,
    bare_pusher_animations: &mut HashMap<IVec3, PusherAnimation>,
) -> Option<StructureMove> {
    let entry = pusher_state
        .entries
        .entry(pos)
        .or_insert_with(|| PusherStateEntry {
            extended: false,
            bound_front: world.is_factory_at(pos + source),
        });
    if desired_extended == entry.extended {
        return None;
    }

    let movement = if desired_extended {
        mark_structure_translate(
            world,
            structures,
            pos,
            pos + source,
            offset,
            MovementMark::Push,
        )
    } else if entry.bound_front {
        mark_structure_translate(
            world,
            structures,
            pos,
            pos + source + offset,
            -offset,
            MovementMark::Push,
        )
    } else {
        None
    };

    let state_changed = movement.is_some() || desired_extended || !entry.bound_front;
    if !state_changed {
        return None;
    }
    entry.extended = desired_extended;

    let (from_extension, to_extension) = if desired_extended {
        (0.0, 1.0)
    } else {
        (1.0, 0.0)
    };
    let animation = if desired_extended {
        PusherAnimationKind::Extend
    } else {
        PusherAnimationKind::Retract
    };

    if let Some(movement) = movement {
        return Some(
            movement
                .with_pusher_actor(pos, MovementMark::Push, animation)
                .with_source(pos),
        );
    }

    bare_pusher_animations.insert(
        pos,
        PusherAnimation {
            duration: 0.0,
            from_extension,
            to_extension,
        },
    );
    None
}

trait StructureMoveActorExt {
    fn with_pusher_actor(
        self,
        actor: IVec3,
        mark: MovementMark,
        animation: PusherAnimationKind,
    ) -> StructureMove;
}

impl StructureMoveActorExt for StructureMove {
    fn with_pusher_actor(
        self,
        actor: IVec3,
        mark: MovementMark,
        animation: PusherAnimationKind,
    ) -> StructureMove {
        match self {
            StructureMove::Translate {
                structure,
                offset,
                source,
                ..
            } => StructureMove::translate_by_pusher_actor(
                structure,
                offset,
                PusherActor {
                    pos: actor,
                    animation,
                },
                mark,
            )
            .with_optional_source(source),
            movement => movement,
        }
    }
}

trait StructureMoveSourceExt {
    fn with_optional_source(self, source: Option<IVec3>) -> StructureMove;
}

impl StructureMoveSourceExt for StructureMove {
    fn with_optional_source(self, source: Option<IVec3>) -> StructureMove {
        if let Some(source) = source {
            self.with_source(source)
        } else {
            self
        }
    }
}

fn mark_structure_translate(
    world: &WorldBlocks,
    structures: &StructureState,
    actor: IVec3,
    source: IVec3,
    offset: IVec3,
    mark: MovementMark,
) -> Option<StructureMove> {
    if world.is_material_at(source) {
        return structures
            .pushable_structure_at(source, offset)
            .map(|structure| StructureMove::translate_marked(structure, offset, mark));
    }

    let structure = if matches!(mark, MovementMark::Push)
        && world
            .blocks
            .get(&actor)
            .is_some_and(|block| matches!(block.kind, BlockKind::Pusher | BlockKind::Blocker))
    {
        structures.pusher_target_structure(world, actor, source, offset)?
    } else {
        if structures.structure_contains(source, actor) {
            return None;
        }
        structures.active_structure_at(source, offset)?
    };
    Some(StructureMove::translate_marked(structure, offset, mark))
}

fn mark_lift_structure(
    world: &WorldBlocks,
    structures: &StructureState,
    pos: IVec3,
    range: i32,
) -> Option<StructureMove> {
    let source = (1..=range)
        .map(|height| pos + IVec3::Y * height)
        .find(|candidate| {
            world.is_material_at(*candidate)
                || structures
                    .active_structure_at(*candidate, IVec3::Y)
                    .is_some()
        })?;

    mark_structure_translate(
        world,
        structures,
        pos,
        source,
        IVec3::Y,
        MovementMark::Vertical,
    )
}

fn mark_rotate_material_structure(
    structures: &StructureState,
    pos: IVec3,
    clockwise: bool,
) -> Option<StructureMove> {
    let source = pos + IVec3::Y;
    let structure = structures.pushable_structure_at(source, IVec3::ZERO)?;
    Some(StructureMove::rotate(structure, pos, clockwise))
}
