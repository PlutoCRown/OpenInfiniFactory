use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::game::blocks::{BlockKind, MovementRule};
use crate::game::world::animation::PusherAnimation;
use crate::game::world::grid::WorldBlocks;

use super::structure_state::StructureState;
use super::structures::{
    can_translate_structure, MovementCandidate, MovementMark, PusherActor, PusherAnimationKind,
    StructureMove,
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

    pub fn set_extended(&mut self, pos: IVec3, world: &WorldBlocks, extended: bool) {
        let bound_front = world.is_factory_at(
            pos + world
                .blocks
                .get(&pos)
                .map(|block| block.facing.forward_ivec3())
                .unwrap_or(IVec3::ZERO),
        );
        self.entries
            .entry(pos)
            .and_modify(|entry| entry.extended = extended)
            .or_insert(PusherStateEntry {
                extended,
                bound_front,
            });
    }

    fn is_extended(&self, pos: IVec3) -> bool {
        self.entries
            .get(&pos)
            .map(|entry| entry.extended)
            .unwrap_or(false)
    }

    fn is_bound_front(&self, pos: IVec3, world: &WorldBlocks) -> bool {
        self.entries
            .get(&pos)
            .map(|entry| entry.bound_front)
            .unwrap_or_else(|| world.is_factory_at(pos + forward(world, pos)))
    }

    pub fn sustained_animations(&self) -> HashMap<IVec3, PusherAnimation> {
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
                let current_extended = self.is_extended(*pos);
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

fn forward(world: &WorldBlocks, pos: IVec3) -> IVec3 {
    world
        .blocks
        .get(&pos)
        .map(|block| block.facing.forward_ivec3())
        .unwrap_or(IVec3::ZERO)
}

pub fn blocker_animations(
    world: &WorldBlocks,
    powered_devices: &HashSet<IVec3>,
) -> HashMap<IVec3, PusherAnimation> {
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

pub(super) fn sorted_factory_movers(world: &WorldBlocks) -> Vec<(IVec3, BlockKind, MovementRule)> {
    let mut movers: Vec<(IVec3, BlockKind, MovementRule)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            block
                .kind
                .movement_rule(block.facing)
                .map(|mover| (*pos, block.kind, mover))
        })
        .collect();
    movers.sort_by_key(|(pos, _, _)| (pos.x, pos.y, pos.z));
    movers
}

pub(super) struct MovementMarkContext<'a> {
    pub turn: &'a WorldBlocks,
    pub solution: &'a WorldBlocks,
    pub turn_structures: &'a StructureState,
    pub solution_structures: &'a StructureState,
}

pub(super) struct PusherMarkResult {
    pub candidate: Option<MovementCandidate>,
    pub bare_animation: Option<(IVec3, PusherAnimation)>,
}

pub(super) fn collect_pusher_candidate(
    ctx: &MovementMarkContext<'_>,
    pos: IVec3,
    source: IVec3,
    offset: IVec3,
    desired_extended: bool,
    pusher_state: &PusherState,
) -> PusherMarkResult {
    if desired_extended == pusher_state.is_extended(pos) {
        return PusherMarkResult {
            candidate: None,
            bare_animation: None,
        };
    }

    let movement = if desired_extended {
        mark_structure_translate(ctx, pos, pos + source, offset, MovementMark::Push)
    } else if pusher_state.is_bound_front(pos, ctx.turn) {
        mark_structure_translate(
            ctx,
            pos,
            pos + source + offset,
            -offset,
            MovementMark::Push,
        )
    } else {
        None
    };

    let animation = if desired_extended {
        PusherAnimationKind::Extend
    } else {
        PusherAnimationKind::Retract
    };
    let (from_extension, to_extension) = if desired_extended {
        (0.0, 1.0)
    } else {
        (1.0, 0.0)
    };

    if let Some(movement) = movement {
        return PusherMarkResult {
            candidate: Some(MovementCandidate {
                primary: movement
                    .with_pusher_actor(pos, MovementMark::Push, animation)
                    .with_source(pos),
                fallbacks: Vec::new(),
            }),
            bare_animation: None,
        };
    }

    PusherMarkResult {
        candidate: None,
        bare_animation: Some((
            pos,
            PusherAnimation {
                duration: 0.0,
                from_extension,
                to_extension,
            },
        )),
    }
}

