use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::game::blocks::{BlockData, MovementRule};
use crate::game::world::animation::{BlockAnimation, BlockAnimationKind, PusherAnimation};
use crate::game::world::direction::Facing;
use crate::game::world::grid::{MaterialFace, MaterialFaceMark, MaterialWeld, WorldBlocks};

use super::structure_state::StructureState;

pub(crate) use super::structure_state::material_structure;

pub(super) fn gravity_moves(
    world: &WorldBlocks,
    structures: &mut StructureState,
    skip_factory_positions: &HashSet<IVec3>,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> Vec<StructureMove> {
    let indices = structures.gravity_structure_indices();
    let mut moves = Vec::new();
    let mut handled = HashSet::new();

    for index in indices {
        let Some(positions) = structures.structure_positions(index) else {
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
            structures.clear_gravity_support(index);
            continue;
        }
        if structures.gravity_support_valid(index, world) {
            continue;
        }
        // 整块结构一起下落；不可拆开，否则焊接材料会被撕开并丢掉焊缝
        if can_move_gravity_structure(world, &structure, structures, hard_pusher_head_occupancy) {
            structures.clear_gravity_support(index);
            moves.push(StructureMove::translate_marked(
                structure,
                IVec3::NEG_Y,
                MovementMark::Vertical,
            ));
        } else {
            structures.record_gravity_support(index, world);
        }
    }
    moves
}

pub(super) enum StructureMove {
    Translate {
        structure: HashSet<IVec3>,
        offset: IVec3,
        actor: Option<PusherActor>,
        mark: MovementMark,
        source: Option<IVec3>,
    },
    Rotate {
        structure: HashSet<IVec3>,
        pivot: IVec3,
        clockwise: bool,
        source: Option<IVec3>,
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct StructureKey(Vec<IVec3>);

#[derive(Resource, Default, Clone)]
pub struct MovementInfluenceCache {
    counts: HashMap<StructureKey, HashMap<IVec3, u32>>,
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
            .get(&StructureKey::from_structure(movement.structure()))
            .and_then(|sources| sources.get(&source).copied())
            .unwrap_or(0)
    }

    fn retain_active_sources(&mut self, active_sources: &HashMap<StructureKey, HashSet<IVec3>>) {
        self.counts.retain(|structure, sources| {
            let Some(active) = active_sources.get(structure) else {
                return false;
            };
            sources.retain(|source, _| active.contains(source));
            !sources.is_empty()
        });
    }

    fn record_executed(&mut self, executed: Vec<ExecutedMovement>) {
        for movement in executed {
            let mut counts = self.counts.remove(&movement.before).unwrap_or_default();
            if movement.before != movement.after {
                let moved_counts = counts;
                counts = self.counts.remove(&movement.after).unwrap_or_default();
                for (source, count) in moved_counts {
                    counts
                        .entry(source)
                        .and_modify(|current| *current = (*current).max(count))
                        .or_insert(count);
                }
            }
            *counts.entry(movement.source).or_insert(0) += 1;
            self.counts.insert(movement.after, counts);
        }
    }
}

impl StructureKey {
    fn from_structure(structure: &HashSet<IVec3>) -> Self {
        let mut positions: Vec<IVec3> = structure.iter().copied().collect();
        positions.sort_by_key(|pos| (pos.x, pos.y, pos.z));
        Self(positions)
    }
}

struct ExecutedMovement {
    before: StructureKey,
    after: StructureKey,
    source: IVec3,
}

impl StructureMove {
    pub(super) fn translate_marked(
        structure: HashSet<IVec3>,
        offset: IVec3,
        mark: MovementMark,
    ) -> Self {
        Self::Translate {
            structure,
            offset,
            actor: None,
            mark,
            source: None,
        }
    }

    pub(super) fn translate_by_pusher_actor(
        structure: HashSet<IVec3>,
        offset: IVec3,
        actor: PusherActor,
        mark: MovementMark,
    ) -> Self {
        Self::Translate {
            structure,
            offset,
            actor: Some(actor),
            mark,
            source: None,
        }
    }

    pub(super) fn rotate(structure: HashSet<IVec3>, pivot: IVec3, clockwise: bool) -> Self {
        Self::Rotate {
            structure,
            pivot,
            clockwise,
            source: None,
        }
    }

    pub(super) fn with_source(mut self, source_pos: IVec3) -> Self {
        match &mut self {
            Self::Translate { source, .. } | Self::Rotate { source, .. } => {
                *source = Some(source_pos);
            }
        }
        self
    }

    fn source(&self) -> Option<IVec3> {
        match self {
            Self::Translate { source, .. } | Self::Rotate { source, .. } => *source,
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
) -> Vec<StructureMove> {
    let active_sources = active_device_sources(&device_moves);
    influence_cache.retain_active_sources(&active_sources);
    planned_moves.extend(device_moves);
    planned_moves.sort_by(|a, b| compare_movement_priority(a, b, influence_cache));
    planned_moves
}

fn active_device_sources(moves: &[StructureMove]) -> HashMap<StructureKey, HashSet<IVec3>> {
    let mut active = HashMap::new();
    for movement in moves {
        let Some(source) = movement.source() else {
            continue;
        };
        active
            .entry(StructureKey::from_structure(movement.structure()))
            .or_insert_with(HashSet::new)
            .insert(source);
    }
    active
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
    let Some(source) = movement.source() else {
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
                structure,
                offset,
                actor,
                mark,
                source,
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
                let before_key = StructureKey::from_structure(&structure);
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
                        before: before_key,
                        after: StructureKey::from_structure(&target_structure),
                        source,
                    });
                }
                moved.extend(target_structure);
            }
            StructureMove::Rotate {
                structure,
                pivot,
                clockwise,
                source,
            } => {
                if structure.iter().any(|pos| moved.contains(pos)) {
                    continue;
                }
                if can_rotate_structure(world, &structure, pivot, clockwise) {
                    let before_key = StructureKey::from_structure(&structure);
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
                            before: before_key,
                            after: StructureKey::from_structure(&target_structure),
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

fn movement_expansion_mode(mark: MovementMark, source: Option<IVec3>) -> MovementExpansionMode {
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
        target.y >= 0 && (structure.contains(&target) || world.can_place_platform_at(target))
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
            let a_in = structure.contains(&weld.a);
            let b_in = structure.contains(&weld.b);
            match (a_in, b_in) {
                (false, false) => Some(*weld),
                (true, true) => Some(MaterialWeld::new(transform(weld.a), transform(weld.b))),
                _ => None,
            }
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
        let rotate =
            StructureMove::rotate(structure.clone(), IVec3::ZERO, true).with_source(IVec3::ZERO);
        let conveyor = StructureMove::translate_marked(structure, IVec3::X, MovementMark::Conveyor)
            .with_source(IVec3::new(1, 0, 0));
        let mut cache = MovementInfluenceCache::default();
        let merged = merge_structure_movement_plan(vec![], vec![conveyor, rotate], &mut cache);
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
        let structure = set(material);
        let rotate = StructureMove::rotate(structure.clone(), pivot, true);
        let conveyor = StructureMove::translate_marked(structure, IVec3::X, MovementMark::Conveyor);
        let mut cache = MovementInfluenceCache::default();
        let plan = merge_structure_movement_plan(vec![], vec![rotate, conveyor], &mut cache);
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
        let push_b =
            StructureMove::translate_marked(set(b), IVec3::new(0, 0, 1), MovementMark::Push);
        let conveyor_a = StructureMove::translate_marked(set(a), IVec3::X, MovementMark::Conveyor);
        let mut cache = MovementInfluenceCache::default();
        let plan = merge_structure_movement_plan(vec![], vec![conveyor_a, push_b], &mut cache);
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
        let push_b =
            StructureMove::translate_marked(set(b), IVec3::new(0, 0, 1), MovementMark::Push);
        let conveyor_a = StructureMove::translate_marked(set(a), IVec3::X, MovementMark::Conveyor);
        let mut cache = MovementInfluenceCache::default();
        let plan = merge_structure_movement_plan(vec![], vec![conveyor_a, push_b], &mut cache);
        execute_structure_moves_with_pushers(&mut world, plan, &mut structures, &mut cache);

        assert!(!world.is_material_at(a));
        assert!(world.is_material_at(b));
        assert!(world.is_material_at(IVec3::new(2, 1, 0)));
        assert!(world.blocks.contains_key(&IVec3::new(1, 1, 1)));
    }
}
