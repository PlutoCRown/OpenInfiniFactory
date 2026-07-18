use glam::IVec3;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::blocks::{BlockId, SignalBehavior};
use crate::world::grid::{MaterialFace, WorldBlocks};

use super::signal_offsets;

/// 信号导线连通分量 ID（缓存内局部，拓扑重建时重分配）
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SignalComponentId(pub usize);

/// 信号网络缓存：导线/用电器按 BlockId 索引，移动后身份仍有效
#[derive(Default, Clone)]
pub struct SignalNetworkCache {
    topology_revision: u64,
    wire_components: HashMap<BlockId, SignalComponentId>,
    component_detectors: Vec<Vec<BlockId>>,
    device_components: HashMap<BlockId, Vec<SignalComponentId>>,
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
            ) || self.wire_components.contains_key(&block.id)
            {
                continue;
            }

            let component = SignalComponentId(self.component_detectors.len());
            self.component_detectors.push(Vec::new());
            let mut queue = VecDeque::from([pos]);
            self.wire_components.insert(block.id, component);

            while let Some(wire_pos) = queue.pop_front() {
                let Some(wire_block) = world.blocks.get(&wire_pos).copied() else {
                    continue;
                };
                for offset in signal_offsets() {
                    if world
                        .wire_face_panels
                        .contains(&MaterialFace::new(wire_block.id, offset))
                    {
                        continue;
                    }
                    let neighbor = wire_pos + offset;
                    let Some(neighbor_block) = world.blocks.get(&neighbor) else {
                        continue;
                    };
                    if !matches!(
                        neighbor_block.kind.signal_behavior(neighbor_block.facing),
                        Some(SignalBehavior::Wire)
                    ) {
                        continue;
                    }
                    if world
                        .wire_face_panels
                        .contains(&MaterialFace::new(neighbor_block.id, -offset))
                    {
                        continue;
                    }
                    if self
                        .wire_components
                        .insert(neighbor_block.id, component)
                        .is_none()
                    {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        for (&pos, block) in &world.blocks {
            match block.kind.signal_behavior(block.facing) {
                Some(SignalBehavior::Detector { detection_pos }) => {
                    self.cache_detector(world, block.id, pos, detection_pos);
                }
                Some(SignalBehavior::PoweredDevice) => {
                    self.cache_powered_device(world, block.id, pos, block.facing.forward_ivec3());
                }
                Some(SignalBehavior::Wire) | None => {}
            }
        }
    }

    fn cache_detector(
        &mut self,
        world: &WorldBlocks,
        id: BlockId,
        pos: IVec3,
        blocked_offset: IVec3,
    ) {
        for component in adjacent_wire_components(world, &self.wire_components, pos, blocked_offset)
        {
            self.component_detectors[component.0].push(id);
        }
    }

    fn cache_powered_device(
        &mut self,
        world: &WorldBlocks,
        id: BlockId,
        pos: IVec3,
        blocked_offset: IVec3,
    ) {
        let components =
            adjacent_wire_components(world, &self.wire_components, pos, blocked_offset);
        if !components.is_empty() {
            self.device_components.insert(id, components);
        }
    }

    // laser_hit_detectors：本回合激光打中工作面的传感器，视为已激活（按当前坐标，回合内临时）
    pub(super) fn powered_components(
        &self,
        world: &WorldBlocks,
        laser_hit_detectors: &HashSet<IVec3>,
    ) -> HashSet<SignalComponentId> {
        let id_to_pos: HashMap<BlockId, IVec3> = world
            .blocks
            .iter()
            .map(|(pos, block)| (block.id, *pos))
            .collect();
        self.component_detectors
            .iter()
            .enumerate()
            .filter_map(|(component, detectors)| {
                detectors
                    .iter()
                    .any(|detector_id| {
                        let Some(&detector_pos) = id_to_pos.get(detector_id) else {
                            return false;
                        };
                        laser_hit_detectors.contains(&detector_pos)
                            || detector_is_active(world, detector_pos)
                    })
                    .then_some(SignalComponentId(component))
            })
            .collect()
    }

    pub(super) fn powered_devices(
        &self,
        world: &WorldBlocks,
        powered_components: &HashSet<SignalComponentId>,
    ) -> HashSet<IVec3> {
        world
            .blocks
            .iter()
            .filter_map(|(pos, block)| {
                self.device_components
                    .get(&block.id)
                    .filter(|components| {
                        components
                            .iter()
                            .any(|component| powered_components.contains(component))
                    })
                    .map(|_| *pos)
            })
            .collect()
    }

    pub(super) fn powered_wires(
        &self,
        world: &WorldBlocks,
        powered_components: &HashSet<SignalComponentId>,
    ) -> HashSet<IVec3> {
        world
            .blocks
            .iter()
            .filter_map(|(pos, block)| {
                self.wire_components
                    .get(&block.id)
                    .filter(|component| powered_components.contains(component))
                    .map(|_| *pos)
            })
            .collect()
    }
}

/// 收集设备/传感器相邻导线所属的信号连通分量（跳过 blocked 面）
fn adjacent_wire_components(
    world: &WorldBlocks,
    wire_components: &HashMap<BlockId, SignalComponentId>,
    pos: IVec3,
    blocked_offset: IVec3,
) -> Vec<SignalComponentId> {
    let mut components = Vec::new();
    let mut seen = HashSet::new();
    for offset in signal_offsets() {
        if offset == blocked_offset {
            continue;
        }
        let Some(wire) = world.blocks.get(&(pos + offset)) else {
            continue;
        };
        let Some(&component) = wire_components.get(&wire.id) else {
            continue;
        };
        if seen.insert(component) {
            components.push(component);
        }
    }
    components
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{BlockData, BlockKind};
    use crate::world::direction::Facing;

    fn place_factory(world: &mut WorldBlocks, pos: IVec3, kind: BlockKind) {
        world.insert(pos, BlockData::new(kind, Facing::North));
    }

    fn wired_detector(world: &mut WorldBlocks, detector: IVec3, wire: IVec3) {
        world.insert(
            detector,
            BlockData::new(BlockKind::Detector, Facing::East),
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

        let no_laser = HashSet::new();
        place_factory(&mut world, target, BlockKind::Platform);
        cache.refresh(&world);
        assert!(cache
            .powered_components(&world, &no_laser)
            .contains(&SignalComponentId(0)));

        place_factory(&mut world, target, BlockKind::Conveyor);
        cache.refresh(&world);
        assert!(cache.powered_components(&world, &no_laser).is_empty());

        world.remove(&target);
        place_factory(
            &mut world,
            target,
            BlockKind::material("basic"),
        );
        cache.refresh(&world);
        assert!(cache
            .powered_components(&world, &no_laser)
            .contains(&SignalComponentId(0)));
    }

    #[test]
    fn laser_hit_detector_powers_network_without_detectable_target() {
        let mut world = WorldBlocks::default();
        let detector = IVec3::ZERO;
        let wire = IVec3::NEG_X;
        wired_detector(&mut world, detector, wire);
        let mut cache = SignalNetworkCache::default();
        cache.refresh(&world);

        let no_laser = HashSet::new();
        assert!(cache.powered_components(&world, &no_laser).is_empty());

        let laser_hits = HashSet::from([detector]);
        assert!(cache
            .powered_components(&world, &laser_hits)
            .contains(&SignalComponentId(0)));
    }

    #[test]
    fn material_relocate_skips_topology_bump_and_wire_relocate_bumps_once() {
        let mut world = WorldBlocks::default();
        let detector = IVec3::ZERO;
        let wire = IVec3::NEG_X;
        let target = IVec3::X;
        wired_detector(&mut world, detector, wire);
        place_factory(&mut world, target, BlockKind::Platform);
        place_factory(
            &mut world,
            IVec3::new(0, 2, 0),
            BlockKind::material("basic"),
        );
        let mut cache = SignalNetworkCache::default();
        cache.refresh(&world);
        let revision_after_build = world.topology_revision;
        assert!(cache
            .powered_components(&world, &HashSet::new())
            .contains(&SignalComponentId(0)));

        let material_pos = IVec3::new(0, 2, 0);
        let material = world.blocks[&material_pos];
        world.relocate_blocks(vec![(material_pos, material_pos + IVec3::Y, material)]);
        assert_eq!(world.topology_revision, revision_after_build);
        cache.refresh(&world);
        assert!(cache
            .powered_components(&world, &HashSet::new())
            .contains(&SignalComponentId(0)));

        let wire_block = world.blocks[&wire];
        let detector_block = world.blocks[&detector];
        world.relocate_blocks(vec![
            (wire, wire + IVec3::Y, wire_block),
            (detector, detector + IVec3::Y, detector_block),
            (
                target,
                target + IVec3::Y,
                world.blocks[&target],
            ),
        ]);
        assert_eq!(
            world.topology_revision,
            revision_after_build.wrapping_add(1)
        );
        cache.refresh(&world);
        let powered = cache.powered_components(&world, &HashSet::new());
        assert!(powered.contains(&SignalComponentId(0)));
        assert!(cache
            .powered_wires(&world, &powered)
            .contains(&(wire + IVec3::Y)));
    }

    #[test]
    fn wire_face_panel_splits_signal_components() {
        let mut world = WorldBlocks::default();
        let a = IVec3::ZERO;
        let b = IVec3::X;
        place_factory(&mut world, a, BlockKind::Wire);
        place_factory(&mut world, b, BlockKind::Wire);
        let a_id = world.blocks[&a].id;
        world.set_wire_face_panel(MaterialFace::new(a_id, IVec3::X), true);

        let mut cache = SignalNetworkCache::default();
        cache.refresh(&world);
        let comp_a = cache.wire_components[&a_id];
        let comp_b = cache.wire_components[&world.blocks[&b].id];
        assert_ne!(comp_a, comp_b);

        world.set_wire_face_panel(MaterialFace::new(a_id, IVec3::X), false);
        cache.refresh(&world);
        assert_eq!(
            cache.wire_components[&a_id],
            cache.wire_components[&world.blocks[&b].id]
        );
    }

    #[test]
    fn neighbor_panel_also_blocks_signal() {
        let mut world = WorldBlocks::default();
        let a = IVec3::ZERO;
        let b = IVec3::X;
        place_factory(&mut world, a, BlockKind::Wire);
        place_factory(&mut world, b, BlockKind::Wire);
        let b_id = world.blocks[&b].id;
        world.set_wire_face_panel(MaterialFace::new(b_id, IVec3::NEG_X), true);

        let mut cache = SignalNetworkCache::default();
        cache.refresh(&world);
        assert_ne!(
            cache.wire_components[&world.blocks[&a].id],
            cache.wire_components[&b_id]
        );
    }
}
