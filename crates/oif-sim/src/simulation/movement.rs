use glam::IVec3;
use std::collections::{HashMap, HashSet};

use crate::blocks::{BlockId, MovementRule};
use crate::world::grid::WorldBlocks;

use super::motion::PusherMotion;
use super::structure_state::{StructureKind, StructureState};
use super::structures::{
    MovementMark, PusherActor, PusherAnimationKind, StructureMove, can_translate_structure,
};
use super::suction::SuctionLinks;

/// 活塞/拦截器工作面推/拉失败时是否反推自身（图预拆仍保留，仅跳过反推尝试）
pub const PUSHER_REVERSE_ENABLED: bool = true;

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

    /// 该格若为已伸出推杆的头，返回其本体坐标（头占两格中的工作面格）
    pub(super) fn body_at_extended_head(&self, world: &WorldBlocks, head: IVec3) -> Option<IVec3> {
        world.blocks.iter().find_map(|(pos, block)| {
            if !matches!(
                block.kind.movement_rule(block.facing),
                Some(MovementRule::PoweredTranslate { .. })
            ) {
                return None;
            }
            let extended = self
                .entries
                .get(&block.id)
                .is_some_and(|entry| entry.extended);
            if !extended {
                return None;
            }
            (*pos + block.facing.forward_ivec3() == head).then_some(*pos)
        })
    }

    /// 推动/收回执行成功后提交伸出状态
    pub(super) fn set_extended(&mut self, id: BlockId, extended: bool) {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.extended = extended;
        }
    }
}

