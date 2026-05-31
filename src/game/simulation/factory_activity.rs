use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::game::world::blocks::BlockKind;
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

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn ensure_current_world(&mut self, world: &WorldBlocks) {
        if self.structure_by_pos.is_empty() {
            self.rebuild_from_world(world);
        }
    }

    pub fn from_world(world: &WorldBlocks) -> Self {
        let mut structures = Vec::new();
        let mut structure_by_pos = HashMap::new();
        let mut handled = HashSet::new();
        let mut starts: Vec<IVec3> = world
            .blocks
            .iter()
            .filter_map(|(pos, block)| block.kind.is_factory().then_some(*pos))
            .collect();
        starts.sort_by_key(|pos| (pos.x, pos.y, pos.z));

        for start in starts {
            if handled.contains(&start) || !world.is_factory_at(start) {
                continue;
            }

            let positions = factory_structure(world, start);
            let index = structures.len();
            for pos in &positions {
                handled.insert(*pos);
                structure_by_pos.insert(*pos, index);
            }
            structures.push(FactoryStructure {
                kind: StructureKind::Factory,
                positions,
                activity: FactoryActivity::Active,
                freedom: StructureFreedom::All,
                pushable: true,
            });
        }

        let scene_anchored: Vec<bool> = structures
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
            for pos in &structures[index].positions {
                for offset in signal_offsets() {
                    let neighbor = *pos + offset;
                    let Some(neighbor_index) = structure_by_pos.get(&neighbor).copied() else {
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

        for (index, structure) in structures.iter_mut().enumerate() {
            if inactive[index] {
                structure.activity = FactoryActivity::Inactive;
                if scene_anchored[index] {
                    structure.freedom = StructureFreedom::None;
                    structure.pushable = false;
                }
            }
        }

        Self {
            structures,
            structure_by_pos,
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
            let should_move = structure
                .positions
                .iter()
                .any(|pos| positions.contains(pos));
            if !should_move {
                continue;
            }
            structure.positions = structure
                .positions
                .iter()
                .map(|pos| *pos + offset)
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
    let mut structure = HashSet::new();
    let mut queue = VecDeque::from([start]);
    structure.insert(start);

    while let Some(pos) = queue.pop_front() {
        for offset in signal_offsets() {
            let neighbor = pos + offset;
            if structure.contains(&neighbor)
                || !world.is_factory_at(neighbor)
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

fn is_blocked_factory_connection(world: &WorldBlocks, from: IVec3, to: IVec3) -> bool {
    world.blocks.get(&from).is_some_and(|block| {
        let offset = to - from;
        match block.kind {
            BlockKind::Pusher
            | BlockKind::Blocker
            | BlockKind::Detector
            | BlockKind::Drill
            | BlockKind::Welder => offset == block.facing.forward_ivec3(),
            BlockKind::Lifter | BlockKind::Conveyor => offset == IVec3::Y,
            BlockKind::ReverseConveyor => offset == IVec3::NEG_Y,
            _ => false,
        }
    })
}

fn touches_scene(world: &WorldBlocks, structure: &HashSet<IVec3>) -> bool {
    structure.iter().any(|pos| {
        signal_offsets()
            .into_iter()
            .any(|offset| world.is_scene_at(*pos + offset))
    })
}
