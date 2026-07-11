use glam::IVec3;
use std::collections::{HashMap, HashSet};

use crate::blocks::{BlockId, MovementRule};
use crate::world::grid::WorldBlocks;

use super::motion::PusherMotion;
use super::structure_state::StructureState;
use super::structures::{
    can_translate_structure, MovementMark, PusherActor, PusherAnimationKind, StructureMove,
};

/// 活塞/拦截器伸出状态，按方块运行时 ID 索引（随实体移动，不跟格子走）
#[derive(Default, Clone)]
pub struct PusherState {
    entries: HashMap<BlockId, PusherStateEntry>,
}

#[derive(Clone, Copy)]
struct PusherStateEntry {
    extended: bool,
    /// 开局快照时头前是否已有工厂方块；运行时掉到面前的不粘
    bound_front: bool,
}

impl PusherState {
    pub fn rebuild_from_world(world: &WorldBlocks) -> Self {
        let entries = world
            .blocks
            .iter()
            .filter_map(|(pos, block)| {
                matches!(
                    block.kind.movement_rule(block.facing),
                    Some(MovementRule::PoweredTranslate { .. })
                )
                .then_some((
                    block.id,
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

    pub fn sustained_animations(&self, world: &WorldBlocks) -> HashMap<IVec3, PusherMotion> {
        world
            .blocks
            .iter()
            .filter_map(|(pos, block)| {
                self.entries
                    .get(&block.id)
                    .filter(|entry| entry.extended)
                    .map(|_| {
                        (
                            *pos,
                            PusherMotion {
                                from_extension: 1.0,
                                to_extension: 1.0,
                            },
                        )
                    })
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
                let Some(MovementRule::PoweredTranslate {
                    extend_when_powered,
                    ..
                }) = block.kind.movement_rule(block.facing)
                else {
                    return None;
                };
                let powered = powered_devices.contains(pos);
                let desired_extended = if extend_when_powered {
                    powered
                } else {
                    !powered
                };
                let current_extended = self
                    .entries
                    .get(&block.id)
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
        world
            .blocks
            .iter()
            .filter_map(|(pos, block)| {
                if !matches!(
                    block.kind.movement_rule(block.facing),
                    Some(MovementRule::PoweredTranslate { .. })
                ) {
                    return None;
                }
                self.entries
                    .get(&block.id)
                    .filter(|entry| entry.extended)
                    .map(|_| *pos + block.facing.forward_ivec3())
            })
            .collect()
    }

    /// 推动/收回执行成功后提交伸出状态
    pub(super) fn set_extended(&mut self, id: BlockId, extended: bool) {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.extended = extended;
        }
    }
}

pub(super) fn mark_structure_movement_phase(
    world: &WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    structures: &StructureState,
    pusher_state: &mut PusherState,
) -> Vec<StructureMove> {
    let mut movers: Vec<(IVec3, MovementRule)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            block
                .kind
                .movement_rule(block.facing)
                .map(|mover| (*pos, mover))
        })
        .collect();
    // 空头争夺同一格时按坐标稳定决出胜者
    movers.sort_by_key(|(pos, _)| (pos.x, pos.y, pos.z));
    let mut moves = Vec::new();
    let mut claimed_heads = pusher_state.hard_head_occupancy(world);

    for (pos, mover) in movers {
        let source_id = world.blocks.get(&pos).map(|block| block.id);
        match mover {
            MovementRule::Translate { source, offset } => {
                if let Some(movement) =
                    mark_conveyor_movement(world, structures, pos, source, offset)
                {
                    if let Some(source_id) = source_id {
                        moves.push(movement.with_source(source_id, pos));
                    }
                }
            }
            MovementRule::Lift { range } => {
                if let Some(movement) = mark_lift_structure(world, structures, pos, range) {
                    if let Some(source_id) = source_id {
                        moves.push(movement.with_source(source_id, pos));
                    }
                }
            }
            MovementRule::Rotate { clockwise } => {
                if let Some(movement) = mark_rotate_material_structure(structures, pos, clockwise) {
                    if let Some(source_id) = source_id {
                        moves.push(movement.with_source(source_id, pos));
                    }
                }
            }
            MovementRule::PoweredTranslate {
                source,
                offset,
                extend_when_powered,
            } => {
                let powered = powered_devices.contains(&pos);
                let desired_extended = if extend_when_powered {
                    powered
                } else {
                    !powered
                };
                if let Some(movement) = mark_pusher_movement(
                    world,
                    structures,
                    pusher_state,
                    pos,
                    source,
                    offset,
                    desired_extended,
                    &mut claimed_heads,
                ) {
                    moves.push(movement);
                }
            }
        }
    }
    moves
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
        structures.id_at(pos)?,
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
    claimed_heads: &mut HashSet<IVec3>,
) -> Option<StructureMove> {
    let id = world.blocks.get(&pos)?.id;
    // 粘头只在开局 rebuild 写入；运行时新建条目视为不粘（不应靠当面有块临时粘上）
    let entry = pusher_state
        .entries
        .entry(id)
        .or_insert_with(|| PusherStateEntry {
            extended: false,
            bound_front: false,
        });
    if desired_extended == entry.extended {
        return None;
    }

    let head = pos + source;
    let movement = if desired_extended {
        // 伸出：面前有结构就推（与是否粘头无关）
        mark_structure_translate(
            world,
            structures,
            pos,
            pos + source,
            offset,
            MovementMark::Push,
        )
    } else if entry.bound_front {
        // 收回：仅开局已粘的才拉回头前一格的结构
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

    let animation = if desired_extended {
        PusherAnimationKind::Extend
    } else {
        PusherAnimationKind::Retract
    };

    // 粘了方块的伸出/收回：状态等执行成功后再提交
    if let Some(movement) = movement {
        return Some(
            movement
                .with_pusher_actor(pos, MovementMark::Push, animation)
                .with_source(id, pos),
        );
    }

    if desired_extended {
        // 空头伸出：格被占或已被其他头占用则失败
        if world.is_occupied(head) || !claimed_heads.insert(head) {
            return None;
        }
    } else if entry.bound_front {
        // 粘着方块却推不动：保持伸出
        return None;
    } else {
        claimed_heads.remove(&head);
    }

    // 空头伸出/收回：对本结构发 Push 零位移标签，执行时优先于重力并抑制自身下落 fallback
    let structure_id = structures.id_at(pos)?;
    let structure = structures.structure_positions(structure_id)?.clone();
    Some(
        StructureMove::translate_by_pusher_actor(
            structure_id,
            structure,
            IVec3::ZERO,
            PusherActor { pos, animation },
            MovementMark::Push,
        )
        .with_source(id, pos),
    )
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
                structure_id,
                structure,
                offset,
                source,
                source_pos,
                ..
            } => StructureMove::translate_by_pusher_actor(
                structure_id,
                structure,
                offset,
                PusherActor {
                    pos: actor,
                    animation,
                },
                mark,
            )
            .with_optional_source(source, source_pos),
            movement => movement,
        }
    }
}

trait StructureMoveSourceExt {
    fn with_optional_source(
        self,
        source: Option<crate::blocks::BlockId>,
        source_pos: Option<IVec3>,
    ) -> StructureMove;
}

impl StructureMoveSourceExt for StructureMove {
    fn with_optional_source(
        self,
        source: Option<crate::blocks::BlockId>,
        source_pos: Option<IVec3>,
    ) -> StructureMove {
        match (source, source_pos) {
            (Some(source), Some(source_pos)) => self.with_source(source, source_pos),
            _ => self,
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
        let structure_id = structures.id_at(source)?;
        return structures
            .pushable_structure_at(source, offset)
            .map(|structure| {
                StructureMove::translate_marked(structure_id, structure, offset, mark)
            });
    }

    let structure_id = structures.id_at(source)?;
    let structure = if matches!(mark, MovementMark::Push)
        && world.blocks.get(&actor).is_some_and(|block| {
            matches!(
                block.kind.movement_rule(block.facing),
                Some(MovementRule::PoweredTranslate { .. })
            )
        })
    {
        structures.pusher_target_structure(world, actor, source, offset)?
    } else {
        if structures.structure_contains(source, actor) {
            return None;
        }
        structures.active_structure_at(source, offset)?
    };
    Some(StructureMove::translate_marked(
        structure_id,
        structure,
        offset,
        mark,
    ))
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
    let structure_id = structures.id_at(source)?;
    let structure = structures.pushable_structure_at(source, IVec3::ZERO)?;
    Some(StructureMove::rotate(
        structure_id,
        structure,
        pos,
        clockwise,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{BlockData, BlockKind};
    use crate::simulation::structures::{
        execute_structure_moves_with_pushers, merge_structure_movement_plan, MovementInfluenceCache,
    };
    use crate::world::direction::Facing;

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
        let device_moves =
            mark_structure_movement_phase(&world, &powered, &structures, &mut pusher_state);
        assert_eq!(device_moves.len(), 1, "only one bare extend may be marked");
        assert!(pusher_state.entries.values().all(|e| !e.extended));

        let mut cache = MovementInfluenceCache::default();
        let plan =
            merge_structure_movement_plan(vec![], device_moves, &mut cache, &structures, &world);
        let heads = HashSet::new();
        let (_anims, pusher_anims) = execute_structure_moves_with_pushers(
            &mut world,
            plan,
            &mut structures,
            &mut cache,
            &heads,
        );
        for (pos, animation) in &pusher_anims {
            if let Some(block) = world.blocks.get(pos) {
                pusher_state.set_extended(block.id, animation.to_extension > 0.5);
            }
        }

        let heads = pusher_state.hard_head_occupancy(&world);
        assert_eq!(heads.len(), 1, "only one head may occupy the shared cell");
        assert!(heads.contains(&IVec3::new(1, 1, 0)));
        // 坐标更小的西侧先处理，应胜出
        let west_id = world.blocks.get(&west).unwrap().id;
        let east_id = world.blocks.get(&east).unwrap().id;
        assert!(pusher_state
            .entries
            .get(&west_id)
            .is_some_and(|e| e.extended));
        assert!(pusher_state
            .entries
            .get(&east_id)
            .is_some_and(|e| !e.extended));
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
        let device_moves =
            mark_structure_movement_phase(&world, &powered, &structures, &mut pusher_state);
        assert_eq!(device_moves.len(), 2);
        // 标记阶段尚未提交伸出
        assert!(pusher_state.entries.values().all(|e| !e.extended));

        let mut cache = MovementInfluenceCache::default();
        let plan =
            merge_structure_movement_plan(vec![], device_moves, &mut cache, &structures, &world);
        let heads = HashSet::new();
        let (_anims, pusher_anims) = execute_structure_moves_with_pushers(
            &mut world,
            plan,
            &mut structures,
            &mut cache,
            &heads,
        );
        for (pos, animation) in &pusher_anims {
            if let Some(block) = world.blocks.get(pos) {
                pusher_state.set_extended(block.id, animation.to_extension > 0.5);
            }
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
        let moves = mark_structure_movement_phase(&world, &powered, &structures, &mut pusher_state);

        assert!(moves.is_empty());
        let blocker_id = world.blocks.get(&blocker).unwrap().id;
        assert!(pusher_state
            .entries
            .get(&blocker_id)
            .is_some_and(|e| !e.extended));
    }

    fn run_pusher_phase(
        world: &mut WorldBlocks,
        structures: &mut StructureState,
        pusher_state: &mut PusherState,
        powered: &HashSet<IVec3>,
    ) {
        let device_moves = mark_structure_movement_phase(world, powered, structures, pusher_state);
        if device_moves.is_empty() {
            return;
        }
        let mut cache = MovementInfluenceCache::default();
        let heads = pusher_state.hard_head_occupancy(world);
        let plan =
            merge_structure_movement_plan(vec![], device_moves, &mut cache, structures, world);
        let (_anims, pusher_anims) =
            execute_structure_moves_with_pushers(world, plan, structures, &mut cache, &heads);
        for (pos, animation) in &pusher_anims {
            if let Some(block) = world.blocks.get(pos) {
                pusher_state.set_extended(block.id, animation.to_extension > 0.5);
            }
        }
    }

    #[test]
    fn initial_front_factory_sticks_on_retract() {
        let mut world = WorldBlocks::default();
        place(
            &mut world,
            IVec3::new(0, 0, 0),
            BlockKind::Stone,
            Facing::North,
        );
        let pusher = IVec3::new(0, 1, 0);
        place(&mut world, pusher, BlockKind::Pusher, Facing::East);
        place(
            &mut world,
            IVec3::new(1, 1, 0),
            BlockKind::Platform,
            Facing::North,
        );

        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher_state = PusherState::rebuild_from_world(&world);
        let pusher_id = world.blocks.get(&pusher).unwrap().id;
        assert!(pusher_state
            .entries
            .get(&pusher_id)
            .is_some_and(|e| e.bound_front));

        run_pusher_phase(
            &mut world,
            &mut structures,
            &mut pusher_state,
            &HashSet::from([pusher]),
        );
        assert!(world.is_factory_at(IVec3::new(2, 1, 0)));

        run_pusher_phase(
            &mut world,
            &mut structures,
            &mut pusher_state,
            &HashSet::new(),
        );
        assert!(
            world.is_factory_at(IVec3::new(1, 1, 0)),
            "开局已粘：收回应拉回平台"
        );
        assert!(!world.is_factory_at(IVec3::new(2, 1, 0)));
    }

    #[test]
    fn runtime_arrived_front_does_not_stick_on_retract() {
        let mut world = WorldBlocks::default();
        place(
            &mut world,
            IVec3::new(0, 0, 0),
            BlockKind::Stone,
            Facing::North,
        );
        let pusher = IVec3::new(0, 1, 0);
        place(&mut world, pusher, BlockKind::Pusher, Facing::East);

        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher_state = PusherState::rebuild_from_world(&world);
        let pusher_id = world.blocks.get(&pusher).unwrap().id;
        assert!(pusher_state
            .entries
            .get(&pusher_id)
            .is_some_and(|e| !e.bound_front));

        // 模拟运行中掉到面前：只重建结构，不重建粘头快照
        place(
            &mut world,
            IVec3::new(1, 1, 0),
            BlockKind::Platform,
            Facing::North,
        );
        structures.rebuild_for_simulation(&world);

        run_pusher_phase(
            &mut world,
            &mut structures,
            &mut pusher_state,
            &HashSet::from([pusher]),
        );
        assert!(world.is_factory_at(IVec3::new(2, 1, 0)));

        run_pusher_phase(
            &mut world,
            &mut structures,
            &mut pusher_state,
            &HashSet::new(),
        );
        assert!(
            world.is_factory_at(IVec3::new(2, 1, 0)),
            "开局不粘：收回不应拉回运行中到达的平台"
        );
        assert!(!world.is_factory_at(IVec3::new(1, 1, 0)));
    }
}