pub(super) fn mark_structure_movement_phase(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    structures: &StructureState,
    pusher_state: &mut PusherState,
    suction: &SuctionLinks,
) -> Vec<StructureMove> {
    world.sync_rotator_arrivals();
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
                if let Some(movement) = mark_conveyor_movement(
                    world,
                    structures,
                    pusher_state,
                    pos,
                    source,
                    offset,
                    suction,
                ) {
                    if let Some(source_id) = source_id {
                        moves.push(movement.with_source(source_id, pos));
                    }
                }
            }
            MovementRule::Lift { range } => {
                // 通电关闭：本回合不打抬升标签
                if powered_devices.contains(&pos) {
                    continue;
                }
                // 与参考实现一致：range 内每个可动结构各自打抬升标签（叠层同拍一起抬）
                for movement in
                    mark_lift_structures(world, structures, pusher_state, pos, range, suction)
                {
                    if let Some(source_id) = source_id {
                        moves.push(movement.with_source(source_id, pos));
                    }
                }
            }
            MovementRule::Rotate { clockwise } => {
                if let Some(movement) = mark_rotate_structure(
                    world,
                    powered_devices,
                    structures,
                    pos,
                    clockwise,
                    suction,
                ) {
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
    pusher_state: &PusherState,
    pos: IVec3,
    source: IVec3,
    offset: IVec3,
    suction: &SuctionLinks,
) -> Option<StructureMove> {
    let heads = pusher_state.hard_head_occupancy(world);
    let target = pos + source;
    if let Some(movement) = mark_structure_translate(
        world,
        structures,
        pusher_state,
        pos,
        target,
        offset,
        MovementMark::Conveyor,
        suction,
    ) {
        if can_translate_structure(
            world,
            movement.structure(),
            offset,
            structures,
            suction,
            &heads,
        ) {
            return Some(movement);
        }
    } else if !world.is_occupied(target)
        && pusher_state.body_at_extended_head(world, target).is_none()
    {
        return None;
    }

    let structure = structures.linked_pushable_at(suction, pos, -offset)?;
    if !can_translate_structure(world, &structure, -offset, structures, suction, &heads) {
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
    let (current_extended, bound_front) = {
        let entry = pusher_state
            .entries
            .entry(id)
            .or_insert_with(|| PusherStateEntry {
                extended: false,
                bound_front: false,
            });
        (entry.extended, entry.bound_front)
    };
    if desired_extended == current_extended {
        return None;
    }

    let head = pos + source;
    let front_is_fragile = world.is_fragile_material_at(head);
    let animation = if desired_extended {
        PusherAnimationKind::Extend
    } else {
        PusherAnimationKind::Retract
    };
    // 伸出推 +offset，收回拉 -offset；失败时自身走反方向
    let attempt_offset = if desired_extended { offset } else { -offset };

    let work = if desired_extended {
        // 前方脆弱：压碎而非推动其所在结构
        if front_is_fragile {
            None
        } else {
            mark_structure_translate(
                world,
                structures,
                pusher_state,
                pos,
                pos + offset,
                offset,
                MovementMark::Push,
                suction,
            )
        }
    } else if bound_front {
        // 收回：仅开局已粘的才拉回头前一格的结构
        mark_structure_translate(
            world,
            structures,
            pusher_state,
            pos,
            pos + offset + offset,
            -offset,
            MovementMark::Push,
            suction,
        )
    } else {
        None
    };

    // 工作面标到结构：能走则推/拉对方；伸出失败才反推；收回失败只缩头
    if let Some(movement) = work {
        // 收回时头会让出，被拉块可进原头格——校验忽略本推杆头占位
        let mut heads_for_check = claimed_heads.clone();
        if !desired_extended {
            heads_for_check.remove(&head);
        }
        if can_translate_structure(
            world,
            movement.structure(),
            attempt_offset,
            structures,
            suction,
            &heads_for_check,
        ) {
            if !desired_extended {
                claimed_heads.remove(&head);
            }
            return Some(
                movement
                    .with_pusher_actor(id, pos, MovementMark::Push, animation)
                    .with_source(id, pos),
            );
        }
        if desired_extended {
            return mark_pusher_reverse_self(
                world,
                structures,
                suction,
                claimed_heads,
                pos,
                id,
                -attempt_offset,
                animation,
            );
        }
        // 收回拉不动：不反推，下面走零位移缩头
    }

    if desired_extended {
        // 空头伸出：脆弱格视为可压碎让出；实心占用或头格争用则反推自身
        if front_is_fragile {
            if !claimed_heads.insert(head) {
                return None;
            }
        } else if world.is_occupied(head) || !claimed_heads.insert(head) {
            return mark_pusher_reverse_self(
                world,
                structures,
                suction,
                claimed_heads,
                pos,
                id,
                -attempt_offset,
                animation,
            );
        }
    } else {
        // 收回（含粘头但拉不动 / 未粘头）：释放头占位，零位移缩头
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
            PusherActor { id, pos, animation },
            MovementMark::Push,
        )
        .with_source(id, pos),
    )
}

/// 活塞/拦截器工作面失败时：自身反向平移并完成伸出/收回
fn mark_pusher_reverse_self(
    world: &WorldBlocks,
    structures: &StructureState,
    suction: &SuctionLinks,
    hard_heads: &HashSet<IVec3>,
    pos: IVec3,
    id: BlockId,
    reverse: IVec3,
    animation: PusherAnimationKind,
) -> Option<StructureMove> {
    if !PUSHER_REVERSE_ENABLED {
        return None;
    }
    // 与正推对称：切断头前边，只带动活塞本体一侧；头前子结构留在原地
    let subset = structures.pusher_actor_structure(world, pos, reverse)?;
    let structure = structures.linked_expand_pusher_subset(suction, &subset, reverse)?;
    if !can_translate_structure(world, &structure, reverse, structures, suction, hard_heads) {
        return None;
    }
    Some(
        StructureMove::translate_by_pusher_actor(
            structures.id_at(pos)?,
            structure,
            reverse,
            PusherActor { id, pos, animation },
            MovementMark::Push,
        )
        .with_source(id, pos),
    )
}

trait StructureMoveActorExt {
    fn with_pusher_actor(
        self,
        actor_id: BlockId,
        actor: IVec3,
        mark: MovementMark,
        animation: PusherAnimationKind,
    ) -> StructureMove;
}

impl StructureMoveActorExt for StructureMove {
    fn with_pusher_actor(
        self,
        actor_id: BlockId,
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
                    id: actor_id,
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
    pusher_state: &PusherState,
    actor: IVec3,
    mut source: IVec3,
    offset: IVec3,
    mark: MovementMark,
    suction: &SuctionLinks,
) -> Option<StructureMove> {
    // 推到已伸出的头：视为推动该推杆整坨（头+体占两格）
    if structures.id_at(source).is_none() {
        source = pusher_state.body_at_extended_head(world, source)?;
    }
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

/// 抬升器 range 内每个可动结构各打一条抬升标签（不并成一条，避免只抬底层）
fn mark_lift_structures(
    world: &WorldBlocks,
    structures: &StructureState,
    pusher_state: &PusherState,
    pos: IVec3,
    range: i32,
    suction: &SuctionLinks,
) -> Vec<StructureMove> {
    let heads = pusher_state.hard_head_occupancy(world);
    let mut moves = Vec::new();
    let mut seen_ids = HashSet::new();
    for height in 1..=range {
        let candidate = pos + IVec3::Y * height;
        let seed = structures
            .id_at(candidate)
            .map(|_| candidate)
            .or_else(|| pusher_state.body_at_extended_head(world, candidate));
        let Some(seed) = seed else {
            continue;
        };
        let Some(id) = structures.id_at(seed) else {
            continue;
        };
        if !seen_ids.insert(id) {
            continue;
        }
        let eligible = world.is_material_at(seed)
            || structures
                .linked_pushable_at(suction, seed, IVec3::Y)
                .is_some();
        if !eligible {
            seen_ids.remove(&id);
            continue;
        }
        let Some(movement) = mark_structure_translate(
            world,
            structures,
            pusher_state,
            pos,
            seed,
            IVec3::Y,
            MovementMark::Vertical,
            suction,
        ) else {
            seen_ids.remove(&id);
            continue;
        };
        if !can_translate_structure(
            world,
            movement.structure(),
            IVec3::Y,
            structures,
            suction,
            &heads,
        ) {
            seen_ids.remove(&id);
            continue;
        }
        for member in movement.structure() {
            if let Some(member_id) = structures.id_at(*member) {
                seen_ids.insert(member_id);
            }
        }
        moves.push(movement);
    }
    moves
}

fn mark_rotate_structure(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    structures: &StructureState,
    pos: IVec3,
    clockwise: bool,
    suction: &SuctionLinks,
) -> Option<StructureMove> {
    // 通电清锁，同拍可再转工作面上同一块
    if powered_devices.contains(&pos) {
        world.rotator_arrivals.remove(&pos);
    }
    let source = pos + IVec3::Y;
    let block = world.blocks.get(&source)?;
    if !(block.kind.is_material() || block.kind.is_factory()) {
        return None;
    }
    if world.is_rotator_arrival(pos, block.id) {
        return None;
    }
    let structure_id = structures.id_at(source)?;
    // 材料被吸盘粘到工厂时不转；纯工厂结构可以转
    if structures.kind_at(source) == Some(StructureKind::Material)
        && structures.linked_contains_factory(suction, source)
    {
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
