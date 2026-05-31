use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::game::world::animation::{BlockAnimation, BlockAnimationKind, PusherAnimation};
use crate::game::world::blocks::BlockData;
use crate::game::world::direction::Facing;
use crate::game::world::grid::{MaterialFace, MaterialFaceMark, MaterialWeld, WorldBlocks};

use super::factory_activity::FactoryStructureState;

pub(super) fn material_gravity_moves(
    world: &WorldBlocks,
    factory_structures: &FactoryStructureState,
) -> Vec<StructureMove> {
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
        if can_move_structure(world, &structure, IVec3::NEG_Y, factory_structures) {
            moves.push(StructureMove::translate_marked(
                structure,
                IVec3::NEG_Y,
                MovementMark::Vertical,
            ));
        }
    }
    moves
}

pub(super) fn factory_gravity_moves(
    world: &WorldBlocks,
    factory_structures: &FactoryStructureState,
) -> Vec<StructureMove> {
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

        let Some(structure) = factory_structures.falling_structure_at(pos, IVec3::NEG_Y) else {
            continue;
        };
        handled.extend(structure.iter().copied());
        if can_move_structure(world, &structure, IVec3::NEG_Y, factory_structures) {
            moves.push(StructureMove::translate_marked(
                structure,
                IVec3::NEG_Y,
                MovementMark::Vertical,
            ));
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

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum MovementMark {
    Conveyor,
    Push,
    Vertical,
}

#[derive(Clone, Copy)]
pub(super) struct PusherActor {
    pub(super) pos: IVec3,
    pub(super) animation: PusherAnimationKind,
}

#[derive(Clone, Copy)]
pub(super) enum PusherAnimationKind {
    Extend,
    Retract,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct StructureKey(Vec<IVec3>);

#[derive(Resource, Default)]
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

    fn structure(&self) -> &HashSet<IVec3> {
        match self {
            Self::Translate { structure, .. } | Self::Rotate { structure, .. } => structure,
        }
    }

    pub(super) fn overlaps_structure(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Translate { structure: a, .. }, Self::Translate { structure: b, .. })
            | (Self::Translate { structure: a, .. }, Self::Rotate { structure: b, .. })
            | (Self::Rotate { structure: a, .. }, Self::Translate { structure: b, .. })
            | (Self::Rotate { structure: a, .. }, Self::Rotate { structure: b, .. }) => {
                a.iter().any(|pos| b.contains(pos))
            }
        }
    }

    fn expanded_for_plan(
        self,
        world: &WorldBlocks,
        factory_structures: &FactoryStructureState,
    ) -> Self {
        match self {
            Self::Translate {
                structure,
                offset,
                actor,
                mark,
                source,
            } => {
                let structure =
                    expanded_move_structure(world, &structure, offset, factory_structures)
                        .unwrap_or(structure);
                Self::Translate {
                    structure,
                    offset,
                    actor,
                    mark,
                    source,
                }
            }
            movement => movement,
        }
    }
}

