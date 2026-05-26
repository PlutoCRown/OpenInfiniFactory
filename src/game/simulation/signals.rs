use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::game::world::blocks::{BlockKind, Facing};
use crate::game::world::grid::WorldBlocks;

use super::signal_offsets;

#[derive(Default, Resource)]
pub struct SignalNetworkCache {
    topology_revision: u64,
    wire_components: HashMap<IVec3, usize>,
    component_detectors: Vec<Vec<IVec3>>,
    device_components: HashMap<IVec3, Vec<usize>>,
    initialized: bool,
}

impl SignalNetworkCache {
    pub(super) fn refresh(&mut self, world: &WorldBlocks) {
        if self.initialized && self.topology_revision == world.topology_revision {
            return;
        }

        self.topology_revision = world.topology_revision;
        self.wire_components.clear();
        self.component_detectors.clear();
        self.device_components.clear();
        self.initialized = true;

        for (&pos, block) in &world.blocks {
            if block.kind != BlockKind::Wire || self.wire_components.contains_key(&pos) {
                continue;
            }

            let component = self.component_detectors.len();
            self.component_detectors.push(Vec::new());
            let mut queue = VecDeque::from([pos]);
            self.wire_components.insert(pos, component);

            while let Some(wire_pos) = queue.pop_front() {
                for offset in signal_offsets() {
                    let neighbor = wire_pos + offset;
                    if is_wire_at(world, neighbor)
                        && self.wire_components.insert(neighbor, component).is_none()
                    {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        for (&pos, block) in &world.blocks {
            match block.kind {
                BlockKind::Detector => self.cache_detector(pos, block.facing),
                BlockKind::Piston | BlockKind::Blocker | BlockKind::Laser => {
                    self.cache_powered_device(pos, block.facing)
                }
                _ => {}
            }
        }
    }

    fn cache_detector(&mut self, pos: IVec3, facing: Facing) {
        let mut connected_components = HashSet::new();
        for offset in signal_offsets() {
            if offset == facing.forward_ivec3() {
                continue;
            }

            let Some(&component) = self.wire_components.get(&(pos + offset)) else {
                continue;
            };
            if connected_components.insert(component) {
                self.component_detectors[component].push(pos);
            }
        }
    }

    fn cache_powered_device(&mut self, pos: IVec3, facing: Facing) {
        let mut components = Vec::new();
        let mut seen = HashSet::new();
        for offset in signal_offsets() {
            if offset == facing.forward_ivec3() {
                continue;
            }

            let Some(&component) = self.wire_components.get(&(pos + offset)) else {
                continue;
            };
            if seen.insert(component) {
                components.push(component);
            }
        }

        if !components.is_empty() {
            self.device_components.insert(pos, components);
        }
    }

    pub(super) fn powered_components(&self, world: &WorldBlocks) -> HashSet<usize> {
        self.component_detectors
            .iter()
            .enumerate()
            .filter_map(|(component, detectors)| {
                detectors
                    .iter()
                    .any(|detector_pos| detector_is_active(world, *detector_pos))
                    .then_some(component)
            })
            .collect()
    }

    pub(super) fn is_device_powered(
        &self,
        pos: IVec3,
        powered_components: &HashSet<usize>,
    ) -> bool {
        self.device_components.get(&pos).is_some_and(|components| {
            components
                .iter()
                .any(|component| powered_components.contains(component))
        })
    }
}

fn detector_is_active(world: &WorldBlocks, pos: IVec3) -> bool {
    let Some(block) = world.blocks.get(&pos) else {
        return false;
    };
    if block.kind != BlockKind::Detector {
        return false;
    }

    let detected_pos = pos + block.facing.forward_ivec3();
    world
        .blocks
        .get(&detected_pos)
        .is_some_and(|detected| detected.kind.is_material())
}

fn is_wire_at(world: &WorldBlocks, pos: IVec3) -> bool {
    world
        .blocks
        .get(&pos)
        .is_some_and(|block| block.kind == BlockKind::Wire)
}
