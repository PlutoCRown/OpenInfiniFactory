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

#[derive(Clone)]
pub struct FactoryStructure {
    pub kind: StructureKind,
    pub positions: HashSet<IVec3>,
    pub activity: FactoryActivity,
    pub freedom: StructureFreedom,
    pub pushable: bool,
}

#[derive(Resource, Default, Clone)]
pub struct FactoryStructureState {
    structures: Vec<FactoryStructure>,
    structure_by_pos: HashMap<IVec3, usize>,
}

impl FactoryStructureState {
    pub fn rebuild_from_world(&mut self, world: &WorldBlocks) {
        *self = Self::from_world(world);
    }

    pub fn rebuild_near(&mut self, world: &WorldBlocks, changed: impl IntoIterator<Item = IVec3>) {
        let changed: HashSet<IVec3> = changed.into_iter().collect();
        if changed.is_empty() {
            return;
        }
        if self.structure_by_pos.is_empty() {
            self.rebuild_from_world(world);
            return;
        }

        let neighborhood = neighborhood_around(&changed);
        let touched_indices = self.structure_indices_in(&neighborhood);
        let mut positions_to_rebuild = HashSet::new();
        for index in &touched_indices {
            if let Some(structure) = self.structures.get(*index) {
                positions_to_rebuild.extend(structure.positions.iter().copied());
            }
        }
        for pos in &changed {
            if world.is_factory_at(*pos) {
                positions_to_rebuild.insert(*pos);
            }
        }

        self.remove_structures_at_indices(touched_indices);
        self.append_connected_structures(world, positions_to_rebuild);
        self.apply_inactive_propagation(world);
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn ensure_current_world(&mut self, world: &WorldBlocks) {
        if self.structure_by_pos.is_empty() {
            self.rebuild_from_world(world);
        }
    }

    pub fn from_world(world: &WorldBlocks) -> Self {
        let starts: Vec<IVec3> = world
            .blocks
            .iter()
            .filter_map(|(pos, block)| block.kind.is_factory().then_some(*pos))
            .collect();
        let mut state = Self::default();
        state.append_connected_structures(world, starts);
        state.apply_inactive_propagation(world);
        state
    }

    fn append_connected_structures(
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
            self.structures.push(FactoryStructure {
                kind: StructureKind::Factory,
                positions,
                activity: FactoryActivity::Active,
                freedom: StructureFreedom::All,
                pushable: true,
            });
        }
    }

