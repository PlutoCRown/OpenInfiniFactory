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
                        duration: None,
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

    /// 推动/收回执行成功后提交伸出状态
    pub(super) fn set_extended(&mut self, pos: IVec3, extended: bool) {
        if let Some(entry) = self.entries.get_mut(&pos) {
            entry.extended = extended;
        }
    }
}

pub(super) fn mark_structure_movement_phase(
    world: &WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    structures: &StructureState,
    pusher_state: &mut PusherState,
) -> (Vec<StructureMove>, HashMap<IVec3, PusherAnimation>) {
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
    // 空头争夺同一格时按坐标稳定决出胜者
    movers.sort_by_key(|(pos, _, _)| (pos.x, pos.y, pos.z));
    let mut moves = Vec::new();
    let mut bare_pusher_animations = HashMap::new();
    let mut claimed_heads = pusher_state.hard_head_occupancy(world);

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
                        &mut claimed_heads,
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
    claimed_heads: &mut HashSet<IVec3>,
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

    let head = pos + source;
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

    // 粘了方块的伸出/收回：状态等执行成功后再提交，避免推失败却伸出
    if let Some(movement) = movement {
        return Some(
            movement
                .with_pusher_actor(pos, MovementMark::Push, animation)
                .with_source(pos),
        );
    }

    if desired_extended {
        // 空头伸出：格被占或已被其他头占用则失败
        if world.is_occupied(head) || !claimed_heads.insert(head) {
            return None;
        }
        entry.extended = true;
    } else if entry.bound_front {
        // 粘着方块却推不动：保持伸出
        return None;
    } else {
        claimed_heads.remove(&head);
        entry.extended = false;
    }

    bare_pusher_animations.insert(
        pos,
        PusherAnimation {
            duration: None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::blocks::{BlockData, BlockKind};
    use crate::game::simulation::structures::{
        execute_structure_moves_with_pushers, merge_structure_movement_plan, MovementInfluenceCache,
    };
    use crate::game::world::direction::Facing;

    fn place(world: &mut WorldBlocks, pos: IVec3, kind: BlockKind, facing: Facing) {
        world.insert(pos, BlockData::new(kind, facing));
    }

    #[test]
    fn bare_head_to_head_only_one_extends() {
        let mut world = WorldBlocks::default();
        // 中间空格，两侧阻拦器头对头
        place(
            &mut world,
            IVec3::new(0, 0, 0),
            BlockKind::Stone,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(2, 0, 0),
            BlockKind::Stone,
            Facing::North,
        );
        let west = IVec3::new(0, 1, 0);
        let east = IVec3::new(2, 1, 0);
        place(&mut world, west, BlockKind::Blocker, Facing::East);
        place(&mut world, east, BlockKind::Blocker, Facing::West);

        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher_state = PusherState::rebuild_from_world(&world);
        let powered = HashSet::new();
        let (_moves, bare) =
            mark_structure_movement_phase(&world, &powered, &structures, &mut pusher_state);

        let heads = pusher_state.hard_head_occupancy(&world);
        assert_eq!(heads.len(), 1, "only one head may occupy the shared cell");
        assert!(heads.contains(&IVec3::new(1, 1, 0)));
        assert_eq!(bare.len(), 1);
        // 坐标更小的西侧先处理，应胜出
        assert!(pusher_state.entries.get(&west).is_some_and(|e| e.extended));
        assert!(pusher_state.entries.get(&east).is_some_and(|e| !e.extended));
    }

    #[test]
    fn competing_bound_pushes_only_winner_extends() {
        let mut world = WorldBlocks::default();
        // 两侧立柱锚定阻拦器；被推平台悬空，不贴场景
        place(
            &mut world,
            IVec3::new(0, 0, 0),
            BlockKind::Stone,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(0, 1, 0),
            BlockKind::Platform,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(4, 0, 0),
            BlockKind::Stone,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(4, 1, 0),
            BlockKind::Platform,
            Facing::North,
        );
        let west = IVec3::new(0, 2, 0);
        let east = IVec3::new(4, 2, 0);
        place(&mut world, west, BlockKind::Blocker, Facing::East);
        place(
            &mut world,
            IVec3::new(1, 2, 0),
            BlockKind::Platform,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(3, 2, 0),
            BlockKind::Platform,
            Facing::North,
        );
        place(&mut world, east, BlockKind::Blocker, Facing::West);

        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher_state = PusherState::rebuild_from_world(&world);
        let powered = HashSet::new();
        let (device_moves, bare) =
            mark_structure_movement_phase(&world, &powered, &structures, &mut pusher_state);
        assert!(bare.is_empty());
        assert_eq!(device_moves.len(), 2);
        // 标记阶段尚未提交伸出
        assert!(pusher_state.entries.values().all(|e| !e.extended));

        let mut cache = MovementInfluenceCache::default();
        let plan = merge_structure_movement_plan(vec![], device_moves, &mut cache);
        let (_anims, pusher_anims) =
            execute_structure_moves_with_pushers(&mut world, plan, &mut structures, &mut cache);
        for (pos, animation) in &pusher_anims {
            pusher_state.set_extended(*pos, animation.to_extension > 0.5);
        }

        assert_eq!(pusher_anims.len(), 1, "only one push may succeed");
        let extended_count = pusher_state.entries.values().filter(|e| e.extended).count();
        assert_eq!(extended_count, 1);
        assert!(world.is_factory_at(IVec3::new(2, 2, 0)));
        let platforms = [1, 2, 3]
            .iter()
            .filter(|&&x| world.is_factory_at(IVec3::new(x, 2, 0)))
            .count();
        assert_eq!(platforms, 2);
    }

    #[test]
    fn bare_extend_blocked_by_occupied_cell() {
        let mut world = WorldBlocks::default();
        place(
            &mut world,
            IVec3::new(0, 0, 0),
            BlockKind::Stone,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(1, 0, 0),
            BlockKind::Stone,
            Facing::North,
        );
        let blocker = IVec3::new(0, 1, 0);
        place(&mut world, blocker, BlockKind::Blocker, Facing::East);
        // 头前是场景石块，不可推动 → 空头路径也应拒绝伸出
        place(
            &mut world,
            IVec3::new(1, 1, 0),
            BlockKind::Stone,
            Facing::North,
        );

        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher_state = PusherState::rebuild_from_world(&world);
        let powered = HashSet::new();
        let (_moves, bare) =
            mark_structure_movement_phase(&world, &powered, &structures, &mut pusher_state);

        assert!(bare.is_empty());
        assert!(pusher_state
            .entries
            .get(&blocker)
            .is_some_and(|e| !e.extended));
    }
}
