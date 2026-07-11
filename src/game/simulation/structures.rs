use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::game::blocks::{BlockData, BlockId, MovementRule};
use crate::game::world::animation::{BlockAnimation, BlockAnimationKind, PusherAnimation};
use crate::game::world::direction::Facing;
use crate::game::world::grid::{MaterialFace, WorldBlocks};

use super::structure_state::{StructureId, StructureState};

pub(crate) use super::structure_state::material_structure;

pub(super) fn gravity_moves(
    world: &WorldBlocks,
    structures: &mut StructureState,
    skip_factory_positions: &HashSet<IVec3>,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> Vec<StructureMove> {
    let ids = structures.gravity_structure_ids();
    let mut moves = Vec::new();
    let mut handled = HashSet::new();

    for id in ids {
        let Some(positions) = structures.structure_positions(id) else {
            continue;
        };
        let Some(&sample) = positions.iter().next() else {
            continue;
        };
        if handled.contains(&sample) {
            continue;
        }

        let structure = positions.clone();
        handled.extend(structure.iter().copied());

        if structure
            .iter()
            .any(|pos| skip_factory_positions.contains(pos))
        {
            continue;
        }
        if structure
            .iter()
            .any(|pos| world.anchors_material_at_teleport_entrance(*pos))
        {
            continue;
        }
        if structure_supported_by_lifter(world, &structure) {
            structures.clear_gravity_support(id);
            continue;
        }
        if structures.gravity_support_valid(id, world) {
            continue;
        }
        // 整块结构一起下落；不可拆开，否则焊接材料会被撕开并丢掉焊缝
        if can_move_gravity_structure(world, &structure, structures, hard_pusher_head_occupancy) {
            structures.clear_gravity_support(id);
            moves.push(StructureMove::translate_marked(
                id,
                structure,
                IVec3::NEG_Y,
                MovementMark::Vertical,
            ));
        } else {
            structures.record_gravity_support(id, world);
        }
    }
    moves
}

pub(super) enum StructureMove {
    Translate {
        structure_id: StructureId,
        structure: HashSet<IVec3>,
        offset: IVec3,
        actor: Option<PusherActor>,
        mark: MovementMark,
        source: Option<BlockId>,
        source_pos: Option<IVec3>,
    },
    Rotate {
        structure_id: StructureId,
        structure: HashSet<IVec3>,
        pivot: IVec3,
        clockwise: bool,
        source: Option<BlockId>,
        source_pos: Option<IVec3>,
    },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum MovementMark {
    Conveyor,
    Push,
    Vertical,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PusherActor {
    pub(super) pos: IVec3,
    pub(super) animation: PusherAnimationKind,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum PusherAnimationKind {
    Extend,
    Retract,
}

#[derive(Resource, Default, Clone)]
pub struct MovementInfluenceCache {
    /// 按结构 ID + 推动源方块 ID 累计；结构 ID 稳定时跨回合保留，优先未作用过的源
    counts: HashMap<StructureId, HashMap<BlockId, u32>>,
}

impl MovementInfluenceCache {
    pub fn clear(&mut self) {
        self.counts.clear();
    }

    fn count(&self, movement: &StructureMove) -> u32 {
        let Some(source) = movement.source() else {
            return 0;
        };
        self.counts
            .get(&movement.structure_id())
            .and_then(|sources| sources.get(&source).copied())
            .unwrap_or(0)
    }

    /// 只丢掉已不存在的结构/源，不因本回合未作用而清零
    fn prune_missing(
        &mut self,
        living_structures: &HashSet<StructureId>,
        living_blocks: &HashSet<BlockId>,
    ) {
        self.counts.retain(|structure, sources| {
            if !living_structures.contains(structure) {
                return false;
            }
            sources.retain(|source, _| living_blocks.contains(source));
            !sources.is_empty()
        });
    }

    fn record_executed(&mut self, executed: Vec<ExecutedMovement>) {
        for movement in executed {
            *self
                .counts
                .entry(movement.structure_id)
                .or_default()
                .entry(movement.source)
                .or_insert(0) += 1;
        }
    }
}

struct ExecutedMovement {
    structure_id: StructureId,
    source: BlockId,
}

impl StructureMove {
    pub(super) fn translate_marked(
        structure_id: StructureId,
        structure: HashSet<IVec3>,
        offset: IVec3,
        mark: MovementMark,
    ) -> Self {
        Self::Translate {
            structure_id,
            structure,
            offset,
            actor: None,
            mark,
            source: None,
            source_pos: None,
        }
    }

    pub(super) fn translate_by_pusher_actor(
        structure_id: StructureId,
        structure: HashSet<IVec3>,
        offset: IVec3,
        actor: PusherActor,
        mark: MovementMark,
    ) -> Self {
        Self::Translate {
            structure_id,
            structure,
            offset,
            actor: Some(actor),
            mark,
            source: None,
            source_pos: None,
        }
    }

    pub(super) fn rotate(
        structure_id: StructureId,
        structure: HashSet<IVec3>,
        pivot: IVec3,
        clockwise: bool,
    ) -> Self {
        Self::Rotate {
            structure_id,
            structure,
            pivot,
            clockwise,
            source: None,
            source_pos: None,
        }
    }

    pub(super) fn with_source(mut self, source: BlockId, source_pos: IVec3) -> Self {
        match &mut self {
            Self::Translate {
                source: slot,
                source_pos: pos_slot,
                ..
            }
            | Self::Rotate {
                source: slot,
                source_pos: pos_slot,
                ..
            } => {
                *slot = Some(source);
                *pos_slot = Some(source_pos);
            }
        }
        self
    }

    fn source(&self) -> Option<BlockId> {
        match self {
            Self::Translate { source, .. } | Self::Rotate { source, .. } => *source,
        }
    }

    fn source_pos(&self) -> Option<IVec3> {
        match self {
            Self::Translate { source_pos, .. } | Self::Rotate { source_pos, .. } => *source_pos,
        }
    }

    pub(super) fn structure_id(&self) -> StructureId {
        match self {
            Self::Translate { structure_id, .. } | Self::Rotate { structure_id, .. } => {
                *structure_id
            }
        }
    }

    pub(super) fn structure(&self) -> &HashSet<IVec3> {
        match self {
            Self::Translate { structure, .. } | Self::Rotate { structure, .. } => structure,
        }
    }
}

/// 合并重力与设备运动标签：保留全部重叠标签，按优先级排序，执行时再 fallback
pub(super) fn merge_structure_movement_plan(
    mut planned_moves: Vec<StructureMove>,
    device_moves: Vec<StructureMove>,
    influence_cache: &mut MovementInfluenceCache,
    structures: &StructureState,
    world: &WorldBlocks,
) -> Vec<StructureMove> {
    let living_structures: HashSet<StructureId> = structures.structure_ids().collect();
    let living_blocks: HashSet<BlockId> = world.blocks.values().map(|block| block.id).collect();
    influence_cache.prune_missing(&living_structures, &living_blocks);
    planned_moves.extend(device_moves);
    planned_moves.sort_by(|a, b| compare_movement_priority(a, b, influence_cache));
    planned_moves
}

fn compare_movement_priority(
    a: &StructureMove,
    b: &StructureMove,
    influence_cache: &MovementInfluenceCache,
) -> Ordering {
    movement_priority_key(a, influence_cache).cmp(&movement_priority_key(b, influence_cache))
}

fn movement_priority_key(
    movement: &StructureMove,
    influence_cache: &MovementInfluenceCache,
) -> (u8, u32, ConveyorSourcePriority) {
    // 种类优先：活塞 > 抬升 > 下落 > 旋转 > 传送带
    (
        movement_kind_priority(movement),
        movement
            .source()
            .map_or(0, |_| influence_cache.count(movement)),
        conveyor_source_priority(movement),
    )
}

fn movement_kind_priority(movement: &StructureMove) -> u8 {
    match movement {
        StructureMove::Translate {
            mark: MovementMark::Push,
            ..
        } => 0,
        // 抬升器：有 source 的竖直移动
        StructureMove::Translate {
            mark: MovementMark::Vertical,
            source: Some(_),
            ..
        } => 1,
        StructureMove::Translate {
            mark: MovementMark::Vertical,
            source: None,
            ..
        } => 2,
        StructureMove::Rotate { .. } => 3,
        StructureMove::Translate {
            mark: MovementMark::Conveyor,
            ..
        } => 4,
    }
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct ConveyorSourcePriority {
    positive_x: i32,
    negative_x: i32,
    positive_y: i32,
    negative_y: i32,
    positive_z: i32,
    negative_z: i32,
}

fn conveyor_source_priority(movement: &StructureMove) -> ConveyorSourcePriority {
    let Some(source) = movement.source_pos() else {
        return ConveyorSourcePriority::neutral();
    };
    if !matches!(
        movement,
        StructureMove::Translate {
            mark: MovementMark::Conveyor,
            ..
        }
    ) {
        return ConveyorSourcePriority::neutral();
    }
    ConveyorSourcePriority {
        positive_x: -source.x,
        negative_x: source.x,
        positive_y: -source.y,
        negative_y: source.y,
        positive_z: -source.z,
        negative_z: source.z,
    }
}

impl ConveyorSourcePriority {
    fn neutral() -> Self {
        Self {
            positive_x: 0,
            negative_x: 0,
            positive_y: 0,
            negative_y: 0,
            positive_z: 0,
            negative_z: 0,
        }
    }
}

fn structure_supported_by_lifter(world: &WorldBlocks, structure: &HashSet<IVec3>) -> bool {
    world.blocks.iter().any(|(pos, block)| {
        matches!(
            block.kind.movement_rule(block.facing),
            Some(MovementRule::Lift { range }) if structure.contains(&(*pos + IVec3::Y * (range + 1)))
        )
    })
}

/// 按序执行运动标签：失败则试下一个；种子判占用，成功后标记展开后的格子
pub(super) fn execute_structure_moves_with_pushers(
    world: &mut WorldBlocks,
    moves: Vec<StructureMove>,
    structures: &mut StructureState,
    influence_cache: &mut MovementInfluenceCache,
) -> (
    HashMap<IVec3, BlockAnimation>,
    HashMap<IVec3, PusherAnimation>,
) {
    let mut moved = HashSet::new();
    let mut animations = HashMap::new();
    let mut pusher_animations = HashMap::new();
    let mut executed = Vec::new();
    for movement in moves {
        match movement {
            StructureMove::Translate {
                structure_id,
                structure,
                offset,
                actor,
                mark,
                source,
                source_pos: _,
            } => {
                // 仅用种子结构判占用；展开在当前世界上做，避免预展开导致误跳过
                if structure.iter().any(|pos| moved.contains(pos)) {
                    continue;
                }
                let Some(structure) = expanded_move_structure(
                    world,
                    &structure,
                    offset,
                    structures,
                    movement_expansion_mode(mark, source),
                ) else {
                    continue;
                };
                // 展开卷入的格子若本回合已动过，本标签失败（可 fallback）
                if structure.iter().any(|pos| moved.contains(pos)) {
                    continue;
                }
                for pos in &structure {
                    if let Some(block) = world.blocks.get(pos) {
                        animations.insert(
                            *pos + offset,
                            BlockAnimation {
                                block_id: block.id,
                                from_pos: *pos,
                                to_pos: *pos + offset,
                                from_facing: block.facing,
                                to_facing: block.facing,
                                kind: BlockAnimationKind::Move,
                                duration: None,
                                progress: None,
                            },
                        );
                    }
                }
                if let Some(actor) = actor {
                    let (from_extension, to_extension) = match actor.animation {
                        PusherAnimationKind::Extend => (0.0, 1.0),
                        PusherAnimationKind::Retract => (1.0, 0.0),
                    };
                    pusher_animations.insert(
                        actor.pos,
                        PusherAnimation {
                            duration: None,
                            from_extension,
                            to_extension,
                        },
                    );
                }
                moved.extend(structure.iter().copied());
                move_structure(world, &structure, offset);
                structures.move_positions(&structure, offset);
                let target_structure: HashSet<IVec3> =
                    structure.into_iter().map(|pos| pos + offset).collect();
                if let Some(source) = source {
                    executed.push(ExecutedMovement {
                        structure_id,
                        source,
                    });
                }
                moved.extend(target_structure);
            }
            StructureMove::Rotate {
                structure_id,
                structure,
                pivot,
                clockwise,
                source,
                source_pos: _,
            } => {
                if structure.iter().any(|pos| moved.contains(pos)) {
                    continue;
                }
                if can_rotate_structure(world, &structure, pivot, clockwise) {
                    let targets: Vec<IVec3> = structure
                        .iter()
                        .map(|pos| rotate_pos_y(*pos, pivot, clockwise))
                        .collect();
                    for pos in &structure {
                        if let Some(block) = world.blocks.get(pos) {
                            let target = rotate_pos_y(*pos, pivot, clockwise);
                            animations.insert(
                                target,
                                BlockAnimation {
                                    block_id: block.id,
                                    from_pos: *pos,
                                    to_pos: target,
                                    from_facing: block.facing,
                                    to_facing: rotate_facing(block.facing, clockwise),
                                    kind: BlockAnimationKind::Rotate { pivot, clockwise },
                                    duration: None,
                                    progress: None,
                                },
                            );
                        }
                    }
                    moved.extend(structure.iter().copied());
                    rotate_structure(world, &structure, pivot, clockwise);
                    let target_structure: HashSet<IVec3> = targets.iter().copied().collect();
                    structures.replace_structure_positions(&structure, target_structure.clone());
                    if let Some(source) = source {
                        executed.push(ExecutedMovement {
                            structure_id,
                            source,
                        });
                    }
                    moved.extend(target_structure);
                }
            }
        }
    }
    influence_cache.record_executed(executed);
    (animations, pusher_animations)
}

fn can_move_gravity_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    structures: &StructureState,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> bool {
    if hard_pusher_head_blocked_below(world, structure, hard_pusher_head_occupancy) {
        return false;
    }
    let Some(expanded) = expanded_move_structure(
        world,
        structure,
        IVec3::NEG_Y,
        structures,
        MovementExpansionMode::Gravity,
    ) else {
        return false;
    };
    !hard_pusher_head_blocks_move(&expanded, IVec3::NEG_Y, hard_pusher_head_occupancy)
}

fn hard_pusher_head_blocked_below(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> bool {
    structure.iter().any(|pos| {
        let Some(block) = world.blocks.get(pos) else {
            return false;
        };
        if !matches!(
            block.kind,
            crate::game::blocks::BlockKind::Pusher | crate::game::blocks::BlockKind::Blocker
        ) {
            return false;
        }
        let head = *pos + block.facing.forward_ivec3();
        if !hard_pusher_head_occupancy.contains(&head) {
            return false;
        }
        let target = head + IVec3::NEG_Y;
        target.y < 0 || (!structure.contains(&target) && !world.can_move_into(target))
    })
}

fn hard_pusher_head_blocks_move(
    structure: &HashSet<IVec3>,
    offset: IVec3,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> bool {
    structure.iter().any(|pos| {
        let target = *pos + offset;
        !structure.contains(&target) && hard_pusher_head_occupancy.contains(&target)
    })
}

fn expanded_move_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    offset: IVec3,
    structures: &StructureState,
    mode: MovementExpansionMode,
) -> Option<HashSet<IVec3>> {
    if offset.abs().element_sum() != 1 {
        return can_move_structure_without_push(world, structure, offset)
            .then(|| structure.clone());
    }

    let mut expanded = structure.clone();
    let mut queue: VecDeque<IVec3> = structure.iter().copied().collect();
    while let Some(pos) = queue.pop_front() {
        let target = pos + offset;
        if target.y < 0 || expanded.contains(&target) {
            continue;
        }
        if world.can_move_into(target) {
            continue;
        }

        let pushed = pushable_structure_at(world, structures, target, offset)?;
        if mode == MovementExpansionMode::Gravity && structure_supported_by_lifter(world, &pushed) {
            return None;
        }
        for pushed_pos in pushed {
            if expanded.insert(pushed_pos) {
                queue.push_back(pushed_pos);
            }
        }
    }

    can_move_structure_without_push(world, &expanded, offset)
        .then_some(expanded)
        .filter(|expanded| {
            !expanded
                .iter()
                .any(|pos| world.anchors_material_at_teleport_entrance(*pos))
        })
}

pub(super) fn can_translate_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    offset: IVec3,
    structures: &StructureState,
) -> bool {
    expanded_move_structure(
        world,
        structure,
        offset,
        structures,
        MovementExpansionMode::Normal,
    )
    .is_some()
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum MovementExpansionMode {
    Normal,
    Gravity,
}

fn movement_expansion_mode(mark: MovementMark, source: Option<BlockId>) -> MovementExpansionMode {
    if mark == MovementMark::Vertical && source.is_none() {
        MovementExpansionMode::Gravity
    } else {
        MovementExpansionMode::Normal
    }
}

fn pushable_structure_at(
    world: &WorldBlocks,
    structures: &StructureState,
    pos: IVec3,
    offset: IVec3,
) -> Option<HashSet<IVec3>> {
    let block = world.blocks.get(&pos)?;
    if block.kind.is_material() || block.kind.is_factory() {
        return structures.pushable_structure_at(pos, offset);
    }
    None
}

fn can_move_structure_without_push(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    offset: IVec3,
) -> bool {
    structure.iter().all(|pos| {
        let target = *pos + offset;
        target.y >= 0 && (structure.contains(&target) || world.can_move_into(target))
    })
}

pub(super) fn move_structure(world: &mut WorldBlocks, structure: &HashSet<IVec3>, offset: IVec3) {
    let moves: Vec<(IVec3, IVec3, BlockData)> = structure
        .iter()
        .filter_map(|pos| {
            world
                .blocks
                .get(pos)
                .copied()
                .map(|block| (*pos, *pos + offset, block))
        })
        .collect();
    world.relocate_blocks(moves);
}

pub(super) fn can_rotate_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    pivot: IVec3,
    clockwise: bool,
) -> bool {
    structure.iter().all(|pos| {
        let target = rotate_pos_y(*pos, pivot, clockwise);
        target.y >= 0 && (structure.contains(&target) || world.can_place_platform_at(target))
    })
}

pub(super) fn rotate_structure(
    world: &mut WorldBlocks,
    structure: &HashSet<IVec3>,
    pivot: IVec3,
    clockwise: bool,
) {
    let structure_ids: HashSet<BlockId> = structure
        .iter()
        .filter_map(|pos| world.blocks.get(pos).map(|block| block.id))
        .collect();
    let moves: Vec<(IVec3, IVec3, BlockData)> = structure
        .iter()
        .filter_map(|pos| {
            world.blocks.get(pos).copied().map(|mut block| {
                block.facing = rotate_facing(block.facing, clockwise);
                (*pos, rotate_pos_y(*pos, pivot, clockwise), block)
            })
        })
        .collect();
    world.relocate_blocks(moves);

    // 焊接按 BlockId 无需改写；旋转只更新面标记法线
    let updated_marks: HashMap<_, _> = world
        .material_face_marks
        .iter()
        .map(|(face, mark)| {
            if structure_ids.contains(&face.block) {
                (
                    MaterialFace {
                        block: face.block,
                        normal: rotate_offset_y(face.normal, clockwise),
                    },
                    *mark,
                )
            } else {
                (*face, *mark)
            }
        })
        .collect();
    world.material_face_marks = updated_marks;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::blocks::{BlockData, BlockKind, MaterialKind};
    use crate::game::world::direction::Facing;

    fn place(world: &mut WorldBlocks, pos: IVec3, kind: BlockKind) {
        world.insert(pos, BlockData::new(kind, Facing::North));
    }

    fn place_material(world: &mut WorldBlocks, pos: IVec3) {
        let kind = BlockKind::material_block_kind(MaterialKind::Basic).unwrap();
        place(world, pos, kind);
    }

    fn structures_for(world: &WorldBlocks) -> StructureState {
        let mut state = StructureState::default();
        state.rebuild_for_simulation(world);
        state
    }

    fn set(pos: IVec3) -> HashSet<IVec3> {
        HashSet::from([pos])
    }

    #[test]
    fn merge_keeps_overlapping_lower_priority_tags() {
        let structure = set(IVec3::new(1, 1, 0));
        let id = StructureId(1);
        let rotate = StructureMove::rotate(id, structure.clone(), IVec3::ZERO, true)
            .with_source(BlockId(1), IVec3::ZERO);
        let conveyor =
            StructureMove::translate_marked(id, structure, IVec3::X, MovementMark::Conveyor)
                .with_source(BlockId(2), IVec3::new(1, 0, 0));
        let mut cache = MovementInfluenceCache::default();
        let merged = merge_structure_movement_plan(
            vec![],
            vec![conveyor, rotate],
            &mut cache,
            &StructureState::default(),
            &WorldBlocks::default(),
        );
        assert_eq!(merged.len(), 2);
        assert!(matches!(merged[0], StructureMove::Rotate { .. }));
        assert!(matches!(
            merged[1],
            StructureMove::Translate {
                mark: MovementMark::Conveyor,
                ..
            }
        ));
    }

    #[test]
    fn rotate_fail_falls_back_to_conveyor() {
        let mut world = WorldBlocks::default();
        let pivot = IVec3::ZERO;
        let material = IVec3::new(1, 1, 0);
        place_material(&mut world, material);
        // 顺时针转到 (0,1,1)，用障碍挡住旋转
        place(&mut world, IVec3::new(0, 1, 1), BlockKind::Stone);

        let mut structures = structures_for(&world);
        let id = structures.id_at(material).unwrap();
        let structure = set(material);
        let rotate = StructureMove::rotate(id, structure.clone(), pivot, true);
        let conveyor =
            StructureMove::translate_marked(id, structure, IVec3::X, MovementMark::Conveyor);
        let mut cache = MovementInfluenceCache::default();
        let plan = merge_structure_movement_plan(
            vec![],
            vec![rotate, conveyor],
            &mut cache,
            &structures,
            &world,
        );
        execute_structure_moves_with_pushers(&mut world, plan, &mut structures, &mut cache);

        assert!(!world.is_material_at(material));
        assert!(world.is_material_at(IVec3::new(2, 1, 0)));
        assert!(world.blocks.contains_key(&IVec3::new(0, 1, 1)));
    }

    #[test]
    fn conveyor_moves_after_pusher_clears_front_structure() {
        let mut world = WorldBlocks::default();
        let a = IVec3::new(0, 1, 0);
        let b = IVec3::new(1, 1, 0);
        // 材料默认不因相邻合并，才能测跨结构推动
        place_material(&mut world, a);
        place_material(&mut world, b);

        let mut structures = structures_for(&world);
        let id_a = structures.id_at(a).unwrap();
        let id_b = structures.id_at(b).unwrap();
        let push_b =
            StructureMove::translate_marked(id_b, set(b), IVec3::new(0, 0, 1), MovementMark::Push);
        let conveyor_a =
            StructureMove::translate_marked(id_a, set(a), IVec3::X, MovementMark::Conveyor);
        let mut cache = MovementInfluenceCache::default();
        let plan = merge_structure_movement_plan(
            vec![],
            vec![conveyor_a, push_b],
            &mut cache,
            &structures,
            &world,
        );
        execute_structure_moves_with_pushers(&mut world, plan, &mut structures, &mut cache);

        assert!(!world.is_material_at(a));
        assert!(world.is_material_at(IVec3::new(1, 1, 0)));
        assert!(world.is_material_at(IVec3::new(1, 1, 1)));
    }

    #[test]
    fn conveyor_pushes_front_structure_when_pusher_fails() {
        let mut world = WorldBlocks::default();
        let a = IVec3::new(0, 1, 0);
        let b = IVec3::new(1, 1, 0);
        place_material(&mut world, a);
        place_material(&mut world, b);
        // 挡住 B 的活塞方向
        place(&mut world, IVec3::new(1, 1, 1), BlockKind::Stone);

        let mut structures = structures_for(&world);
        let id_a = structures.id_at(a).unwrap();
        let id_b = structures.id_at(b).unwrap();
        let push_b =
            StructureMove::translate_marked(id_b, set(b), IVec3::new(0, 0, 1), MovementMark::Push);
        let conveyor_a =
            StructureMove::translate_marked(id_a, set(a), IVec3::X, MovementMark::Conveyor);
        let mut cache = MovementInfluenceCache::default();
        let plan = merge_structure_movement_plan(
            vec![],
            vec![conveyor_a, push_b],
            &mut cache,
            &structures,
            &world,
        );
        execute_structure_moves_with_pushers(&mut world, plan, &mut structures, &mut cache);

        assert!(!world.is_material_at(a));
        assert!(world.is_material_at(b));
        assert!(world.is_material_at(IVec3::new(2, 1, 0)));
        assert!(world.blocks.contains_key(&IVec3::new(1, 1, 1)));
    }

    #[test]
    fn unused_mover_outranks_already_used_mover() {
        let mut world = WorldBlocks::default();
        let material = IVec3::new(1, 1, 0);
        let old_pos = IVec3::ZERO;
        let new_pos = IVec3::new(2, 0, 0);
        place_material(&mut world, material);
        place(&mut world, old_pos, BlockKind::Rotator);
        place(&mut world, new_pos, BlockKind::Rotator);
        let structures = structures_for(&world);
        let id = structures.id_at(material).unwrap();
        let old_source = world.blocks.get(&old_pos).unwrap().id;
        let new_source = world.blocks.get(&new_pos).unwrap().id;
        let mut cache = MovementInfluenceCache::default();
        cache.record_executed(vec![ExecutedMovement {
            structure_id: id,
            source: old_source,
        }]);

        let old = StructureMove::rotate(id, set(material), old_pos, true)
            .with_source(old_source, old_pos);
        let new = StructureMove::rotate(id, set(material), new_pos, false)
            .with_source(new_source, new_pos);
        // 故意把旧源放前面；排序后未作用过的新源应优先
        let plan =
            merge_structure_movement_plan(vec![], vec![old, new], &mut cache, &structures, &world);
        assert!(matches!(
            &plan[0],
            StructureMove::Rotate {
                source: Some(source),
                ..
            } if *source == new_source
        ));
    }
}
