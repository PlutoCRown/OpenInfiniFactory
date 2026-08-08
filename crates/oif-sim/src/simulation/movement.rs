use glam::IVec3;
use std::collections::{HashMap, HashSet};

use crate::blocks::{BlockData, BlockId, BlockKind, MovementRule};
use crate::world::grid::WorldBlocks;

use super::motion::PusherMotion;
use super::structure_state::{StructureId, StructureKind, StructureState};
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
                .then_some({
                    let head = *pos + block.facing.forward_ivec3();
                    (
                        block.id,
                        PusherStateEntry {
                            extended: world
                                .blocks
                                .get(&head)
                                .is_some_and(|b| b.kind == BlockKind::PusherHead),
                            bound_front: world.is_factory_at(head),
                        },
                    )
                })
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

    /// 已伸出头占格：世界里的真实 PusherHead 方块
    pub(super) fn hard_head_occupancy(world: &WorldBlocks) -> HashSet<IVec3> {
        world
            .blocks
            .iter()
            .filter(|(_, block)| block.kind == BlockKind::PusherHead)
            .map(|(pos, _)| *pos)
            .collect()
    }

    /// 该格若为已伸出推杆头，返回其本体坐标
    pub(super) fn body_at_extended_head(world: &WorldBlocks, head: IVec3) -> Option<IVec3> {
        let head_block = world.blocks.get(&head)?;
        if head_block.kind != BlockKind::PusherHead {
            return None;
        }
        let body_pos = head - head_block.facing.forward_ivec3();
        let body = world.blocks.get(&body_pos)?;
        matches!(
            body.kind.movement_rule(body.facing),
            Some(MovementRule::PoweredTranslate { .. })
        )
        .then_some(body_pos)
    }

    /// 推动/收回执行成功后提交伸出状态，并同步真实头方块
    pub(super) fn set_extended(&mut self, world: &mut WorldBlocks, id: BlockId, extended: bool) {
        let Some(entry) = self.entries.get_mut(&id) else {
            return;
        };
        if entry.extended == extended {
            return;
        }
        entry.extended = extended;
        let Some((pos, facing)) = world
            .blocks
            .iter()
            .find(|(_, block)| block.id == id)
            .map(|(pos, block)| (*pos, block.facing))
        else {
            return;
        };
        let head = pos + facing.forward_ivec3();
        if extended {
            if !world.blocks.contains_key(&head) {
                let _ = world.insert(head, BlockData::new(BlockKind::PusherHead, facing));
            }
        } else if world
            .blocks
            .get(&head)
            .is_some_and(|block| block.kind == BlockKind::PusherHead)
        {
            let _ = world.remove(&head);
        }
    }
}

