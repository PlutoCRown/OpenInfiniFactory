use glam::IVec3;
use std::collections::{HashMap, HashSet};

use crate::blocks::{BlockId, MovementRule};
use crate::world::grid::WorldBlocks;

use super::motion::PusherMotion;
use super::structure_state::StructureState;
use super::structures::{
    MovementMark, PusherActor, PusherAnimationKind, StructureMove, can_translate_structure,
};
use super::suction::SuctionLinks;

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
    suction: &SuctionLinks,
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
                    mark_conveyor_movement(world, structures, pos, source, offset, suction)
                {
                    if let Some(source_id) = source_id {
                        moves.push(movement.with_source(source_id, pos));
                    }
                }
            }
            MovementRule::Lift { range } => {
                if let Some(movement) = mark_lift_structure(world, structures, pos, range, suction)
                {
                    if let Some(source_id) = source_id {
                        moves.push(movement.with_source(source_id, pos));
                    }
                }
            }
            MovementRule::Rotate { clockwise } => {
                if let Some(movement) =
                    mark_rotate_material_structure(structures, pos, clockwise, suction)
                {
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
                    suction,
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
    suction: &SuctionLinks,
) -> Option<StructureMove> {
    let target = pos + source;
    if let Some(movement) = mark_structure_translate(
        world,
        structures,
        pos,
        target,
        offset,
        MovementMark::Conveyor,
        suction,
    ) {
        if can_translate_structure(world, movement.structure(), offset, structures, suction) {
            return Some(movement);
        }
    } else if !world.is_occupied(target) {
        return None;
    }

    let structure = structures.linked_pushable_at(suction, pos, -offset)?;
    if !can_translate_structure(world, &structure, -offset, structures, suction) {
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
    suction: &SuctionLinks,
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
    let front_is_fragile = world.is_fragile_material_at(head);
    let movement = if desired_extended {
        // 前方脆弱：压碎而非推动其所在结构
        if front_is_fragile {
            None
        } else {
            mark_structure_translate(
                world,
                structures,
                pos,
                pos + source,
                offset,
                MovementMark::Push,
                suction,
            )
        }
    } else if entry.bound_front {
        // 收回：仅开局已粘的才拉回头前一格的结构
        mark_structure_translate(
            world,
            structures,
            pos,
            pos + source + offset,
            -offset,
            MovementMark::Push,
            suction,
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
        // 空头伸出：脆弱格视为可压碎让出；实心占用或头格争用则失败
        if front_is_fragile {
            if !claimed_heads.insert(head) {
                return None;
            }
        } else if world.is_occupied(head) || !claimed_heads.insert(head) {
            return None;
        }
    } else if entry.bound_front {
        // 粘着方块却推不动：保持伸出
        return None;
    } else {
        claimed_heads.remove(&head);
    }

    // 空头伸出/收回：对本结构发 Push 零位移标签，执行时优先于重力并抑制自身下落
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
                mut actors,
                source,
                source_pos,
                ..
            } => {
                actors.push(PusherActor {
                    pos: actor,
                    animation,
                });
                StructureMove::Translate {
                    structure_id,
                    structure,
                    offset,
                    actors,
                    mark,
                    source: None,
                    source_pos: None,
                }
                .with_optional_source(source, source_pos)
            }
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
    suction: &SuctionLinks,
) -> Option<StructureMove> {
    if world.is_material_at(source) {
        let structure_id = structures.id_at(source)?;
        return structures
            .linked_pushable_at(suction, source, offset)
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
        }) {
        // 活塞子集后再经吸盘扩展（子集不膨胀为整结构）
        let subset = structures.pusher_target_structure(world, actor, source, offset)?;
        structures.linked_expand_pusher_subset(suction, &subset, offset)?
    } else {
        if structures.structure_contains(source, actor) {
            return None;
        }
        structures.linked_pushable_at(suction, source, offset)?
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
    suction: &SuctionLinks,
) -> Option<StructureMove> {
    let source = (1..=range)
        .map(|height| pos + IVec3::Y * height)
        .find(|candidate| {
            world.is_material_at(*candidate)
                || structures
                    .linked_pushable_at(suction, *candidate, IVec3::Y)
                    .is_some()
        })?;

    mark_structure_translate(
        world,
        structures,
        pos,
        source,
        IVec3::Y,
        MovementMark::Vertical,
        suction,
    )
}

fn mark_rotate_material_structure(
    structures: &StructureState,
    pos: IVec3,
    clockwise: bool,
    suction: &SuctionLinks,
) -> Option<StructureMove> {
    let source = pos + IVec3::Y;
    let structure_id = structures.id_at(source)?;
    // 粘连组含工厂时不允许旋转
    if structures.linked_contains_factory(suction, source) {
        return None;
    }
    let structure = structures.linked_pushable_at(suction, source, IVec3::ZERO)?;
    Some(StructureMove::rotate(
        structure_id,
        structure,
        pos,
        clockwise,
    ))
}