pub(super) fn collect_powered_translate_candidate(
    ctx: &MovementMarkContext<'_>,
    pos: IVec3,
    source: IVec3,
    offset: IVec3,
) -> Option<MovementCandidate> {
    mark_structure_translate(ctx, pos, pos + source, offset, MovementMark::Push).map(|movement| {
        MovementCandidate {
            primary: movement.with_source(pos),
            fallbacks: Vec::new(),
        }
    })
}

pub(super) fn collect_conveyor_candidate(
    ctx: &MovementMarkContext<'_>,
    pos: IVec3,
    source: IVec3,
    offset: IVec3,
) -> Option<MovementCandidate> {
    let target = pos + source;
    let primary = mark_structure_translate(ctx, pos, target, offset, MovementMark::Conveyor)?;
    if can_translate_structure(
        ctx.turn,
        primary.structure(),
        offset,
        ctx.turn_structures,
    ) {
        return Some(MovementCandidate {
            primary: primary.with_source(pos),
            fallbacks: Vec::new(),
        });
    }

    let structure = ctx.turn_structures.active_structure_at(pos, -offset)?;
    if !can_translate_structure(ctx.turn, &structure, -offset, ctx.turn_structures) {
        return None;
    }
    Some(MovementCandidate {
        primary: primary.with_source(pos),
        fallbacks: vec![StructureMove::translate_marked(structure, -offset, MovementMark::Conveyor)
            .with_source(pos)],
    })
}

pub(super) fn collect_lift_candidate(
    ctx: &MovementMarkContext<'_>,
    pos: IVec3,
    range: i32,
) -> Option<MovementCandidate> {
    let source = (1..=range)
        .map(|height| pos + IVec3::Y * height)
        .find(|candidate| {
            ctx.turn.is_material_at(*candidate)
                || ctx
                    .turn_structures
                    .active_structure_at(*candidate, IVec3::Y)
                    .is_some()
        })?;
    mark_structure_translate(ctx, pos, source, IVec3::Y, MovementMark::Vertical).map(|movement| {
        MovementCandidate {
            primary: movement.with_source(pos),
            fallbacks: Vec::new(),
        }
    })
}

pub(super) fn collect_rotate_candidate(
    structures: &StructureState,
    pos: IVec3,
    clockwise: bool,
) -> Option<MovementCandidate> {
    let source = pos + IVec3::Y;
    let structure = structures.pushable_structure_at(source, IVec3::ZERO)?;
    Some(MovementCandidate {
        primary: StructureMove::rotate(structure, pos, clockwise).with_source(pos),
        fallbacks: Vec::new(),
    })
}

fn mark_structure_translate(
    ctx: &MovementMarkContext<'_>,
    actor: IVec3,
    source: IVec3,
    offset: IVec3,
    mark: MovementMark,
) -> Option<StructureMove> {
    if ctx.turn.is_material_at(source) {
        return ctx
            .turn_structures
            .pushable_structure_at(source, offset)
            .map(|structure| StructureMove::translate_marked(structure, offset, mark));
    }

    let structure = if matches!(mark, MovementMark::Push)
        && ctx
            .turn
            .blocks
            .get(&actor)
            .is_some_and(|block| matches!(block.kind, BlockKind::Pusher | BlockKind::Blocker))
    {
        ctx.solution_structures.pusher_target_structure(
            ctx.solution,
            ctx.turn,
            actor,
            source,
            offset,
        )?
    } else {
        if ctx.turn_structures.structure_contains(source, actor) {
            return None;
        }
        ctx.turn_structures.active_structure_at(source, offset)?
    };
    Some(StructureMove::translate_marked(structure, offset, mark))
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
