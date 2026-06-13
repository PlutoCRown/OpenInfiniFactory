use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::game::blocks::BlockKind;
use crate::game::world::grid::WorldBlocks;

use super::signal_offsets;

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
    pub kind: StructureKind,
    pub positions: HashSet<IVec3>,
    pub activity: FactoryActivity,
    pub freedom: StructureFreedom,
    pub pushable: bool,
    gravity_support: Vec<GravitySupportContact>,
}

#[derive(Clone, Debug)]
pub struct AcceptorStructure {
    pub positions: HashSet<IVec3>,
    pub count: u32,
}

#[derive(Resource, Default, Clone)]
pub struct StructureState {
    structures: Vec<Structure>,
    structure_by_pos: HashMap<IVec3, usize>,
    acceptor_structures: Vec<AcceptorStructure>,
}

impl StructureState {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn is_empty(&self) -> bool {
        self.structure_by_pos.is_empty()
    }

    /// Build factory structures (connectivity + activity), acceptor structures, and material structures (welds).
    pub fn rebuild_for_simulation(&mut self, world: &WorldBlocks) {
        *self = Self::default();
        self.append_factory_structures(world);
        self.apply_factory_inactive_propagation(world);
        self.append_acceptor_structures(world);
        self.append_material_structures(world);
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
        self.retain_factory_only();
        self.append_material_structures(world);
    }