pub(super) fn merge_structure_movement_plan(
    mut planned_moves: Vec<StructureMove>,
    device_moves: Vec<StructureMove>,
    world: &WorldBlocks,
    factory_structures: &FactoryStructureState,
    influence_cache: &mut MovementInfluenceCache,
) -> Vec<StructureMove> {
    planned_moves = expand_structure_movement_plan(planned_moves, world, factory_structures);
    let mut device_moves = expand_structure_movement_plan(device_moves, world, factory_structures);
    let active_sources = active_device_sources(&device_moves);
    influence_cache.retain_active_sources(&active_sources);
    device_moves.sort_by(|a, b| compare_movement_priority(a, b, influence_cache));

    for movement in device_moves {
        let blocked_by_higher_priority = planned_moves.iter().any(|existing| {
            existing.overlaps_structure(&movement)
                && movement_beats(existing, &movement, influence_cache)
        });
        if blocked_by_higher_priority {
            continue;
        }

        planned_moves.retain(|existing| {
            !(existing.overlaps_structure(&movement)
                && movement_beats(&movement, existing, influence_cache))
        });
        planned_moves.push(movement);
    }
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

fn movement_beats(
    challenger: &StructureMove,
    existing: &StructureMove,
    influence_cache: &MovementInfluenceCache,
) -> bool {
    compare_movement_priority(challenger, existing, influence_cache) != Ordering::Greater
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
) -> (u32, u8, ConveyorSourcePriority) {
    (
        influence_cache.count(movement),
        movement_kind_priority(movement),
        conveyor_source_priority(movement),
    )
}

fn movement_kind_priority(movement: &StructureMove) -> u8 {
    match movement {
        StructureMove::Translate {
            mark: MovementMark::Vertical,
            ..
        } => 0,
        StructureMove::Rotate { .. } => 1,
        StructureMove::Translate {
            mark: MovementMark::Push,
            ..
        } => 2,
        StructureMove::Translate {
            mark: MovementMark::Conveyor,
            ..
        } => 3,
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

fn expand_structure_movement_plan(
    moves: Vec<StructureMove>,
    world: &WorldBlocks,
    factory_structures: &FactoryStructureState,
) -> Vec<StructureMove> {
    moves
        .into_iter()
        .map(|movement| movement.expanded_for_plan(world, factory_structures))
        .collect()
}

pub(super) fn execute_structure_moves(
    world: &mut WorldBlocks,
    moves: Vec<StructureMove>,
    factory_structures: &mut FactoryStructureState,
) -> HashMap<IVec3, BlockAnimation> {
    let mut influence_cache = MovementInfluenceCache::default();
    execute_structure_moves_with_pushers(world, moves, factory_structures, &mut influence_cache).0
}

pub(super) fn execute_structure_moves_with_pushers(
    world: &mut WorldBlocks,
    moves: Vec<StructureMove>,
    factory_structures: &mut FactoryStructureState,
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
                source,
                ..
            } => {
                if structure.iter().any(|pos| moved.contains(pos)) {
                    continue;
                }
                if let Some(structure) =
                    expanded_move_structure(world, &structure, offset, factory_structures)
                {
                    let before_key = StructureKey::from_structure(&structure);
                    if offset.abs().element_sum() == 1 {
                        for pos in &structure {
                            if let Some(block) = world.blocks.get(pos) {
                                animations.insert(
                                    *pos + offset,
                                    BlockAnimation {
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
                    }
                    if let Some(actor) = actor {
                        let (from_extension, to_extension) = match actor.animation {
                            PusherAnimationKind::Extend => (0.0, 1.0),
                            PusherAnimationKind::Retract => (1.0, 0.0),
                        };
                        pusher_animations.insert(
                            actor.pos,
                            PusherAnimation {
                                duration: 0.0,
                                from_extension,
                                to_extension,
                            },
                        );
                    }
                    moved.extend(structure.iter().copied());
                    move_structure(world, &structure, offset);
                    factory_structures.move_positions(&structure, offset);
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
                    if let Some(source) = source {
                        let target_structure: HashSet<IVec3> = targets.iter().copied().collect();
                        executed.push(ExecutedMovement {
                            before: before_key,
                            after: StructureKey::from_structure(&target_structure),
                            source,
                        });
                    }
                    moved.extend(targets);
                }
            }
        }
    }
    influence_cache.record_executed(executed);
    (animations, pusher_animations)
}

pub(crate) fn material_structure(world: &WorldBlocks, start: IVec3) -> HashSet<IVec3> {
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
    factory_structures: &FactoryStructureState,
) -> bool {
    expanded_move_structure(world, structure, offset, factory_structures).is_some()
}

fn expanded_move_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    offset: IVec3,
    factory_structures: &FactoryStructureState,
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

        let pushed = pushable_structure_at(world, factory_structures, target, offset)?;
        for pushed_pos in pushed {
            if expanded.insert(pushed_pos) {
                queue.push_back(pushed_pos);
            }
        }
    }

    can_move_structure_without_push(world, &expanded, offset).then_some(expanded)
}

fn pushable_structure_at(
    world: &WorldBlocks,
    factory_structures: &FactoryStructureState,
    pos: IVec3,
    offset: IVec3,
) -> Option<HashSet<IVec3>> {
    let block = world.blocks.get(&pos)?;
    if block.kind.is_material() {
        return Some(material_structure(world, pos));
    }
    if block.kind.is_factory() {
        return factory_structures.active_structure_at(pos, offset);
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