    fn apply_inactive_propagation(&mut self, world: &WorldBlocks) {
        let scene_anchored: Vec<bool> = self
            .structures
            .iter()
            .map(|structure| touches_scene(world, &structure.positions))
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
            for pos in &self.structures[index].positions {
                for offset in signal_offsets() {
                    let neighbor = *pos + offset;
                    let Some(neighbor_index) = self.structure_by_pos.get(&neighbor).copied() else {
                        continue;
                    };
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

    fn structure_indices_in(&self, positions: &HashSet<IVec3>) -> HashSet<usize> {
        positions
            .iter()
            .filter_map(|pos| self.structure_by_pos.get(pos).copied())
            .collect()
    }

    fn remove_structures_at_indices(&mut self, mut indices: HashSet<usize>) {
        if indices.is_empty() {
            return;
        }

        let mut ordered: Vec<usize> = indices.drain().collect();
        ordered.sort_unstable_by(|left, right| right.cmp(left));
        for index in ordered {
            if index >= self.structures.len() {
                continue;
            }
            for pos in &self.structures[index].positions {
                self.structure_by_pos.remove(pos);
            }
            self.structures.swap_remove(index);
            if index < self.structures.len() {
                for pos in &self.structures[index].positions {
                    self.structure_by_pos.insert(*pos, index);
                }
            }
        }
    }

    pub fn activity_at(&self, pos: IVec3) -> Option<FactoryActivity> {
        Some(self.structure(pos)?.activity)
    }

    pub fn active_structure_at(&self, pos: IVec3, offset: IVec3) -> Option<HashSet<IVec3>> {
        let structure = self.structure(pos)?;
        if !structure.pushable || !structure.freedom.can_translate(offset) {
            return None;
        }
        Some(structure.positions.clone())
    }

    pub fn pusher_target_structure(
        &self,
        world: &WorldBlocks,
        pusher_pos: IVec3,
        target_pos: IVec3,
        offset: IVec3,
    ) -> Option<HashSet<IVec3>> {
        let target = self.structure(target_pos)?;
        if !target.pushable || !target.freedom.can_translate(offset) {
            return None;
        }
        let structure =
            connected_subset_with_blocked_edge(world, target, target_pos, Some(pusher_pos));
        (!structure.contains(&pusher_pos)).then_some(structure)
    }

    pub fn falling_structure_at(&self, pos: IVec3, offset: IVec3) -> Option<HashSet<IVec3>> {
        let structure = self.structure(pos)?;
        if structure.activity != FactoryActivity::Active || !structure.freedom.can_translate(offset)
        {
            return None;
        }
        Some(structure.positions.clone())
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
            for pos in &structure.positions {
                self.structure_by_pos.insert(*pos, index);
            }
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

    fn structure(&self, pos: IVec3) -> Option<&FactoryStructure> {
        self.structure_by_pos
            .get(&pos)
            .and_then(|index| self.structures.get(*index))
    }
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
    structure: &FactoryStructure,
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

fn neighborhood_around(changed: &HashSet<IVec3>) -> HashSet<IVec3> {
    let mut neighborhood = HashSet::new();
    for pos in changed {
        neighborhood.insert(*pos);
        for offset in signal_offsets() {
            neighborhood.insert(*pos + offset);
        }
    }
    neighborhood
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::blocks::{BlockData, BlockKind};
    use crate::game::world::direction::Facing;

    fn platform(pos: IVec3) -> BlockData {
        BlockData {
            kind: BlockKind::Platform,
            facing: Facing::North,
        }
    }

    fn insert_platforms(world: &mut WorldBlocks, positions: &[IVec3]) {
        for &pos in positions {
            world.insert(pos, platform(pos));
        }
    }

    fn normalized(
        state: &FactoryStructureState,
    ) -> Vec<(FactoryActivity, StructureFreedom, bool, HashSet<IVec3>)> {
        let mut structures = state
            .structures
            .iter()
            .map(|structure| {
                (
                    structure.activity,
                    structure.freedom,
                    structure.pushable,
                    structure.positions.clone(),
                )
            })
            .collect::<Vec<_>>();
        structures.sort_by_key(|(_, _, _, positions)| {
            positions
                .iter()
                .map(|pos| (pos.x, pos.y, pos.z))
                .min()
                .unwrap_or((0, 0, 0))
        });
        structures
    }

    #[test]
    fn rebuild_near_matches_full_rebuild_after_placing_block() {
        let mut world = WorldBlocks::default();
        insert_platforms(&mut world, &[IVec3::ZERO, IVec3::X, IVec3::NEG_X]);

        let expected = FactoryStructureState::from_world(&world);
        let mut incremental = FactoryStructureState::default();
        incremental.rebuild_near(&world, [IVec3::ZERO]);
        incremental.rebuild_near(&world, [IVec3::X]);
        incremental.rebuild_near(&world, [IVec3::NEG_X]);

        assert_eq!(normalized(&incremental), normalized(&expected));
    }

    #[test]
    fn rebuild_near_splits_connected_structure_after_delete() {
        let mut world = WorldBlocks::default();
        insert_platforms(&mut world, &[IVec3::NEG_X, IVec3::ZERO, IVec3::X]);

        let mut state = FactoryStructureState::from_world(&world);
        world.remove(&IVec3::ZERO);
        state.rebuild_near(&world, [IVec3::ZERO]);

        let expected = FactoryStructureState::from_world(&world);
        assert_eq!(normalized(&state), normalized(&expected));
        assert_eq!(state.structures.len(), 2);
    }

    #[test]
    fn rebuild_near_merges_adjacent_structures_after_bridge() {
        let mut world = WorldBlocks::default();
        insert_platforms(&mut world, &[IVec3::NEG_X, IVec3::X]);

        let mut state = FactoryStructureState::from_world(&world);
        world.insert(IVec3::ZERO, platform(IVec3::ZERO));
        state.rebuild_near(&world, [IVec3::ZERO]);

        let expected = FactoryStructureState::from_world(&world);
        assert_eq!(normalized(&state), normalized(&expected));
        assert_eq!(state.structures.len(), 1);
    }
}