    fn retain_factory_only(&mut self) {
        let factory_structures: Vec<Structure> = self
            .structures
            .iter()
            .filter(|structure| structure.kind == StructureKind::Factory)
            .cloned()
            .collect();
        self.structures = factory_structures;
        self.structure_by_pos.clear();
        for (index, structure) in self.structures.iter().enumerate() {
            for pos in &structure.positions {
                self.structure_by_pos.insert(*pos, index);
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
        let mut handled = HashSet::new();
        let mut starts: Vec<IVec3> = world
            .system_blocks
            .iter()
            .filter_map(|(pos, block)| (block.kind == BlockKind::Goal).then_some(*pos))
            .collect();
        starts.sort_by_key(|pos| (pos.x, pos.y, pos.z));

        for start in starts {
            if handled.contains(&start) {
                continue;
            }
            let positions = acceptor_structure(world, start);
            handled.extend(positions.iter().copied());
            self.acceptor_structures.push(AcceptorStructure {
                positions,
                count: 0,
            });
        }
    }

    fn append_material_structures(&mut self, world: &WorldBlocks) {
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
            let index = self.structures.len();
            for pos in &positions {
                handled.insert(*pos);
                self.structure_by_pos.insert(*pos, index);
            }
            self.structures.push(Structure {
                kind: StructureKind::Material,
                positions,
                activity: FactoryActivity::Active,
                freedom: StructureFreedom::All,
                pushable: true,
                gravity_support: Vec::new(),
            });
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
            let index = self.structures.len();
            for pos in &positions {
                handled.insert(*pos);
                self.structure_by_pos.insert(*pos, index);
            }
            self.structures.push(Structure {
                kind: StructureKind::Factory,
                positions,
                activity: FactoryActivity::Active,
                freedom: StructureFreedom::All,
                pushable: true,
                gravity_support: Vec::new(),
            });
        }
    }

    fn apply_factory_inactive_propagation(&mut self, world: &WorldBlocks) {
        let scene_anchored: Vec<bool> = self
            .structures
            .iter()
            .map(|structure| {
                structure.kind == StructureKind::Factory
                    && touches_scene(world, &structure.positions)
            })
            .collect();
        let mut inactive = scene_anchored.clone();
        let mut queue = VecDeque::new();
        for (index, anchored) in scene_anchored.iter().copied().enumerate() {
            if anchored {
                inactive[index] = true;
                queue.push_back(index);
            }
        }

        while let Some(index) = queue.pop_front() {
            if self.structures[index].kind != StructureKind::Factory {
                continue;
            }
            for pos in &self.structures[index].positions.clone() {
                for offset in signal_offsets() {
                    let neighbor = *pos + offset;
                    let Some(neighbor_index) = self.structure_by_pos.get(&neighbor).copied() else {
                        continue;
                    };
                    if self.structures[neighbor_index].kind != StructureKind::Factory {
                        continue;
                    }
                    if is_blocked_factory_connection(world, *pos, neighbor)
                        || is_blocked_factory_connection(world, neighbor, *pos)
                    {
                        continue;
                    }
                    if !inactive[neighbor_index] {
                        inactive[neighbor_index] = true;
                        queue.push_back(neighbor_index);
                    }
                }
            }
        }

        for (index, structure) in self.structures.iter_mut().enumerate() {
            if structure.kind != StructureKind::Factory {
                continue;
            }
            structure.activity = FactoryActivity::Active;
            structure.freedom = StructureFreedom::All;
            structure.pushable = true;
            if inactive[index] {
                structure.activity = FactoryActivity::Inactive;
                if scene_anchored[index] {
                    structure.freedom = StructureFreedom::None;
                    structure.pushable = false;
                }
            }
        }
    }

    pub fn activity_at(&self, pos: IVec3) -> Option<FactoryActivity> {
        Some(self.structure(pos)?.activity)
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
        solution: &WorldBlocks,
        factory_registry: &crate::game::world::factory_registry::FactoryBlockRegistry,
        pusher_pos: IVec3,
        target_pos: IVec3,
        offset: IVec3,
    ) -> Option<HashSet<IVec3>> {
        if !factory_registry.has_turn_factory(target_pos) {
            return None;
        }
        let solution_target = factory_registry.turn_to_solution_pos(target_pos)?;
        let solution_pusher = factory_registry.turn_to_solution_pos(pusher_pos)?;
        let structure_delta = target_pos - solution_target;
        let structure =
            factory_structure_with_blocked_edge(solution, solution_target, Some(solution_pusher));
        if structure.is_empty() || !factory_subset_pushable(solution, &structure, offset) {
            return None;
        }
        Some(structure.iter().map(|pos| *pos + structure_delta).collect())
    }

    pub fn falling_structure_at(
        &self,
        pos: IVec3,
        offset: IVec3,
    ) -> Option<(usize, HashSet<IVec3>)> {
        let index = *self.structure_by_pos.get(&pos)?;
        let structure = self.structures.get(index)?;
        if structure.activity != FactoryActivity::Active || !structure.freedom.can_translate(offset)
        {
            return None;
        }
        Some((index, structure.positions.clone()))
    }

    pub fn structure_index_at(&self, pos: IVec3) -> Option<usize> {
        self.structure_by_pos.get(&pos).copied()
    }

    pub fn structure_positions(&self, index: usize) -> Option<&HashSet<IVec3>> {
        self.structures
            .get(index)
            .map(|structure| &structure.positions)
    }

    pub fn gravity_support_valid(&self, index: usize, world: &WorldBlocks) -> bool {
        let Some(structure) = self.structures.get(index) else {
            return false;
        };
        let contacts = &structure.gravity_support;
        !contacts.is_empty()
            && contacts.iter().any(|(member, dir)| {
                structure.positions.contains(member) && {
                    let support = *member + *dir;
                    support.y >= 0
                        && !structure.positions.contains(&support)
                        && !world.can_move_into(support)
                }
            })
    }

    pub fn record_gravity_support(&mut self, index: usize, world: &WorldBlocks) {
        let Some(structure) = self.structures.get_mut(index) else {
            return;
        };
        structure.gravity_support = collect_gravity_support(world, &structure.positions);
    }

    pub fn clear_gravity_support(&mut self, index: usize) {
        if let Some(structure) = self.structures.get_mut(index) {
            structure.gravity_support.clear();
        }
    }

    pub fn move_positions(&mut self, positions: &HashSet<IVec3>, offset: IVec3) {
        let mut changed_indices = HashSet::new();
        for pos in positions {
            if let Some(index) = self.structure_by_pos.get(pos).copied() {
                changed_indices.insert(index);
            }
        }
        for index in &changed_indices {
            let Some(structure) = self.structures.get(*index) else {
                continue;
            };
            for pos in &structure.positions {
                self.structure_by_pos.remove(pos);
            }
        }
        for index in changed_indices {
            let Some(structure) = self.structures.get_mut(index) else {
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
                self.structure_by_pos.insert(*pos, index);
            }
        }
    }

    pub fn replace_structure_positions(
        &mut self,
        old_positions: &HashSet<IVec3>,
        new_positions: HashSet<IVec3>,
    ) {
        let Some(&index) = old_positions
            .iter()
            .find_map(|pos| self.structure_by_pos.get(pos))
        else {
            return;
        };
        for pos in old_positions {
            self.structure_by_pos.remove(pos);
        }
        let Some(structure) = self.structures.get_mut(index) else {
            return;
        };
        structure.positions = new_positions;
        structure.gravity_support.clear();
        for pos in &structure.positions {
            self.structure_by_pos.insert(*pos, index);
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

    pub fn pushable_at(&self, pos: IVec3) -> Option<bool> {
        Some(self.structure(pos)?.pushable)
    }

    pub fn member_count_at(&self, pos: IVec3) -> Option<usize> {
        Some(self.structure(pos)?.positions.len())
    }

    pub fn positions_at(&self, pos: IVec3) -> Option<HashSet<IVec3>> {
        Some(self.structure(pos)?.positions.clone())
    }

    pub fn structure_contains(&self, pos: IVec3, candidate: IVec3) -> bool {
        self.structure(pos)
            .is_some_and(|structure| structure.positions.contains(&candidate))
    }

    pub fn gravity_structure_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self
            .structures
            .iter()
            .enumerate()
            .filter(|(_, structure)| structure.activity == FactoryActivity::Active)
            .map(|(index, _)| index)
            .collect();
        indices.sort_by_key(|index| {
            self.structures[*index]
                .positions
                .iter()
                .map(|pos| pos.y)
                .min()
                .unwrap_or(0)
        });
        indices
    }

    fn structure(&self, pos: IVec3) -> Option<&Structure> {
        self.structure_by_pos
            .get(&pos)
            .and_then(|index| self.structures.get(*index))
    }
}

pub fn acceptor_structure(world: &WorldBlocks, start: IVec3) -> HashSet<IVec3> {
    let mut structure = HashSet::new();
    let mut queue = VecDeque::from([start]);
    structure.insert(start);

    while let Some(pos) = queue.pop_front() {
        for offset in signal_offsets() {
            let neighbor = pos + offset;
            if structure.contains(&neighbor) {
                continue;
            }
            if world
                .system_blocks
                .get(&neighbor)
                .is_some_and(|block| block.kind == BlockKind::Goal)
            {
                structure.insert(neighbor);
                queue.push_back(neighbor);
            }
        }
    }

    structure
}

pub fn material_structure(world: &WorldBlocks, start: IVec3) -> HashSet<IVec3> {
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

pub fn query_factory_structure(world: &WorldBlocks, pos: IVec3) -> Option<HashSet<IVec3>> {
    world
        .is_factory_at(pos)
        .then(|| factory_structure(world, pos))
}

fn welded_neighbors(world: &WorldBlocks, pos: IVec3) -> Vec<IVec3> {
    world
        .material_welds
        .iter()
        .filter_map(|weld| weld.other(pos))
        .filter(|neighbor| world.is_material_at(*neighbor))
        .collect()
}

fn collect_gravity_support(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
) -> Vec<GravitySupportContact> {
    structure
        .iter()
        .filter_map(|pos| {
            let below = *pos + IVec3::NEG_Y;
            (below.y >= 0 && !structure.contains(&below) && !world.can_move_into(below))
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
    world.blocks.get(&from).is_some_and(|block| {
        let offset = to - from;
        match block.kind {
            BlockKind::Detector | BlockKind::Drill | BlockKind::Welder => {
                offset == block.facing.forward_ivec3()
            }
            BlockKind::DownDetector | BlockKind::DownWelder => offset == IVec3::NEG_Y,
            BlockKind::Lifter | BlockKind::Conveyor => offset == IVec3::Y,
            BlockKind::ReverseConveyor => offset == IVec3::NEG_Y,
            _ => false,
        }
    })
}

fn pusher_front_neighbor(world: &WorldBlocks, pos: IVec3) -> Option<IVec3> {
    world.blocks.get(&pos).and_then(|block| {
        matches!(block.kind, BlockKind::Pusher | BlockKind::Blocker)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::blocks::{BlockData, BlockKind};
    use crate::game::world::direction::Facing;

    use crate::game::world::factory_registry::FactoryBlockRegistry;

    fn platform(_pos: IVec3) -> BlockData {
        BlockData {
            kind: BlockKind::Platform,
            facing: Facing::North,
        }
    }

    fn basic_material(_pos: IVec3) -> BlockData {
        BlockData {
            kind: BlockKind::Material,
            facing: Facing::North,
        }
    }

    fn frozen_registry(world: &WorldBlocks) -> FactoryBlockRegistry {
        let mut registry = FactoryBlockRegistry::rebuild_from_world(world);
        registry.freeze_solution();
        registry
    }

    #[test]
    fn rebuild_for_simulation_groups_factory_and_material_separately() {
        let mut world = WorldBlocks::default();
        world.insert(IVec3::ZERO, platform(IVec3::ZERO));
        world.insert(IVec3::X, basic_material(IVec3::X));
        world.insert(IVec3::new(2, 0, 0), basic_material(IVec3::new(2, 0, 0)));
        world.weld_materials(IVec3::X, IVec3::new(2, 0, 0));

        let state = StructureState::rebuild_for_simulation_standalone(&world);
        assert_eq!(state.structures.len(), 2);
        assert_eq!(state.kind_at(IVec3::ZERO), Some(StructureKind::Factory));
        assert_eq!(state.kind_at(IVec3::X), Some(StructureKind::Material));
    }

    #[test]
    fn rebuild_for_simulation_groups_connected_acceptors() {
        let mut world = WorldBlocks::default();
        world.insert(
            IVec3::ZERO,
            BlockData {
                kind: BlockKind::Goal,
                facing: Facing::North,
            },
        );
        world.insert(
            IVec3::X,
            BlockData {
                kind: BlockKind::Goal,
                facing: Facing::North,
            },
        );
        world.insert(
            IVec3::new(0, 2, 0),
            BlockData {
                kind: BlockKind::Goal,
                facing: Facing::North,
            },
        );

        let state = StructureState::rebuild_for_simulation_standalone(&world);
        assert_eq!(state.acceptor_structures().len(), 2);
        assert!(state
            .acceptor_structures()
            .iter()
            .any(|structure| structure.positions.len() == 2));
        assert!(state
            .acceptor_structures()
            .iter()
            .any(|structure| structure.positions.len() == 1));
    }

    #[test]
    fn gravity_support_cache_survives_lookup_after_recorded() {
        let mut world = WorldBlocks::default();
        world.insert(
            IVec3::ZERO,
            BlockData {
                kind: BlockKind::Stone,
                facing: Facing::North,
            },
        );
        world.insert(IVec3::Y, platform(IVec3::Y));

        let mut state = StructureState::rebuild_for_simulation_standalone(&world);
        let index = state.structure_index_at(IVec3::Y).unwrap();
        state.record_gravity_support(index, &world);
        assert!(state.gravity_support_valid(index, &world));
    }

    #[test]
    fn pusher_target_structure_allows_front_subset_when_whole_structure_is_scene_anchored() {
        let mut world = WorldBlocks::default();
        world.insert(
            IVec3::ZERO,
            BlockData {
                kind: BlockKind::Stone,
                facing: Facing::North,
            },
        );
        world.insert(IVec3::Y, platform(IVec3::Y));
        world.insert(
            IVec3::new(0, 2, 0),
            BlockData {
                kind: BlockKind::Pusher,
                facing: Facing::East,
            },
        );
        world.insert(IVec3::new(1, 2, 0), platform(IVec3::new(1, 2, 0)));

        let state = StructureState::rebuild_for_simulation_standalone(&world);
        let registry = frozen_registry(&world);
        assert!(!state.structure(IVec3::Y).unwrap().pushable);

        let subset = state.pusher_target_structure(
            &world,
            &registry,
            IVec3::new(0, 2, 0),
            IVec3::new(1, 2, 0),
            IVec3::X,
        );
        assert_eq!(subset, Some(HashSet::from([IVec3::new(1, 2, 0)])));
    }

    #[test]
    fn pusher_target_structure_uses_solution_connectivity_not_turn() {
        let mut solution = WorldBlocks::default();
        solution.insert(
            IVec3::ZERO,
            BlockData {
                kind: BlockKind::Stone,
                facing: Facing::North,
            },
        );
        solution.insert(IVec3::Y, platform(IVec3::Y));
        solution.insert(
            IVec3::new(1, 2, 0),
            BlockData {
                kind: BlockKind::Pusher,
                facing: Facing::East,
            },
        );
        solution.insert(
            IVec3::new(2, 2, 0),
            BlockData {
                kind: BlockKind::ReverseConveyor,
                facing: Facing::North,
            },
        );

        let mut turn = solution.clone();
        turn.insert(IVec3::new(1, 2, 0), platform(IVec3::new(1, 2, 0)));
        turn.insert(IVec3::new(0, 2, 0), platform(IVec3::new(0, 2, 0)));
        let _ = turn;

        let solution_state = StructureState::rebuild_for_simulation_standalone(&solution);
        let registry = frozen_registry(&solution);
        let conveyor = IVec3::new(2, 2, 0);
        let subset = solution_state.pusher_target_structure(
            &solution,
            &registry,
            IVec3::new(1, 2, 0),
            conveyor,
            IVec3::X,
        );
        assert_eq!(subset, Some(HashSet::from([conveyor])));
    }

    #[test]
    fn pusher_target_structure_rejects_scene_anchored_subset() {
        let mut world = WorldBlocks::default();
        world.insert(
            IVec3::new(1, 0, 0),
            BlockData {
                kind: BlockKind::Stone,
                facing: Facing::North,
            },
        );
        world.insert(IVec3::new(1, 1, 0), platform(IVec3::new(1, 1, 0)));
        world.insert(
            IVec3::new(2, 1, 0),
            BlockData {
                kind: BlockKind::Pusher,
                facing: Facing::West,
            },
        );

        let state = StructureState::rebuild_for_simulation_standalone(&world);
        let registry = frozen_registry(&world);
        assert!(state
            .pusher_target_structure(
                &world,
                &registry,
                IVec3::new(2, 1, 0),
                IVec3::new(1, 1, 0),
                IVec3::NEG_X,
            )
            .is_none());
    }
}

impl StructureState {
    #[cfg(test)]
    fn rebuild_for_simulation_standalone(world: &WorldBlocks) -> Self {
        let mut state = Self::default();
        state.rebuild_for_simulation(world);
        state
    }
}
