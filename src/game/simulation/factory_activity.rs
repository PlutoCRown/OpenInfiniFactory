use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::game::world::grid::WorldBlocks;

use super::signal_offsets;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactoryActivity {
    Active,
    Inactive,
}

pub fn factory_activity_map(world: &WorldBlocks) -> HashMap<IVec3, FactoryActivity> {
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

        let structure = factory_structure(world, start);
        let index = structures.len();
        for pos in &structure {
            handled.insert(*pos);
            structure_by_pos.insert(*pos, index);
        }
        structures.push(structure);
    }

    let mut inactive = vec![false; structures.len()];
    let mut queue = VecDeque::new();
    for (index, structure) in structures.iter().enumerate() {
        if touches_scene(world, structure) {
            inactive[index] = true;
            queue.push_back(index);
        }
    }

    while let Some(index) = queue.pop_front() {
        for pos in &structures[index] {
            for offset in signal_offsets() {
                let neighbor = *pos + offset;
                let Some(neighbor_index) = structure_by_pos.get(&neighbor).copied() else {
                    continue;
                };
                if !inactive[neighbor_index] {
                    inactive[neighbor_index] = true;
                    queue.push_back(neighbor_index);
                }
            }
        }
    }

    let mut activity = HashMap::new();
    for (index, structure) in structures.into_iter().enumerate() {
        let value = if inactive[index] {
            FactoryActivity::Inactive
        } else {
            FactoryActivity::Active
        };
        for pos in structure {
            activity.insert(pos, value);
        }
    }
    activity
}

pub(super) fn active_factory_structure(
    world: &WorldBlocks,
    activity: &HashMap<IVec3, FactoryActivity>,
    start: IVec3,
) -> Option<HashSet<IVec3>> {
    if activity.get(&start) != Some(&FactoryActivity::Active) {
        return None;
    }
    Some(factory_structure(world, start))
}

pub(super) fn factory_structure(world: &WorldBlocks, start: IVec3) -> HashSet<IVec3> {
    let mut structure = HashSet::new();
    let mut queue = VecDeque::from([start]);
    structure.insert(start);

    while let Some(pos) = queue.pop_front() {
        for offset in signal_offsets() {
            let neighbor = pos + offset;
            if structure.contains(&neighbor) || !world.is_factory_at(neighbor) {
                continue;
            }
            structure.insert(neighbor);
            queue.push_back(neighbor);
        }
    }

    structure
}

fn touches_scene(world: &WorldBlocks, structure: &HashSet<IVec3>) -> bool {
    structure.iter().any(|pos| {
        signal_offsets()
            .into_iter()
            .any(|offset| world.is_scene_at(*pos + offset))
    })
}
