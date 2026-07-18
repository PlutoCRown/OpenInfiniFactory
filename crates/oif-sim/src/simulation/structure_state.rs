use glam::IVec3;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::blocks::{AcceptorId, BlockId, MovementRule};
use crate::world::grid::WorldBlocks;

use super::signal_offsets;

/// 结构运行时 ID：开局/焊接重建时分配，成员移动时保持不变
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct StructureId(pub u64);

impl StructureId {
    pub const NONE: Self = Self(0);

    pub const fn is_none(self) -> bool {
        self.0 == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactoryActivity {
    Active,
    Inactive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructureFreedom {
    None,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructureKind {
    Material,
    Factory,
}

impl StructureFreedom {
    pub fn can_translate(self, _offset: IVec3) -> bool {
        self == Self::All
    }
}

/// Relative contact from a structure member toward a supporting cell.
pub type GravitySupportContact = (IVec3, IVec3);

#[derive(Clone)]
pub struct Structure {
    pub id: StructureId,
    pub kind: StructureKind,
    pub positions: HashSet<IVec3>,
    pub activity: FactoryActivity,
    pub freedom: StructureFreedom,
    pub pushable: bool,
    gravity_support: Vec<GravitySupportContact>,
}

#[derive(Clone, Debug)]
pub struct AcceptorStructure {
    pub id: AcceptorId,
    pub positions: HashSet<IVec3>,
    pub count: u32,
}

#[derive(Default, Clone)]
pub struct StructureState {
    structures: HashMap<StructureId, Structure>,
    structure_by_pos: HashMap<IVec3, StructureId>,
    next_structure_id: u64,
    acceptor_structures: Vec<AcceptorStructure>,
}

impl StructureState {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn is_empty(&self) -> bool {
        self.structure_by_pos.is_empty()
    }

    fn alloc_id(&mut self) -> StructureId {
        self.next_structure_id += 1;
        StructureId(self.next_structure_id)
    }

    /// Build factory structures (connectivity + activity), acceptor structures, and material structures (welds).
    /// 工厂连通按当前世界相邻关系建一次（开局/放置快照）；之后运行时只搬迁成员，不因贴上而合并。
    pub fn rebuild_for_simulation(&mut self, world: &WorldBlocks) {
        *self = Self::default();
        self.append_factory_structures(world);
        self.apply_factory_inactive_propagation(world);
        self.append_acceptor_structures(world);
        self.append_material_structures(world, &HashMap::new(), &HashMap::new());
    }

    pub fn acceptor_structures(&self) -> &[AcceptorStructure] {
        &self.acceptor_structures
    }

    pub fn increment_acceptor_count(&mut self, index: usize) {
        if let Some(structure) = self.acceptor_structures.get_mut(index) {
            structure.count = structure.count.saturating_add(1);
        }
    }

    /// Debug-only factory connectivity for edit-mode visualization.
    pub fn rebuild_factory_for_debug(&mut self, world: &WorldBlocks) {
        self.retain_factory_only();
        self.append_factory_structures(world);
        self.apply_factory_inactive_propagation(world);
    }

    pub fn refresh_material_structures(&mut self, world: &WorldBlocks) {
        // 按成员 BlockId 集合复用 StructureId，避免影响计数每回合清零
        let mut previous_ids: HashMap<Vec<u64>, StructureId> = HashMap::new();
        let mut previous_support: HashMap<StructureId, Vec<GravitySupportContact>> = HashMap::new();
        for (id, structure) in &self.structures {
            if structure.kind != StructureKind::Material {
                continue;
            }
            let mut members: Vec<u64> = structure
                .positions
                .iter()
                .filter_map(|pos| world.blocks.get(pos).map(|block| block.id.0))
                .collect();
            members.sort_unstable();
            previous_ids.insert(members, *id);
            previous_support.insert(*id, structure.gravity_support.clone());
        }
        self.retain_factory_only();
        self.append_material_structures(world, &previous_ids, &previous_support);
    }

    fn retain_factory_only(&mut self) {
        self.structures
            .retain(|_, structure| structure.kind == StructureKind::Factory);
        self.structure_by_pos.clear();
        for (id, structure) in &self.structures {
            for pos in &structure.positions {
                self.structure_by_pos.insert(*pos, *id);
            }
        }
    }

    fn append_factory_structures(&mut self, world: &WorldBlocks) {
        let starts: Vec<IVec3> = world
            .blocks
            .iter()
            .filter_map(|(pos, block)| block.kind.is_factory().then_some(*pos))
            .collect();
        self.append_connected_factory_structures(world, starts);
    }

    fn append_acceptor_structures(&mut self, world: &WorldBlocks) {
        for stored in &world.acceptor_structures {
            self.acceptor_structures.push(AcceptorStructure {
                id: stored.id,
                positions: stored.positions.iter().copied().collect(),
                count: 0,
            });
        }
    }

    fn append_material_structures(
        &mut self,
        world: &WorldBlocks,
        previous_ids: &HashMap<Vec<u64>, StructureId>,
        previous_support: &HashMap<StructureId, Vec<GravitySupportContact>>,
    ) {
        let mut handled = self
            .structure_by_pos
            .keys()
            .copied()
            .collect::<HashSet<_>>();
        let mut starts: Vec<IVec3> = world
            .blocks
            .iter()
            .filter_map(|(pos, block)| block.kind.is_material().then_some(*pos))
            .collect();
        starts.sort_by_key(|pos| (pos.x, pos.y, pos.z));

        for start in starts {
            if handled.contains(&start) || !world.is_material_at(start) {
                continue;
            }
            let positions = material_structure(world, start);
            let mut members: Vec<u64> = positions
                .iter()
                .filter_map(|pos| world.blocks.get(pos).map(|block| block.id.0))
                .collect();
            members.sort_unstable();
            let id = previous_ids
                .get(&members)
                .copied()
                .unwrap_or_else(|| self.alloc_id());
            let gravity_support = previous_support.get(&id).cloned().unwrap_or_default();
            for pos in &positions {
                handled.insert(*pos);
                self.structure_by_pos.insert(*pos, id);
            }
            self.structures.insert(
                id,
                Structure {
                    id,
                    kind: StructureKind::Material,
                    positions,
                    activity: FactoryActivity::Active,
                    freedom: StructureFreedom::All,
                    pushable: true,
                    gravity_support,
                },
            );
        }
    }

    fn append_connected_factory_structures(
        &mut self,
        world: &WorldBlocks,
        starts: impl IntoIterator<Item = IVec3>,
    ) {
        let mut handled: HashSet<IVec3> = self.structure_by_pos.keys().copied().collect();
        let mut starts: Vec<IVec3> = starts
            .into_iter()
            .filter(|pos| world.is_factory_at(*pos) && !handled.contains(pos))
            .collect();
        starts.sort_by_key(|pos| (pos.x, pos.y, pos.z));

        for start in starts {
            if handled.contains(&start) || !world.is_factory_at(start) {
                continue;
            }

            let positions = factory_structure(world, start);
            let id = self.alloc_id();
            for pos in &positions {
                handled.insert(*pos);
                self.structure_by_pos.insert(*pos, id);
            }
            self.structures.insert(
                id,
                Structure {
                    id,
                    kind: StructureKind::Factory,
                    positions,
                    activity: FactoryActivity::Active,
                    freedom: StructureFreedom::All,
                    pushable: true,
                    gravity_support: Vec::new(),
                },
            );
        }
    }

    fn apply_factory_inactive_propagation(&mut self, world: &WorldBlocks) {
        let factory_ids: Vec<StructureId> = self
            .structures
            .iter()
            .filter(|(_, structure)| structure.kind == StructureKind::Factory)
            .map(|(id, _)| *id)
            .collect();
        let scene_anchored: HashMap<StructureId, bool> = factory_ids
            .iter()
            .map(|id| {
                let anchored = self
                    .structures
                    .get(id)
                    .is_some_and(|structure| touches_scene(world, &structure.positions));
                (*id, anchored)
            })
            .collect();
        let mut inactive: HashMap<StructureId, bool> = scene_anchored.clone();
        let mut queue = VecDeque::new();
        for (id, anchored) in &scene_anchored {
            if *anchored {
                inactive.insert(*id, true);
                queue.push_back(*id);
            }
        }

        while let Some(id) = queue.pop_front() {
            let Some(structure) = self.structures.get(&id) else {
                continue;
            };
            if structure.kind != StructureKind::Factory {
                continue;
            }
            for pos in structure.positions.clone() {
                for offset in signal_offsets() {
                    let neighbor = pos + offset;
                    let Some(neighbor_id) = self.structure_by_pos.get(&neighbor).copied() else {
                        continue;
                    };
                    let Some(neighbor_structure) = self.structures.get(&neighbor_id) else {
                        continue;
                    };
                    if neighbor_structure.kind != StructureKind::Factory {
                        continue;
                    }
                    if is_blocked_factory_connection(world, pos, neighbor)
                        || is_blocked_factory_connection(world, neighbor, pos)
                    {
                        continue;
                    }
                    if !inactive.get(&neighbor_id).copied().unwrap_or(false) {
                        inactive.insert(neighbor_id, true);
                        queue.push_back(neighbor_id);
                    }
                }
            }
        }

        for id in factory_ids {
            let Some(structure) = self.structures.get_mut(&id) else {
                continue;
            };
            structure.activity = FactoryActivity::Active;
            structure.freedom = StructureFreedom::All;
            structure.pushable = true;
            if inactive.get(&id).copied().unwrap_or(false) {
                structure.activity = FactoryActivity::Inactive;
                if scene_anchored.get(&id).copied().unwrap_or(false) {
                    structure.freedom = StructureFreedom::None;
                    structure.pushable = false;
                }
            }
        }
    }

    pub fn structure_ids(&self) -> impl Iterator<Item = StructureId> + '_ {
        self.structures.keys().copied()
    }

    pub fn activity_at(&self, pos: IVec3) -> Option<FactoryActivity> {
        Some(self.structure(pos)?.activity)
    }

    pub fn id_at(&self, pos: IVec3) -> Option<StructureId> {
        self.structure_by_pos.get(&pos).copied()
    }

    pub fn pushable_structure_at(&self, pos: IVec3, offset: IVec3) -> Option<HashSet<IVec3>> {
        let structure = self.structure(pos)?;
        if !structure.pushable || !structure.freedom.can_translate(offset) {
            return None;
        }
        Some(structure.positions.clone())
    }

    pub fn active_structure_at(&self, pos: IVec3, offset: IVec3) -> Option<HashSet<IVec3>> {
        self.pushable_structure_at(pos, offset)
    }

    pub fn pusher_target_structure(
        &self,
        world: &WorldBlocks,
        pusher_pos: IVec3,
        target_pos: IVec3,
        offset: IVec3,
    ) -> Option<HashSet<IVec3>> {
        let target = self.structure(target_pos)?;
        if target.kind != StructureKind::Factory {
            return None;
        }
        let structure =
            connected_subset_with_blocked_edge(world, target, target_pos, Some(pusher_pos));
        if structure.contains(&pusher_pos) || !factory_subset_pushable(world, &structure, offset) {
            return None;
        }
        Some(structure)
    }

    pub fn falling_structure_at(
        &self,
        pos: IVec3,
        offset: IVec3,
    ) -> Option<(StructureId, HashSet<IVec3>)> {
        let id = *self.structure_by_pos.get(&pos)?;
        let structure = self.structures.get(&id)?;
        if structure.activity != FactoryActivity::Active || !structure.freedom.can_translate(offset)
        {
            return None;
        }
        Some((id, structure.positions.clone()))
    }

    pub fn structure_id_at(&self, pos: IVec3) -> Option<StructureId> {
        self.structure_by_pos.get(&pos).copied()
    }

    pub fn structure_positions(&self, id: StructureId) -> Option<&HashSet<IVec3>> {
        self.structures
            .get(&id)
            .map(|structure| &structure.positions)
    }

    pub fn gravity_support_valid(
        &self,
        id: StructureId,
        world: &WorldBlocks,
        hard_pusher_head_occupancy: &HashSet<IVec3>,
    ) -> bool {
        let Some(structure) = self.structures.get(&id) else {
            return false;
        };
        let contacts = &structure.gravity_support;
        !contacts.is_empty()
            && contacts.iter().any(|(member, dir)| {
                structure.positions.contains(member) && {
                    let support = *member + *dir;
                    support.y >= 0
                        && !structure.positions.contains(&support)
                        && (!world.can_move_into_yielding_fragile(support)
                            || hard_pusher_head_occupancy.contains(&support))
                }
            })
    }

    pub fn record_gravity_support(
        &mut self,
        id: StructureId,
        world: &WorldBlocks,
        hard_pusher_head_occupancy: &HashSet<IVec3>,
    ) {
        let Some(structure) = self.structures.get_mut(&id) else {
            return;
        };
        structure.gravity_support =
            collect_gravity_support(world, &structure.positions, hard_pusher_head_occupancy);
    }

    pub fn clear_gravity_support(&mut self, id: StructureId) {
        if let Some(structure) = self.structures.get_mut(&id) {
            structure.gravity_support.clear();
        }
    }

    pub fn move_positions(&mut self, positions: &HashSet<IVec3>, offset: IVec3) {
        let mut changed_ids = HashSet::new();
        for pos in positions {
            if let Some(id) = self.structure_by_pos.get(pos).copied() {
                changed_ids.insert(id);
            }
        }
        for id in &changed_ids {
            let Some(structure) = self.structures.get(id) else {
                continue;
            };
            for pos in &structure.positions {
                self.structure_by_pos.remove(pos);
            }
        }
        for id in changed_ids {
            let Some(structure) = self.structures.get_mut(&id) else {
                continue;
            };
            structure.positions = structure
                .positions
                .iter()
                .map(|pos| {
                    if positions.contains(pos) {
                        *pos + offset
                    } else {
                        *pos
                    }
                })
                .collect();
            for (member, _dir) in &mut structure.gravity_support {
                if positions.contains(member) {
                    *member += offset;
                }
            }
            for pos in &structure.positions {
                self.structure_by_pos.insert(*pos, id);
            }
        }
    }

    pub fn replace_structure_positions(
        &mut self,
        old_positions: &HashSet<IVec3>,
        new_positions: HashSet<IVec3>,
    ) {
        let Some(&id) = old_positions
            .iter()
            .find_map(|pos| self.structure_by_pos.get(pos))
        else {
            return;
        };
        for pos in old_positions {
            self.structure_by_pos.remove(pos);
        }
        let Some(structure) = self.structures.get_mut(&id) else {
            return;
        };
        structure.positions = new_positions;
        structure.gravity_support.clear();
        for pos in &structure.positions {
            self.structure_by_pos.insert(*pos, id);
        }
    }

    pub fn movable_structure_at(&self, pos: IVec3) -> Option<HashSet<IVec3>> {
        let structure = self.structure(pos)?;
        if structure.activity != FactoryActivity::Active
            || structure.freedom == StructureFreedom::None
        {
            return None;
        }
        Some(structure.positions.clone())
    }

    pub fn freedom_at(&self, pos: IVec3) -> Option<StructureFreedom> {
        Some(self.structure(pos)?.freedom)
    }

    pub fn kind_at(&self, pos: IVec3) -> Option<StructureKind> {
        Some(self.structure(pos)?.kind)
    }

    pub fn structure_contains(&self, pos: IVec3, candidate: IVec3) -> bool {
        self.structure(pos)
            .is_some_and(|structure| structure.positions.contains(&candidate))
    }

    pub fn gravity_structure_ids(&self) -> Vec<StructureId> {
        let mut ids: Vec<StructureId> = self
            .structures
            .iter()
            .filter(|(_, structure)| structure.activity == FactoryActivity::Active)
            .map(|(id, _)| *id)
            .collect();
        ids.sort_by_key(|id| {
            self.structures
                .get(id)
                .and_then(|structure| structure.positions.iter().map(|pos| pos.y).min())
                .unwrap_or(0)
        });
        ids
    }

    fn structure(&self, pos: IVec3) -> Option<&Structure> {
        self.structure_by_pos
            .get(&pos)
            .and_then(|id| self.structures.get(id))
    }

    pub(super) fn structure_by_id(&self, id: StructureId) -> Option<&Structure> {
        self.structures.get(&id)
    }
}

pub fn material_structure(world: &WorldBlocks, start: IVec3) -> HashSet<IVec3> {
    let Some(start_id) = world
        .blocks
        .get(&start)
        .filter(|block| block.kind.is_material() && !block.id.is_none())
        .map(|block| block.id)
    else {
        return HashSet::new();
    };
    let id_to_pos: HashMap<BlockId, IVec3> = world
        .blocks
        .iter()
        .filter(|(_, block)| block.kind.is_material() && !block.id.is_none())
        .map(|(pos, block)| (block.id, *pos))
        .collect();

    let mut structure = HashSet::new();
    let mut seen_ids = HashSet::from([start_id]);
    let mut queue = VecDeque::from([start_id]);
    structure.insert(start);

    while let Some(id) = queue.pop_front() {
        for weld in &world.material_welds {
            let Some(other_id) = weld.other(id) else {
                continue;
            };
            if !seen_ids.insert(other_id) {
                continue;
            }
            let Some(&neighbor) = id_to_pos.get(&other_id) else {
                continue;
            };
            if !world.is_material_at(neighbor) {
                continue;
            }
            structure.insert(neighbor);
            queue.push_back(other_id);
        }
        // 附着边：父↔子双向并入同一材料结构
        for (child_id, att) in &world.material_attachments {
            let other_id = if *child_id == id {
                att.parent
            } else if att.parent == id {
                *child_id
            } else {
                continue;
            };
            if !seen_ids.insert(other_id) {
                continue;
            }
            let Some(&neighbor) = id_to_pos.get(&other_id) else {
                continue;
            };
            structure.insert(neighbor);
            queue.push_back(other_id);
        }
    }

    structure
}

pub fn query_factory_structure(world: &WorldBlocks, pos: IVec3) -> Option<HashSet<IVec3>> {
    world
        .is_factory_at(pos)
        .then(|| factory_structure(world, pos))
}

fn collect_gravity_support(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> Vec<GravitySupportContact> {
    structure
        .iter()
        .filter_map(|pos| {
            let below = *pos + IVec3::NEG_Y;
            (below.y >= 0
                && !structure.contains(&below)
                && (!world.can_move_into_yielding_fragile(below)
                    || hard_pusher_head_occupancy.contains(&below)))
            .then_some((*pos, IVec3::NEG_Y))
        })
        .collect()
}

fn factory_structure(world: &WorldBlocks, start: IVec3) -> HashSet<IVec3> {
    factory_structure_with_blocked_edge(world, start, None)
}

fn factory_structure_with_blocked_edge(
    world: &WorldBlocks,
    start: IVec3,
    blocked_pusher_pos: Option<IVec3>,
) -> HashSet<IVec3> {
    let allowed: HashSet<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| block.kind.is_factory().then_some(*pos))
        .collect();
    connected_factory_subset(world, &allowed, start, blocked_pusher_pos)
}

fn connected_subset_with_blocked_edge(
    world: &WorldBlocks,
    structure: &Structure,
    start: IVec3,
    blocked_pusher_pos: Option<IVec3>,
) -> HashSet<IVec3> {
    connected_factory_subset(world, &structure.positions, start, blocked_pusher_pos)
}

fn connected_factory_subset(
    world: &WorldBlocks,
    allowed: &HashSet<IVec3>,
    start: IVec3,
    blocked_pusher_pos: Option<IVec3>,
) -> HashSet<IVec3> {
    let mut structure = HashSet::new();
    let mut queue = VecDeque::from([start]);
    structure.insert(start);

    while let Some(pos) = queue.pop_front() {
        for offset in signal_offsets() {
            let neighbor = pos + offset;
            if structure.contains(&neighbor)
                || !allowed.contains(&neighbor)
                || is_blocked_pusher_edge(world, blocked_pusher_pos, pos, neighbor)
                || is_blocked_factory_connection(world, pos, neighbor)
                || is_blocked_factory_connection(world, neighbor, pos)
            {
                continue;
            }
            structure.insert(neighbor);
            queue.push_back(neighbor);
        }
    }

    structure
}

fn is_blocked_pusher_edge(
    world: &WorldBlocks,
    pusher_pos: Option<IVec3>,
    from: IVec3,
    to: IVec3,
) -> bool {
    let Some(pusher_pos) = pusher_pos else {
        return false;
    };
    pusher_front_neighbor(world, pusher_pos).is_some_and(|front| {
        (from == pusher_pos && to == front) || (from == front && to == pusher_pos)
    })
}

fn is_blocked_factory_connection(world: &WorldBlocks, from: IVec3, to: IVec3) -> bool {
    world
        .blocks
        .get(&from)
        .is_some_and(|block| block.kind.non_connection_face(block.facing) == Some(to - from))
}

fn pusher_front_neighbor(world: &WorldBlocks, pos: IVec3) -> Option<IVec3> {
    world.blocks.get(&pos).and_then(|block| {
        matches!(
            block.kind.movement_rule(block.facing),
            Some(MovementRule::PoweredTranslate { .. })
        )
        .then_some(pos + block.facing.forward_ivec3())
    })
}

fn touches_scene(world: &WorldBlocks, structure: &HashSet<IVec3>) -> bool {
    structure.iter().any(|pos| {
        signal_offsets().into_iter().any(|offset| {
            let neighbor = *pos + offset;
            world.is_scene_at(neighbor) && !is_blocked_factory_connection(world, *pos, neighbor)
        })
    })
}

fn factory_subset_pushable(world: &WorldBlocks, subset: &HashSet<IVec3>, offset: IVec3) -> bool {
    !subset.is_empty()
        && !touches_scene(world, subset)
        && StructureFreedom::All.can_translate(offset)
}

