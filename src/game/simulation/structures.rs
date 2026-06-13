use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::game::blocks::{BlockData, MovementRule};
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

#[derive(Clone)]
pub enum StructureMove {
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
pub enum MovementMark {
    Conveyor,
    Push,
    Vertical,
}

#[derive(Clone, Copy, Debug)]
pub struct PusherActor {
    pub(super) pos: IVec3,
    pub(super) animation: PusherAnimationKind,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum PusherAnimationKind {
    Extend,
    Retract,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructureMovePhaseKind {
    Fixed,
    Rotate,
    Push,
    Lift,
    Gravity,
    Conveyor,
}

#[derive(Clone)]
pub struct MovementCandidate {
    pub primary: StructureMove,
    pub fallbacks: Vec<StructureMove>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct StructureKey(Vec<IVec3>);

#[derive(Resource, Default, Clone)]
pub struct MovementInfluenceCache {
    /// Rotators that already rotated this structure; their marks are skipped until contact ends.
    rotator_ignore: HashMap<StructureKey, HashSet<IVec3>>,
    /// Conveyors that marked this structure recently; still-active ones get lower mark priority.
    conveyor_history: HashMap<StructureKey, HashSet<IVec3>>,
}

impl MovementInfluenceCache {
    pub fn clear(&mut self) {
        self.rotator_ignore.clear();
        self.conveyor_history.clear();
    }

    fn ignores_rotator(&self, movement: &StructureMove) -> bool {
        let StructureMove::Rotate {
            structure,
            source: Some(source),
            ..
        } = movement
        else {
            return false;
        };
        self.rotator_ignore
            .get(&StructureKey::from_structure(structure))
            .is_some_and(|sources| sources.contains(source))
    }

    fn conveyor_stale_penalty(&self, movement: &StructureMove) -> u32 {
        let Some(source) = movement.source() else {
            return 0;
        };
        let StructureMove::Translate {
            mark: MovementMark::Conveyor,
            structure,
            ..
        } = movement
        else {
            return 0;
        };
        u32::from(
            self.conveyor_history
                .get(&StructureKey::from_structure(structure))
                .is_some_and(|sources| sources.contains(&source)),
        )
    }

    fn retain_for_turn(
        &mut self,
        active_rotators: &HashMap<StructureKey, HashSet<IVec3>>,
        active_conveyors: &HashMap<StructureKey, HashSet<IVec3>>,
    ) {
        retain_history_sources(&mut self.rotator_ignore, active_rotators);
        retain_history_sources(&mut self.conveyor_history, active_conveyors);
    }

    fn commit_conveyor_marking(
        &mut self,
        active_conveyors: &HashMap<StructureKey, HashSet<IVec3>>,
    ) {
        for (key, sources) in active_conveyors {
            if sources.is_empty() {
                continue;
            }
            self.conveyor_history
                .entry(key.clone())
                .or_default()
                .extend(sources.iter().copied());
        }
    }

    fn migrate_structure_key(&mut self, before: &StructureKey, after: &StructureKey) {
        if before == after {
            return;
        }
        if let Some(rotators) = self.rotator_ignore.remove(before) {
            self.rotator_ignore
                .entry(after.clone())
                .or_default()
                .extend(rotators);
        }
        if let Some(conveyors) = self.conveyor_history.remove(before) {
            self.conveyor_history
                .entry(after.clone())
                .or_default()
                .extend(conveyors);
        }
    }

    pub(super) fn ignores_rotator_movement(&self, movement: &StructureMove) -> bool {
        self.ignores_rotator(movement)
    }

    pub(super) fn conveyor_stale_penalty_for(&self, movement: &StructureMove) -> u32 {
        self.conveyor_stale_penalty(movement)
    }

    pub(super) fn record_structure_migrated(
        &mut self,
        before: &StructureKey,
        after: &StructureKey,
    ) {
        self.migrate_structure_key(before, after);
    }

    pub(super) fn record_rotate_executed(
        &mut self,
        before: &StructureKey,
        after: &StructureKey,
        source: IVec3,
    ) {
        self.migrate_structure_key(before, after);
        self.rotator_ignore
            .entry(after.clone())
            .or_default()
            .insert(source);
    }

    pub(super) fn record_successful_rotate(
        &mut self,
        structure_before: &HashSet<IVec3>,
        structure_after: &HashSet<IVec3>,
        source: IVec3,
    ) {
        let before = StructureKey::from_structure(structure_before);
        let after = StructureKey::from_structure(structure_after);
        self.record_rotate_executed(&before, &after, source);
    }

    pub(super) fn record_successful_translate(
        &mut self,
        structure_before: &HashSet<IVec3>,
        structure_after: &HashSet<IVec3>,
    ) {
        let before = StructureKey::from_structure(structure_before);
        let after = StructureKey::from_structure(structure_after);
        self.record_structure_migrated(&before, &after);
    }

    pub(super) fn begin_turn_from_candidates(
        &mut self,
        rotate: &[MovementCandidate],
        conveyor: &[MovementCandidate],
    ) {
        let (active_rotators, active_conveyors) = active_sources_from_candidates(rotate, conveyor);
        self.retain_for_turn(&active_rotators, &active_conveyors);
    }

    pub(super) fn finish_turn_from_candidates(&mut self, conveyor: &[MovementCandidate]) {
        let (_, active_conveyors) = active_sources_from_candidates(&[], conveyor);
        self.commit_conveyor_marking(&active_conveyors);
    }

    pub(super) fn compare_conveyor_candidates(
        &self,
        a: &MovementCandidate,
        b: &MovementCandidate,
    ) -> std::cmp::Ordering {
        conveyor_candidate_priority(a, self).cmp(&conveyor_candidate_priority(b, self))
    }

    #[cfg(test)]
    pub(super) fn rotator_is_ignored(&self, movement: &StructureMove) -> bool {
        self.ignores_rotator(movement)
    }

    #[cfg(test)]
    pub(super) fn conveyor_penalty(&self, movement: &StructureMove) -> u32 {
        self.conveyor_stale_penalty(movement)
    }
}

fn retain_history_sources(
    history: &mut HashMap<StructureKey, HashSet<IVec3>>,
    active: &HashMap<StructureKey, HashSet<IVec3>>,
) {
    history.retain(|structure, sources| {
        let Some(active_sources) = active.get(structure) else {
            return false;
        };
        sources.retain(|source| active_sources.contains(source));
        !sources.is_empty()
    });
}

fn active_sources_from_candidates(
    rotate: &[MovementCandidate],
    conveyor: &[MovementCandidate],
) -> (
    HashMap<StructureKey, HashSet<IVec3>>,
    HashMap<StructureKey, HashSet<IVec3>>,
) {
    let mut rotators: HashMap<StructureKey, HashSet<IVec3>> = HashMap::new();
    for candidate in rotate {
        let Some(source) = candidate.primary.source() else {
            continue;
        };
        rotators
            .entry(StructureKey::from_structure(candidate.primary.structure()))
            .or_default()
            .insert(source);
    }
    let mut conveyors: HashMap<StructureKey, HashSet<IVec3>> = HashMap::new();
    for candidate in conveyor {
        let Some(source) = candidate.primary.source() else {
            continue;
        };
        conveyors
            .entry(StructureKey::from_structure(candidate.primary.structure()))
            .or_default()
            .insert(source);
    }
    (rotators, conveyors)
}

fn conveyor_candidate_priority(
    candidate: &MovementCandidate,
    influence: &MovementInfluenceCache,
) -> (u32, i32, i32, i32) {
    (
        influence.conveyor_stale_penalty_for(&candidate.primary),
        candidate
            .primary
            .source()
            .map(|pos| pos.x)
            .unwrap_or(i32::MIN),
        candidate
            .primary
            .source()
            .map(|pos| pos.y)
            .unwrap_or(i32::MIN),
        candidate
            .primary
            .source()
            .map(|pos| pos.z)
            .unwrap_or(i32::MIN),
    )
}

impl StructureKey {
    fn from_structure(structure: &HashSet<IVec3>) -> Self {
        let mut positions: Vec<IVec3> = structure.iter().copied().collect();
        positions.sort_by_key(|pos| (pos.x, pos.y, pos.z));
        Self(positions)
    }
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

    pub(super) fn source(&self) -> Option<IVec3> {
        match self {
            Self::Translate { source, .. } | Self::Rotate { source, .. } => *source,
        }
    }

    pub(super) fn pusher_actor(&self) -> Option<IVec3> {
        match self {
            Self::Translate { actor, .. } => actor.map(|actor| actor.pos),
            Self::Rotate { .. } => None,
        }
    }

    pub(super) fn structure(&self) -> &HashSet<IVec3> {
        match self {
            Self::Translate { structure, .. } | Self::Rotate { structure, .. } => structure,
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

pub(super) fn expanded_move_structure(
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
pub(super) enum MovementExpansionMode {
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

pub(super) fn rotate_facing_internal(facing: Facing, clockwise: bool) -> Facing {
    rotate_facing(facing, clockwise)
}

pub(super) fn movement_expansion_mode_public(
    mark: MovementMark,
    source: Option<IVec3>,
) -> MovementExpansionMode {
    movement_expansion_mode(mark, source)
}

pub(super) fn hard_pusher_head_blocks_move_public(
    structure: &HashSet<IVec3>,
    offset: IVec3,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> bool {
    hard_pusher_head_blocks_move(structure, offset, hard_pusher_head_occupancy)
}

pub(super) fn hard_pusher_head_blocked_below_public(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> bool {
    hard_pusher_head_blocked_below(world, structure, hard_pusher_head_occupancy)
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
mod movement_history_tests {
    use super::*;
    use bevy::prelude::IVec3;
    use std::collections::HashSet;

    #[test]
    fn rotator_history_skips_repeat_mark_until_contact_ends() {
        let structure = HashSet::from([IVec3::new(0, 2, 0)]);
        let rotator = IVec3::new(1, 1, 0);
        let movement = StructureMove::rotate(structure.clone(), rotator, true).with_source(rotator);
        let mut cache = MovementInfluenceCache::default();

        assert!(!cache.rotator_is_ignored(&movement));
        cache.record_successful_rotate(&structure, &HashSet::from([IVec3::new(0, 2, 1)]), rotator);
        let movement_after_rotate =
            StructureMove::rotate(HashSet::from([IVec3::new(0, 2, 1)]), rotator, true)
                .with_source(rotator);
        assert!(cache.rotator_is_ignored(&movement_after_rotate));
    }

    #[test]
    fn conveyor_history_deprioritizes_stale_source() {
        let structure = HashSet::from([IVec3::ZERO]);
        let stale =
            StructureMove::translate_marked(structure.clone(), IVec3::X, MovementMark::Conveyor)
                .with_source(IVec3::new(-1, 0, 0));
        let fresh = StructureMove::translate_marked(structure, IVec3::X, MovementMark::Conveyor)
            .with_source(IVec3::new(1, 0, 0));
        let mut cache = MovementInfluenceCache::default();
        cache.begin_turn_from_candidates(
            &[],
            &[MovementCandidate {
                primary: stale.clone(),
                fallbacks: Vec::new(),
            }],
        );
        cache.finish_turn_from_candidates(&[MovementCandidate {
            primary: stale,
            fallbacks: Vec::new(),
        }]);

        assert_eq!(cache.conveyor_penalty(&fresh), 0);
        assert_eq!(
            cache.conveyor_penalty(
                &StructureMove::translate_marked(
                    HashSet::from([IVec3::ZERO]),
                    IVec3::X,
                    MovementMark::Conveyor,
                )
                .with_source(IVec3::new(-1, 0, 0))
            ),
            1
        );
    }
}
