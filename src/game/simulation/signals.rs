use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::game::blocks::SignalBehavior;
use crate::game::world::grid::WorldBlocks;

use super::signal_offsets;

#[derive(Default, Resource, Clone)]
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
            if !matches!(
                block.kind.signal_behavior(block.facing),
                Some(SignalBehavior::Wire)
            ) || self.wire_components.contains_key(&pos)
            {
                continue;
            }

            let component = self.component_detectors.len();
            self.component_detectors.push(Vec::new());
            let mut queue = VecDeque::from([pos]);
            self.wire_components.insert(pos, component);

            while let Some(wire_pos) = queue.pop_front() {
                for offset in signal_offsets() {
                    let neighbor = wire_pos + offset;
                    if carries_signal_at(world, neighbor)
                        && self.wire_components.insert(neighbor, component).is_none()
                    {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        for (&pos, block) in &world.blocks {
            match block.kind.signal_behavior(block.facing) {
                Some(SignalBehavior::Detector { detection_pos }) => {
                    self.cache_detector(pos, detection_pos);
                }
                Some(SignalBehavior::PoweredDevice) => {
                    self.cache_powered_device(pos, block.facing.forward_ivec3());
                }
                Some(SignalBehavior::Wire) | None => {}
            }
        }
    }

    fn cache_detector(&mut self, pos: IVec3, blocked_offset: IVec3) {
        let mut connected_components = HashSet::new();
        for offset in signal_offsets() {
            if offset == blocked_offset {
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

    fn cache_powered_device(&mut self, pos: IVec3, blocked_offset: IVec3) {
        let mut components = Vec::new();
        let mut seen = HashSet::new();
        for offset in signal_offsets() {
            if offset == blocked_offset {
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

    pub(super) fn powered_devices(&self, powered_components: &HashSet<usize>) -> HashSet<IVec3> {
        self.device_components
            .iter()
            .filter_map(|(pos, components)| {
                components
                    .iter()
                    .any(|component| powered_components.contains(component))
                    .then_some(*pos)
            })
            .collect()
    }

    pub(super) fn powered_wires(&self, powered_components: &HashSet<usize>) -> HashSet<IVec3> {
        self.wire_components
            .iter()
            .filter_map(|(pos, component)| powered_components.contains(component).then_some(*pos))
            .collect()
    }
}

fn detector_is_active(world: &WorldBlocks, pos: IVec3) -> bool {
    let Some(block) = world.blocks.get(&pos) else {
        return false;
    };
    let Some(SignalBehavior::Detector { detection_pos }) = block.kind.signal_behavior(block.facing)
    else {
        return false;
    };

    world.is_detectable_by_detector_at(pos + detection_pos)
}

fn carries_signal_at(world: &WorldBlocks, pos: IVec3) -> bool {
    world.blocks.get(&pos).is_some_and(|block| {
        matches!(
            block.kind.signal_behavior(block.facing),
            Some(SignalBehavior::Wire)
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::blocks::{BlockData, BlockKind};
    use crate::game::world::direction::Facing;

    fn place_factory(world: &mut WorldBlocks, pos: IVec3, kind: BlockKind) {
        world.insert(
            pos,
            BlockData {
                kind,
                facing: Facing::North,
            },
        );
    }

    fn wired_detector(world: &mut WorldBlocks, detector: IVec3, wire: IVec3) {
        world.insert(
            detector,
            BlockData {
                kind: BlockKind::Detector,
                facing: Facing::East,
            },
        );
        place_factory(world, wire, BlockKind::Wire);
    }

    #[test]
    fn detector_detects_material_and_platform_only_among_factory_blocks() {
        let mut world = WorldBlocks::default();
        let detector = IVec3::ZERO;
        let target = IVec3::X;
        let wire = IVec3::NEG_X;
        wired_detector(&mut world, detector, wire);
        let mut cache = SignalNetworkCache::default();

        place_factory(&mut world, target, BlockKind::Platform);
        cache.refresh(&world);
        assert!(cache.powered_components(&world).contains(&0));

        place_factory(&mut world, target, BlockKind::Conveyor);
        cache.refresh(&world);
        assert!(cache.powered_components(&world).is_empty());

        world.remove(&target);
        place_factory(
            &mut world,
            target,
            BlockKind::material_block_kind(crate::game::blocks::MaterialKind::Basic).unwrap(),
        );
        cache.refresh(&world);
        assert!(cache.powered_components(&world).contains(&0));
    }
}