pub(super) fn mark_structure_movement_phase(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    structures: &mut StructureState,
    pusher_state: &mut PusherState,
    suction: &SuctionLinks,
) -> Vec<StructureMove> {
    world.sync_rotator_arrivals();
    structures.clear_turn_marks();
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
    // 按 BlockId 稳定裁决（与 held 冲突规则一致）
    movers.sort_by_key(|(pos, _)| {
        world
            .blocks
            .get(pos)
            .map(|block| block.id.0)
            .unwrap_or(u64::MAX)
    });
    let mut moves = Vec::new();
    let mut claimed_heads = PusherState::hard_head_occupancy(world);

    // 本回合要切换伸出状态的推杆（排序继承 movers）
    let mut actuating: Vec<(IVec3, IVec3, IVec3, bool)> = Vec::new();
    for (pos, mover) in &movers {
        let MovementRule::PoweredTranslate {
            source,
            offset,
            extend_when_powered,
        } = mover
        else {
            continue;
        };
        let powered = powered_devices.contains(pos);
        let desired_extended = if *extend_when_powered {
            powered
        } else {
            !powered
        };
        let current_extended = world
            .blocks
            .get(pos)
            .and_then(|block| pusher_state.entries.get(&block.id))
            .map(|entry| entry.extended)
            .unwrap_or(false);
        if desired_extended != current_extended {
            actuating.push((*pos, *source, *offset, desired_extended));
        }
    }
    let mut motion_held: HashSet<IVec3> = HashSet::new();
    let mut motion_tags: HashMap<IVec3, IVec3> = HashMap::new();
    let mut succeeded_deform: HashSet<(StructureId, u32)> = HashSet::new();
    // 本回合欲伸出 / 欲收回的推杆体（共轴组校验用）
    let mut actuating_extend: HashSet<BlockId> = HashSet::new();
    let mut actuating_retract: HashSet<BlockId> = HashSet::new();
    for (pos, _, _, desired_extended) in &actuating {
        let Some(id) = world.blocks.get(pos).map(|b| b.id) else {
            continue;
        };
        if *desired_extended {
            actuating_extend.insert(id);
        } else {
            actuating_retract.insert(id);
        }
    }

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
                if powered_devices.contains(&pos) {
                    continue;
                }
                for movement in mark_lift_structures(world, structures, pos, range, suction) {
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
            MovementRule::PoweredTranslate { .. } => {}
        }
    }

    for (pos, source, offset, desired_extended) in actuating {
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
            &mut motion_held,
            &mut motion_tags,
            &mut succeeded_deform,
            &actuating_extend,
            &actuating_retract,
        ) {
            let merged = match &movement {
                StructureMove::Translate {
                    offset: move_off,
                    actors,
                    ..
                } if *move_off == IVec3::ZERO && actors.len() == 1 => {
                    let actor = actors[0];
                    let mut found = false;
                    for prior in moves.iter_mut().rev() {
                        if let StructureMove::Translate {
                            structure,
                            offset: prior_off,
                            actors: prior_actors,
                            ..
                        } = prior
                        {
                            if *prior_off != IVec3::ZERO && structure.contains(&actor.pos) {
                                prior_actors.push(actor);
                                found = true;
                                break;
                            }
                        }
                    }
                    found
                }
                _ => false,
            };
            if !merged {
                moves.push(movement);
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
    let heads = PusherState::hard_head_occupancy(world);
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
        && PusherState::body_at_extended_head(world, target).is_none()
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
    structures: &mut StructureState,
    pusher_state: &mut PusherState,
    pos: IVec3,
    source: IVec3,
    offset: IVec3,
    desired_extended: bool,
    claimed_heads: &mut HashSet<IVec3>,
    suction: &SuctionLinks,
    motion_held: &mut HashSet<IVec3>,
    motion_tags: &mut HashMap<IVec3, IVec3>,
    succeeded_deform: &mut HashSet<(StructureId, u32)>,
    actuating_extend: &HashSet<BlockId>,
    actuating_retract: &HashSet<BlockId>,
) -> Option<StructureMove> {
    let id = world.blocks.get(&pos)?.id;
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
    let animation = if desired_extended {
        PusherAnimationKind::Extend
    } else {
        PusherAnimationKind::Retract
    };

    let head = pos + source;
    let structure_id = structures.id_at(pos)?;

    // 体已 held：不可再发动其它 deform；仅当某动作组已成功时挂共轴动画
    if structures.held_blocks.contains(&id) || motion_held.contains(&pos) {
        if desired_extended {
            for forward in [true, false] {
                let Some((_, indices)) = structures.deform_action_groups(world, pos, forward)
                else {
                    continue;
                };
                if indices
                    .iter()
                    .any(|idx| succeeded_deform.contains(&(structure_id, *idx)))
                {
                    return try_deform_action(
                        world,
                        structures,
                        suction,
                        claimed_heads,
                        pos,
                        id,
                        structure_id,
                        forward,
                        if forward { offset } else { -offset },
                        animation,
                        forward,
                        motion_held,
                        motion_tags,
                        succeeded_deform,
                        actuating_extend,
                        actuating_retract,
                    );
                }
            }
        }
        return None;
    }

    if desired_extended {
        if let Some(movement) = try_deform_action(
            world,
            structures,
            suction,
            claimed_heads,
            pos,
            id,
            structure_id,
            true,
            offset,
            animation,
            true,
            motion_held,
            motion_tags,
            succeeded_deform,
            actuating_extend,
            actuating_retract,
        ) {
            return Some(movement);
        }
        // 正推无实体格且头前是外结构：整坨外推
        let forward_physical_empty =
            structures
                .deform_action(world, pos, true)
                .is_some_and(|(seed, _, nodes)| {
                    structures.nodes_to_positions(world, seed, nodes).is_empty()
                });
        if forward_physical_empty && !world.is_fragile_material_at(head) {
            if let Some(front_id) = world.blocks.get(&head).map(|b| b.id) {
                let external = structures
                    .id_at(head)
                    .is_some_and(|sid| sid != structure_id)
                    || PusherState::body_at_extended_head(world, head)
                        .is_some_and(|body| structures.id_at(body) != Some(structure_id));
                if external && !structures.held_blocks.contains(&front_id) {
                    let cargo_pos = PusherState::body_at_extended_head(world, head).unwrap_or(head);
                    if let Some(movement) = mark_structure_translate(
                        world,
                        structures,
                        pos,
                        cargo_pos,
                        offset,
                        MovementMark::Push,
                        suction,
                    ) {
                        let mut heads = claimed_heads.clone();
                        heads.remove(&head);
                        if can_translate_structure(
                            world,
                            movement.structure(),
                            offset,
                            structures,
                            suction,
                            &heads,
                        ) {
                            claimed_heads.insert(head);
                            apply_motion_tags(
                                movement.structure(),
                                offset,
                                motion_held,
                                motion_tags,
                            );
                            for &p in movement.structure() {
                                if let Some(b) = world.blocks.get(&p) {
                                    structures.held_blocks.insert(b.id);
                                }
                            }
                            if let Some((_, _, nodes)) = structures.deform_action(world, pos, true)
                            {
                                let node_ids: Vec<_> = nodes.to_vec();
                                structures.held_blocks.extend(node_ids);
                            }
                            return Some(
                                movement
                                    .with_pusher_actor(id, pos, MovementMark::Push, animation)
                                    .with_source(id, pos),
                            );
                        }
                    }
                }
            }
        }
        if PUSHER_REVERSE_ENABLED {
            return try_deform_action(
                world,
                structures,
                suction,
                claimed_heads,
                pos,
                id,
                structure_id,
                false,
                -offset,
                animation,
                false,
                motion_held,
                motion_tags,
                succeeded_deform,
                actuating_extend,
                actuating_retract,
            );
        }
        return None;
    }

    // 收回：先释放头占格，粘头则拉回正推节点集
    claimed_heads.remove(&head);
    if bound_front {
        if let Some(movement) = try_deform_action(
            world,
            structures,
            suction,
            claimed_heads,
            pos,
            id,
            structure_id,
            true,
            -offset,
            animation,
            false,
            motion_held,
            motion_tags,
            succeeded_deform,
            actuating_extend,
            actuating_retract,
        ) {
            return Some(movement);
        }
    }
    Some(
        StructureMove::translate_by_pusher_actor(
            structure_id,
            HashSet::from([pos]),
            IVec3::ZERO,
            PusherActor { id, pos, animation },
            MovementMark::Push,
        )
        .with_source(id, pos),
    )
}

/// 一次 deform：按节点数升序试候选；共轴已成功则只挂动画；节点撞 held 则试下一条
fn try_deform_action(
    world: &WorldBlocks,
    structures: &mut StructureState,
    suction: &SuctionLinks,
    claimed_heads: &mut HashSet<IVec3>,
    pos: IVec3,
    id: BlockId,
    structure_id: StructureId,
    forward: bool,
    move_offset: IVec3,
    animation: PusherAnimationKind,
    claim_head: bool,
    motion_held: &mut HashSet<IVec3>,
    motion_tags: &mut HashMap<IVec3, IVec3>,
    succeeded_deform: &mut HashSet<(StructureId, u32)>,
    actuating_extend: &HashSet<BlockId>,
    actuating_retract: &HashSet<BlockId>,
) -> Option<StructureMove> {
    let group_indices: Vec<u32> = {
        let (_, indices) = structures.deform_action_groups(world, pos, forward)?;
        indices.to_vec()
    };

    for group_idx in group_indices {
        let (nodes, actions) = {
            let structure = structures.get(structure_id)?;
            let group = structure.deform_groups.get(group_idx as usize)?;
            (group.nodes.clone(), group.actions.clone())
        };

        // 共轴切断集：未出现在 nodes 里的同伴动作，本回合必须同样在伸/缩
        let peers_ready = actions.iter().all(|(body, action_fwd)| {
            if *body == id || *action_fwd != forward {
                return true;
            }
            if nodes.contains(body) {
                return true;
            }
            if forward {
                actuating_extend.contains(body)
            } else {
                actuating_retract.contains(body)
            }
        });
        if !peers_ready {
            continue;
        }

        if succeeded_deform.contains(&(structure_id, group_idx)) {
            if claim_head {
                let head = pos
                    + world
                        .blocks
                        .get(&pos)
                        .map(|b| b.facing.forward_ivec3())
                        .unwrap_or(IVec3::ZERO);
                // 世界尚未提交位移：头格上的货物可能已在本回合 motion_held
                if !world.is_fragile_material_at(head)
                    && world.is_occupied(head)
                    && !motion_held.contains(&head)
                {
                    continue;
                }
                if !claimed_heads.insert(head) {
                    return None;
                }
            }
            return Some(
                StructureMove::translate_by_pusher_actor(
                    structure_id,
                    HashSet::from([pos]),
                    IVec3::ZERO,
                    PusherActor { id, pos, animation },
                    MovementMark::Push,
                )
                .with_source(id, pos),
            );
        }

        if nodes.iter().any(|n| structures.held_blocks.contains(n)) {
            continue;
        }

        let (subset, anchored) = {
            let structure = structures.get(structure_id)?;
            let subset = structures.nodes_to_positions(world, structure, &nodes);
            let anchored = structure.is_scene_anchored_subset(&subset);
            (subset, anchored)
        };
        if anchored || subset.iter().any(|p| motion_held.contains(p)) {
            continue;
        }

        let head = pos
            + world
                .blocks
                .get(&pos)
                .map(|b| b.facing.forward_ivec3())
                .unwrap_or(IVec3::ZERO);

        if subset.is_empty() {
            if move_offset != IVec3::ZERO && !claim_head {
                continue;
            }
            if claim_head {
                if !world.is_fragile_material_at(head) && world.is_occupied(head) {
                    continue;
                }
                if !claimed_heads.insert(head) {
                    return None;
                }
            }
            structures.held_blocks.extend(nodes.iter().copied());
            succeeded_deform.insert((structure_id, group_idx));
            return Some(
                StructureMove::translate_by_pusher_actor(
                    structure_id,
                    HashSet::from([pos]),
                    IVec3::ZERO,
                    PusherActor { id, pos, animation },
                    MovementMark::Push,
                )
                .with_source(id, pos),
            );
        }

        let Some(expanded) = structures.linked_expand_pusher_subset(suction, &subset, move_offset)
        else {
            continue;
        };
        let mut heads_for_check = claimed_heads.clone();
        if let Some(body) = PusherState::body_at_extended_head(world, head) {
            if expanded.contains(&body) {
                heads_for_check.remove(&head);
            }
        }
        if claim_head && expanded.contains(&head) {
            heads_for_check.remove(&head);
        }
        if !claim_head {
            heads_for_check.remove(&head);
        }
        if !can_translate_structure(
            world,
            &expanded,
            move_offset,
            structures,
            suction,
            &heads_for_check,
        ) {
            continue;
        }
        if claim_head {
            if let Some(body) = PusherState::body_at_extended_head(world, head) {
                if expanded.contains(&body) {
                    claimed_heads.remove(&head);
                }
            }
            if !claimed_heads.insert(head) {
                return None;
            }
        }

        apply_motion_tags(&expanded, move_offset, motion_held, motion_tags);
        structures.held_blocks.extend(nodes.iter().copied());
        for &p in &expanded {
            if let Some(b) = world.blocks.get(&p) {
                structures.held_blocks.insert(b.id);
            }
        }
        succeeded_deform.insert((structure_id, group_idx));
        structures.moving_structures.insert(structure_id);

        return Some(
            StructureMove::translate_by_pusher_actor(
                structure_id,
                expanded,
                move_offset,
                PusherActor { id, pos, animation },
                MovementMark::Push,
            )
            .with_source(id, pos),
        );
    }
    None
}

fn apply_motion_tags(
    structure: &HashSet<IVec3>,
    offset: IVec3,
    motion_held: &mut HashSet<IVec3>,
    motion_tags: &mut HashMap<IVec3, IVec3>,
) {
    for &pos in structure {
        motion_held.insert(pos);
        motion_tags.insert(pos, offset);
    }
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
    actor: IVec3,
    mut source: IVec3,
    offset: IVec3,
    mark: MovementMark,
    suction: &SuctionLinks,
) -> Option<StructureMove> {
    // 推到已伸出的头：视为推动该推杆整坨（头+体占两格）
    if structures.id_at(source).is_none() {
        source = PusherState::body_at_extended_head(world, source)?;
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
    pos: IVec3,
    range: i32,
    suction: &SuctionLinks,
) -> Vec<StructureMove> {
    let mut moves = Vec::new();
    let mut seen_ids = HashSet::new();
    for height in 1..=range {
        let candidate = pos + IVec3::Y * height;
        let seed = structures
            .id_at(candidate)
            .map(|_| candidate)
            .or_else(|| PusherState::body_at_extended_head(world, candidate));
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
            pos,
            seed,
            IVec3::Y,
            MovementMark::Vertical,
            suction,
        ) else {
            seen_ids.remove(&id);
            continue;
        };
        // 不在标记期用 can_translate 过滤：抬不动也要打标签，merge 才能压住重力，
        // 否则被挡住时下落→再抬→上下弹（65ff7b1）。执行阶段推不动则原地不动。
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{BlockData, BlockKind, SceneBlockId};
    use crate::simulation::structure_state::StructureState;
    use crate::simulation::structures::{
        MovementInfluenceCache, MovementMark, StructureMove, execute_structure_moves_with_pushers,
    };
    use crate::simulation::suction::SuctionLinks;
    use crate::world::Facing;
    use glam::IVec3;
    use std::collections::HashSet;

    /// 准星读到的面对面活塞：北向 Pusher@(11,2,-4) 对南向 Pusher@(11,2,-5)
    fn face_to_face_cursor_world() -> (WorldBlocks, IVec3, IVec3) {
        let mut world = WorldBlocks::default();
        // 地面（现场 Scene）
        for x in 9..13 {
            for z in -6..2 {
                world.insert(
                    IVec3::new(x, 0, z),
                    BlockData::new(BlockKind::Scene(SceneBlockId(6)), Facing::North),
                );
            }
        }
        let north = IVec3::new(11, 2, -4);
        let south = IVec3::new(11, 2, -5);
        world.insert(south, BlockData::new(BlockKind::Pusher, Facing::South));
        world.insert(north, BlockData::new(BlockKind::Pusher, Facing::North));
        world.insert(
            IVec3::new(11, 2, -3),
            BlockData::new(BlockKind::Wire, Facing::North),
        );
        world.insert(
            IVec3::new(11, 3, -3),
            BlockData::new(BlockKind::Detector, Facing::South),
        );
        world.insert(
            IVec3::new(11, 4, -2),
            BlockData::new(BlockKind::Platform, Facing::North),
        );
        (world, north, south)
    }

    fn run_pusher_phase(
        world: &mut WorldBlocks,
        structures: &mut StructureState,
        pusher_state: &mut PusherState,
        powered: &HashSet<IVec3>,
    ) {
        let suction = SuctionLinks::rebuild(world, structures, powered);
        let moves =
            mark_structure_movement_phase(world, powered, structures, pusher_state, &suction);
        let heads = PusherState::hard_head_occupancy(world);
        let mut influence = MovementInfluenceCache::default();
        let (_a, _p, commits) = execute_structure_moves_with_pushers(
            world,
            moves,
            structures,
            &mut influence,
            &heads,
            &suction,
        );
        for (id, extended) in commits {
            pusher_state.set_extended(world, id, extended);
        }
    }

    /// 平行双杆正面列相连：仅 Blocker 欲伸出时须拖走未动的 Pusher 本体
    #[test]
    fn parallel_gap_solo_blocker_extend_drags_pusher_body() {
        let mut world = WorldBlocks::default();
        let pusher_pos = IVec3::new(-4, 2, -5);
        let blocker_pos = IVec3::new(-4, 2, -3);
        world.insert(pusher_pos, BlockData::new(BlockKind::Pusher, Facing::East));
        world.insert(
            blocker_pos,
            BlockData::new(BlockKind::Blocker, Facing::East),
        );
        world.insert(
            IVec3::new(-3, 2, -5),
            BlockData::new(BlockKind::Platform, Facing::North),
        );
        world.insert(
            IVec3::new(-3, 2, -4),
            BlockData::new(BlockKind::Platform, Facing::North),
        );
        world.insert(
            IVec3::new(-3, 2, -3),
            BlockData::new(BlockKind::Platform, Facing::North),
        );
        let pusher_id = world.blocks.get(&pusher_pos).unwrap().id;

        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher_state = PusherState::rebuild_from_world(&world);
        let powered = HashSet::new();
        run_pusher_phase(&mut world, &mut structures, &mut pusher_state, &powered);

        let pusher_after = world
            .blocks
            .iter()
            .find(|(_, b)| b.id == pusher_id)
            .map(|(p, _)| *p)
            .expect("pusher");
        assert_eq!(
            pusher_after,
            pusher_pos + IVec3::new(1, 0, 0),
            "solo Blocker extend must drag Pusher body east; was {pusher_after:?}"
        );
    }

    #[test]
    fn face_to_face_pusher_extend_retract_body_stays() {
        let (mut world, north, south) = face_to_face_cursor_world();
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher_state = PusherState::rebuild_from_world(&world);

        let north_id = world.blocks.get(&north).unwrap().id;
        let south_id = world.blocks.get(&south).unwrap().id;

        // 仅通电北向杆一回合（对面无电）
        let powered = HashSet::from([north]);
        run_pusher_phase(&mut world, &mut structures, &mut pusher_state, &powered);

        let north_after_ext = world
            .blocks
            .iter()
            .find(|(_, b)| b.id == north_id)
            .map(|(p, _)| *p);

        assert_eq!(
            north_after_ext,
            Some(north),
            "extend must not move the powered pusher body"
        );

        // 断电收回
        let powered = HashSet::new();
        run_pusher_phase(&mut world, &mut structures, &mut pusher_state, &powered);

        let north_after_ret = world
            .blocks
            .iter()
            .find(|(_, b)| b.id == north_id)
            .map(|(p, _)| *p);
        let south_after_ret = world
            .blocks
            .iter()
            .find(|(_, b)| b.id == south_id)
            .map(|(p, _)| *p);

        assert_eq!(
            north_after_ret,
            Some(north),
            "retract must not move the powered pusher body"
        );
        assert_eq!(
            south_after_ret,
            Some(south),
            "opposite pusher should return to original cell after retract"
        );
    }

    /// 三连同向 Blocker 链：每回合最多再伸一根，后手不得把头伸进仍停在身前的同伴
    #[test]
    fn same_facing_blocker_chain_no_head_body_overlap() {
        let mut world = WorldBlocks::default();
        let front = IVec3::new(4, 1, -14);
        let mid = IVec3::new(4, 1, -13);
        let rear = IVec3::new(4, 1, -12);
        world.insert(front, BlockData::new(BlockKind::Blocker, Facing::North));
        world.insert(mid, BlockData::new(BlockKind::Blocker, Facing::North));
        world.insert(rear, BlockData::new(BlockKind::Blocker, Facing::North));
        let ids = [
            world.blocks.get(&front).unwrap().id,
            world.blocks.get(&mid).unwrap().id,
            world.blocks.get(&rear).unwrap().id,
        ];
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher_state = PusherState::rebuild_from_world(&world);
        let powered = HashSet::new();
        for turn in 1..=3 {
            run_pusher_phase(&mut world, &mut structures, &mut pusher_state, &powered);
            structures.rebuild_for_simulation(&world);
            let extended_count = ids
                .iter()
                .filter(|id| pusher_state.entries.get(id).is_some_and(|e| e.extended))
                .count();
            let expected = match turn {
                1 => 2, // 前空头 + 中反推
                _ => 3,
            };
            assert_eq!(
                extended_count, expected,
                "turn {turn}: deform chain extended={extended_count} expected={expected}"
            );
            for id in ids {
                let Some(body) = world
                    .blocks
                    .iter()
                    .find(|(_, b)| b.id == id)
                    .map(|(p, _)| *p)
                else {
                    continue;
                };
                let ext = pusher_state.entries.get(&id).is_some_and(|e| e.extended);
                let facing = world.blocks.get(&body).unwrap().facing;
                let head = body + facing.forward_ivec3();
                if ext {
                    assert_eq!(
                        world.blocks.get(&head).map(|b| b.kind),
                        Some(BlockKind::PusherHead),
                        "turn {turn}: extended {id:?} at {body:?} needs real head"
                    );
                } else if world
                    .blocks
                    .get(&head)
                    .is_some_and(|f| matches!(f.kind, BlockKind::Blocker | BlockKind::Pusher))
                {
                    // 身前仍是同伴本体：保持 hold
                }
            }
            for (pos, b) in &world.blocks {
                if !matches!(b.kind, BlockKind::Blocker | BlockKind::Pusher) {
                    continue;
                }
                let head = *pos + b.facing.forward_ivec3();
                if world
                    .blocks
                    .get(&head)
                    .is_some_and(|f| matches!(f.kind, BlockKind::Blocker | BlockKind::Pusher))
                {
                    assert!(
                        !pusher_state.entries.get(&b.id).is_some_and(|e| e.extended),
                        "turn {turn}: blocker at {pos:?} must not extend into peer at {head:?}"
                    );
                }
            }
        }
    }

    /// 准星贴脸对向：北 Blocker@(0,2,-14) 对南 Blocker@(0,2,-15)
    /// 先手正推后，后手本回合 hold，不可再伸头（否则两头叠在中间格）
    fn face_to_face_blocker_pair_world() -> (WorldBlocks, IVec3, IVec3) {
        let mut world = WorldBlocks::default();
        let north = IVec3::new(0, 2, -14);
        let south = IVec3::new(0, 2, -15);
        world.insert(south, BlockData::new(BlockKind::Blocker, Facing::South));
        world.insert(north, BlockData::new(BlockKind::Blocker, Facing::North));
        (world, north, south)
    }

    #[test]
    fn face_to_face_blocker_deform_actions() {
        let (mut world, north, south) = face_to_face_blocker_pair_world();
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher_state = PusherState::rebuild_from_world(&world);
        let north_id = world.blocks.get(&north).unwrap().id;
        let south_id = world.blocks.get(&south).unwrap().id;
        assert!(south_id.0 < north_id.0);

        let powered = HashSet::new();
        run_pusher_phase(&mut world, &mut structures, &mut pusher_state, &powered);
        structures.rebuild_for_simulation(&world);

        assert!(
            pusher_state
                .entries
                .get(&south_id)
                .is_some_and(|e| e.extended)
        );
        assert!(
            !pusher_state
                .entries
                .get(&north_id)
                .is_some_and(|e| e.extended)
        );

        let heads: Vec<_> = world
            .blocks
            .iter()
            .filter(|(_, b)| b.kind == BlockKind::PusherHead)
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(heads.len(), 1);

        let north_pos = world
            .blocks
            .iter()
            .find(|(_, b)| b.id == north_id)
            .map(|(p, _)| *p)
            .unwrap();
        let south_pos = world
            .blocks
            .iter()
            .find(|(_, b)| b.id == south_id)
            .map(|(p, _)| *p)
            .unwrap();
        assert_eq!(south_pos, south);
        assert_eq!(north_pos, north + IVec3::new(0, 0, 1));
        assert_eq!(structures.id_at(north_pos), structures.id_at(south_pos));
    }

    #[test]
    fn bent_three_blocker_east_forward_north_reverse() {
        let mut world = WorldBlocks::default();
        let east = IVec3::new(8, 2, -13);
        let south = IVec3::new(9, 2, -13);
        let north = IVec3::new(8, 2, -12);
        world.insert(east, BlockData::new(BlockKind::Blocker, Facing::East));
        world.insert(south, BlockData::new(BlockKind::Blocker, Facing::South));
        world.insert(north, BlockData::new(BlockKind::Blocker, Facing::North));

        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher_state = PusherState::rebuild_from_world(&world);
        let east_id = world.blocks.get(&east).unwrap().id;
        let south_id = world.blocks.get(&south).unwrap().id;
        let north_id = world.blocks.get(&north).unwrap().id;
        assert!(east_id.0 < south_id.0 && south_id.0 < north_id.0);

        let powered = HashSet::new();
        run_pusher_phase(&mut world, &mut structures, &mut pusher_state, &powered);

        assert!(
            pusher_state
                .entries
                .get(&east_id)
                .is_some_and(|e| e.extended)
        );
        assert!(
            !pusher_state
                .entries
                .get(&south_id)
                .is_some_and(|e| e.extended)
        );
        assert!(
            pusher_state
                .entries
                .get(&north_id)
                .is_some_and(|e| e.extended)
        );

        let east_pos = world
            .blocks
            .iter()
            .find(|(_, b)| b.id == east_id)
            .map(|(p, _)| *p)
            .unwrap();
        let south_pos = world
            .blocks
            .iter()
            .find(|(_, b)| b.id == south_id)
            .map(|(p, _)| *p)
            .unwrap();
        let north_pos = world
            .blocks
            .iter()
            .find(|(_, b)| b.id == north_id)
            .map(|(p, _)| *p)
            .unwrap();
        assert_eq!(east_pos, east);
        assert_eq!(south_pos, south + IVec3::X);
        assert_eq!(north_pos, north + IVec3::new(0, 0, 1));
    }

    /// 准星 2×2 同向四格环：两西向 Blocker + 身前 Platform，伸出后仍是一块工厂结构
    fn same_facing_blocker_square_world() -> (WorldBlocks, IVec3, IVec3) {
        let mut world = WorldBlocks::default();
        let a = IVec3::new(-12, 2, -13);
        let b = IVec3::new(-12, 2, -12);
        world.insert(
            IVec3::new(-13, 2, -13),
            BlockData::new(BlockKind::Platform, Facing::West),
        );
        world.insert(
            IVec3::new(-13, 2, -12),
            BlockData::new(BlockKind::Platform, Facing::West),
        );
        world.insert(a, BlockData::new(BlockKind::Blocker, Facing::West));
        world.insert(b, BlockData::new(BlockKind::Blocker, Facing::West));
        (world, a, b)
    }

    #[test]
    fn same_facing_blocker_square_extend_stays_one_structure() {
        let (mut world, a, b) = same_facing_blocker_square_world();
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher_state = PusherState::rebuild_from_world(&world);
        let a_id = world.blocks.get(&a).unwrap().id;
        let b_id = world.blocks.get(&b).unwrap().id;
        assert_eq!(structures.id_at(a), structures.id_at(b));
        assert_eq!(
            structures
                .structure_positions(structures.id_at(a).unwrap())
                .map(|s| s.len()),
            Some(4)
        );

        let powered = HashSet::new();
        run_pusher_phase(&mut world, &mut structures, &mut pusher_state, &powered);
        structures.rebuild_for_simulation(&world);

        let a_pos = world
            .blocks
            .iter()
            .find(|(_, blk)| blk.id == a_id)
            .map(|(p, _)| *p)
            .expect("blocker a");
        let b_pos = world
            .blocks
            .iter()
            .find(|(_, blk)| blk.id == b_id)
            .map(|(p, _)| *p)
            .expect("blocker b");
        assert_eq!(
            a_pos, a,
            "same-facing square must not reverse-drag blocker bodies"
        );
        assert_eq!(
            b_pos, b,
            "same-facing square must not reverse-drag blocker bodies"
        );
        assert_eq!(
            structures.id_at(a_pos),
            structures.id_at(b_pos),
            "factory structure must stay one piece after extend"
        );
        assert_eq!(
            structures
                .structure_positions(structures.id_at(a_pos).unwrap())
                .map(|s| s.len()),
            Some(4),
            "still four factory members (heads are not members)"
        );
        for (id, body) in [(a_id, a_pos), (b_id, b_pos)] {
            let facing = world.blocks.get(&body).unwrap().facing;
            let head = body + facing.forward_ivec3();
            assert_eq!(
                world.blocks.get(&head).map(|blk| blk.kind),
                Some(BlockKind::PusherHead),
                "blocker {id:?} needs its own head cell"
            );
        }
    }

    /// 准星 8 格 3×3 对向拦截器环：一正推应拖走同伴，两杆错开竖轴且头不叠块
    fn opposing_blocker_ring_world() -> (WorldBlocks, IVec3, IVec3) {
        let mut world = WorldBlocks::default();
        let west = IVec3::new(13, 2, 16);
        let east = IVec3::new(13, 4, 16);
        // 3×3 环：两侧立柱平台 + 对向 Blocker，中心空
        for (x, y, z) in [
            (12, 2, 16),
            (12, 3, 16),
            (12, 4, 16),
            (14, 2, 16),
            (14, 3, 16),
            (14, 4, 16),
        ] {
            world.insert(
                IVec3::new(x, y, z),
                BlockData::new(BlockKind::Platform, Facing::North),
            );
        }
        world.insert(west, BlockData::new(BlockKind::Blocker, Facing::West));
        world.insert(east, BlockData::new(BlockKind::Blocker, Facing::East));
        (world, west, east)
    }

    #[test]
    fn opposing_blocker_ring_one_push_one_reverse_keeps_one_structure() {
        let (mut world, west, east) = opposing_blocker_ring_world();
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher_state = PusherState::rebuild_from_world(&world);

        let west_id = world.blocks.get(&west).unwrap().id;
        let east_id = world.blocks.get(&east).unwrap().id;
        let sid_before = structures.id_at(west).expect("structure");
        assert_eq!(structures.id_at(east), Some(sid_before));
        assert_eq!(
            structures.structure_positions(sid_before).map(|s| s.len()),
            Some(8)
        );

        // Blocker：断电要伸出
        let powered = HashSet::new();
        let suction = SuctionLinks::rebuild(&world, &structures, &powered);
        let moves = mark_structure_movement_phase(
            &mut world,
            &powered,
            &mut structures,
            &mut pusher_state,
            &suction,
        );

        let mut nonzero_push_dirs = Vec::new();
        let mut actor_anims = Vec::new();
        for m in &moves {
            if let StructureMove::Translate {
                offset,
                actors,
                mark: MovementMark::Push,
                structure,
                ..
            } = m
            {
                for a in actors {
                    actor_anims.push((a.id, a.pos, *offset, structure.len()));
                }
                if *offset != IVec3::ZERO {
                    nonzero_push_dirs.push(*offset);
                }
            }
        }
        assert!(
            !nonzero_push_dirs.is_empty() && !actor_anims.is_empty(),
            "ring should move; dirs={nonzero_push_dirs:?} actors={actor_anims:?}"
        );

        let heads = PusherState::hard_head_occupancy(&world);
        let mut influence = MovementInfluenceCache::default();
        let (_a, _p, commits) = execute_structure_moves_with_pushers(
            &mut world,
            moves,
            &mut structures,
            &mut influence,
            &heads,
            &suction,
        );
        for (id, extended) in commits {
            pusher_state.set_extended(&mut world, id, extended);
        }
        structures.rebuild_for_simulation(&world);

        let west_pos = world
            .blocks
            .iter()
            .find(|(_, b)| b.id == west_id)
            .map(|(p, _)| *p)
            .expect("west blocker");
        let east_pos = world
            .blocks
            .iter()
            .find(|(_, b)| b.id == east_id)
            .map(|(p, _)| *p)
            .expect("east blocker");
        assert_ne!(
            west_pos.x, east_pos.x,
            "reverse/drag must unstack blockers off the same vertical axis: west={west_pos:?} east={east_pos:?}"
        );
        assert_eq!(
            structures.id_at(west_pos),
            structures.id_at(east_pos),
            "ring must stay one structure after the turn"
        );
        // 两杆都应伸出真实头，且头格不得被其它方块占用（HashMap 单格单块；缺头即曾与身前重叠而未写入）
        for (id, body) in [(west_id, west_pos), (east_id, east_pos)] {
            let facing = world.blocks.get(&body).unwrap().facing;
            let head = body + facing.forward_ivec3();
            let head_kind = world.blocks.get(&head).map(|b| b.kind);
            assert_eq!(
                head_kind,
                Some(BlockKind::PusherHead),
                "blocker {id:?} at {body:?} must own empty head cell {head:?}, got {head_kind:?}"
            );
            assert!(
                pusher_state.entries.get(&id).is_some_and(|e| e.extended),
                "blocker {id:?} should be extended"
            );
        }
    }

    /// 面对面：南小 ID 先手应 198+；北小 ID 先手应 5+；次回合后手应正推顶头而非反推
    #[test]
    fn face_pair_south_lower_id_and_second_turn_forward() {
        use crate::simulation::core::simulate_turn;
        use crate::simulation::pending::PendingGeneratedMaterials;
        use crate::simulation::signals::SignalNetworkCache;
        use crate::simulation::structures::MovementInfluenceCache;

        fn floor(world: &mut WorldBlocks, x0: i32, x1: i32, z0: i32, z1: i32) {
            for z in z0..z1 {
                for x in x0..x1 {
                    world.insert(
                        IVec3::new(x, 0, z),
                        BlockData::new(BlockKind::Scene(SceneBlockId(6)), Facing::North),
                    );
                }
            }
        }

        fn pos_of(world: &WorldBlocks, id: crate::blocks::BlockId) -> IVec3 {
            world
                .blocks
                .iter()
                .find(|(_, b)| b.id == id)
                .map(|(p, _)| *p)
                .expect("block id present")
        }

        // Pair A: 北小 ID（同 #5/#156）
        {
            let mut world = WorldBlocks::default();
            floor(&mut world, -1, 2, -20, -10);
            let north = IVec3::new(0, 2, -14);
            let south = IVec3::new(0, 2, -15);
            world.insert(north, BlockData::new(BlockKind::Blocker, Facing::North));
            world.insert(south, BlockData::new(BlockKind::Blocker, Facing::South));
            let n_id = world.blocks.get(&north).unwrap().id;
            let s_id = world.blocks.get(&south).unwrap().id;
            assert!(n_id.0 < s_id.0);
            let mut structures = StructureState::default();
            structures.rebuild_for_simulation(&world);
            let mut pusher = PusherState::rebuild_from_world(&world);
            let mut pending = PendingGeneratedMaterials::default();
            let mut signals = SignalNetworkCache::default();
            let mut influence = MovementInfluenceCache::default();
            simulate_turn(
                &mut world,
                &mut pending,
                &mut signals,
                1,
                &mut structures,
                &mut influence,
                &mut pusher,
                None,
                None,
            );
            assert!(
                pusher.entries.get(&n_id).is_some_and(|e| e.extended),
                "T1 north+"
            );
            assert!(
                !pusher.entries.get(&s_id).is_some_and(|e| e.extended),
                "T1 south held"
            );
            assert_eq!(pos_of(&world, n_id), north);
            assert_eq!(pos_of(&world, s_id), south + IVec3::NEG_Z);
            let head = world.blocks.get(&(north + IVec3::NEG_Z));
            assert_eq!(head.map(|b| b.kind), Some(BlockKind::PusherHead));
            assert_eq!(head.map(|b| b.facing), Some(Facing::North));
            // T2: south 正推顶头，北侧被顶走；反推会把南体再往 -Z 移一格
            simulate_turn(
                &mut world,
                &mut pending,
                &mut signals,
                2,
                &mut structures,
                &mut influence,
                &mut pusher,
                None,
                None,
            );
            assert!(
                pusher.entries.get(&s_id).is_some_and(|e| e.extended),
                "T2 south should forward-extend"
            );
            assert_eq!(
                pos_of(&world, s_id),
                south + IVec3::NEG_Z,
                "T2 must not reverse south"
            );
            assert_eq!(
                pos_of(&world, n_id),
                north + IVec3::Z,
                "T2 forward pushes north partner"
            );
            let hs = world.blocks.get(&(south + IVec3::NEG_Z + IVec3::Z));
            assert_eq!(
                hs.map(|b| b.facing),
                Some(Facing::South),
                "T2 south head facing"
            );
        }

        // Pair B: 南小 ID（同 #198/#199）
        {
            let mut world = WorldBlocks::default();
            floor(&mut world, 8, 11, -8, 0);
            let south = IVec3::new(9, 2, -5);
            let north = IVec3::new(9, 2, -4);
            world.insert(south, BlockData::new(BlockKind::Blocker, Facing::South));
            world.insert(north, BlockData::new(BlockKind::Blocker, Facing::North));
            let s_id = world.blocks.get(&south).unwrap().id;
            let n_id = world.blocks.get(&north).unwrap().id;
            assert!(s_id.0 < n_id.0);
            let mut structures = StructureState::default();
            structures.rebuild_for_simulation(&world);
            let mut pusher = PusherState::rebuild_from_world(&world);
            let mut pending = PendingGeneratedMaterials::default();
            let mut signals = SignalNetworkCache::default();
            let mut influence = MovementInfluenceCache::default();
            simulate_turn(
                &mut world,
                &mut pending,
                &mut signals,
                1,
                &mut structures,
                &mut influence,
                &mut pusher,
                None,
                None,
            );
            assert!(
                pusher.entries.get(&s_id).is_some_and(|e| e.extended),
                "T1 south lower-id must 198+"
            );
            assert!(
                !pusher.entries.get(&n_id).is_some_and(|e| e.extended),
                "T1 north must not 199-"
            );
            assert_eq!(pos_of(&world, s_id), south);
            assert_eq!(pos_of(&world, n_id), north + IVec3::Z);
            let head = world.blocks.get(&(south + IVec3::Z));
            assert_eq!(head.map(|b| b.facing), Some(Facing::South));
            // T2: north 正推顶头
            simulate_turn(
                &mut world,
                &mut pending,
                &mut signals,
                2,
                &mut structures,
                &mut influence,
                &mut pusher,
                None,
                None,
            );
            assert!(
                pusher.entries.get(&n_id).is_some_and(|e| e.extended),
                "T2 north should forward-extend"
            );
            assert_eq!(
                pos_of(&world, n_id),
                north + IVec3::Z,
                "T2 must not reverse north"
            );
            assert_eq!(
                pos_of(&world, s_id),
                south + IVec3::NEG_Z,
                "T2 forward pushes south partner"
            );
            let hn = world.blocks.get(&(north + IVec3::Z + IVec3::NEG_Z));
            assert_eq!(
                hn.map(|b| b.facing),
                Some(Facing::North),
                "T2 north head facing"
            );
        }
    }
}
