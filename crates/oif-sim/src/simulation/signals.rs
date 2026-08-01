use glam::IVec3;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::blocks::{BlockId, SignalBehavior, WireFacePolicy};
use crate::world::grid::{MaterialFace, WorldBlocks};

use super::signal_offsets;

/// 信号导线连通分量 ID（缓存内局部，拓扑重建时重分配）
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SignalComponentId(pub usize);

/// 触达某格的信号连通分量查询结果（导线 + 接在该分量上的用电器）
#[derive(Clone, Debug)]
pub struct PowerQuery {
    pub component: SignalComponentId,
    pub wires: Vec<IVec3>,
    pub devices: Vec<IVec3>,
    /// 当前是否通电（仅占位类传感器，不含激光二次供电）
    pub powered: bool,
}

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
    /// 按世界拓扑重建导线连通分量与用电器接线
    pub fn refresh(&mut self, world: &WorldBlocks) {
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
                Some(SignalBehavior::PoweredDevice { wire_face }) => {
                    self.cache_powered_device(world, block.id, pos, wire_face);
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
        for component in adjacent_wire_components(
            world,
            &self.wire_components,
            pos,
            WireFacePolicy::BlockOne(blocked_offset),
        ) {
            self.component_detectors[component.0].push(id);
        }
    }

    fn cache_powered_device(
        &mut self,
        world: &WorldBlocks,
        id: BlockId,
        pos: IVec3,
        wire_face: WireFacePolicy,
    ) {
        let components = adjacent_wire_components(world, &self.wire_components, pos, wire_face);
        if !components.is_empty() {
            self.device_components.insert(id, components);
        }
    }

    /// 传感器激活：激光打中工作面 ∪ 检测格占位（材料 / Behavior 声明目标）
    pub fn powered_components(
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
                        detector_activated(world, detector_pos, laser_hit_detectors)
                    })
                    .then_some(SignalComponentId(component))
            })
            .collect()
    }

    /// 通电连通分量上的用电器格
    pub fn powered_devices(
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

    /// 通电连通分量上的导线格
    pub fn powered_wires(
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

    /// 刷新缓存并查询触达 `pos` 的信号网络（导线 + 用电器）
    pub fn query_power_at(&mut self, world: &WorldBlocks, pos: IVec3) -> Option<PowerQuery> {
        self.refresh(world);
        let component = self.component_touching(world, pos)?;
        let mut wires: Vec<IVec3> = world
            .blocks
            .iter()
            .filter_map(|(p, block)| {
                self.wire_components
                    .get(&block.id)
                    .filter(|c| **c == component)
                    .map(|_| *p)
            })
            .collect();
        wires.sort_by_key(|p| (p.x, p.y, p.z));
        let mut devices: Vec<IVec3> = world
            .blocks
            .iter()
            .filter_map(|(p, block)| {
                self.device_components
                    .get(&block.id)
                    .filter(|cs| cs.contains(&component))
                    .map(|_| *p)
            })
            .collect();
        devices.sort_by_key(|p| (p.x, p.y, p.z));
        let powered = self
            .powered_components(world, &HashSet::new())
            .contains(&component);
        Some(PowerQuery {
            component,
            wires,
            devices,
            powered,
        })
    }

    /// 导线格、用电器格或其邻接导线所属的连通分量
    fn component_touching(&self, world: &WorldBlocks, pos: IVec3) -> Option<SignalComponentId> {
        if let Some(block) = world.blocks.get(&pos) {
            if let Some(&component) = self.wire_components.get(&block.id) {
                return Some(component);
            }
            if let Some(components) = self.device_components.get(&block.id) {
                return components.first().copied();
            }
        }
        for offset in signal_offsets() {
            let Some(block) = world.blocks.get(&(pos + offset)) else {
                continue;
            };
            if let Some(&component) = self.wire_components.get(&block.id) {
                return Some(component);
            }
        }
        None
    }
}

/// 收集设备/传感器相邻导线所属的信号连通分量（按接线策略过滤面）
fn adjacent_wire_components(
    world: &WorldBlocks,
    wire_components: &HashMap<BlockId, SignalComponentId>,
    pos: IVec3,
    wire_face: WireFacePolicy,
) -> Vec<SignalComponentId> {
    let mut components = Vec::new();
    let mut seen = HashSet::new();
    for offset in signal_offsets() {
        if !wire_face.allows(offset) {
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

/// 传感器是否激活：激光打中（LaserHit）或检测格被可检测目标占据（Occupancy）
fn detector_activated(
    world: &WorldBlocks,
    detector_pos: IVec3,
    laser_hit_detectors: &HashSet<IVec3>,
) -> bool {
    if laser_hit_detectors.contains(&detector_pos) {
        return true;
    }
    let Some(block) = world.blocks.get(&detector_pos) else {
        return false;
    };
    let Some(SignalBehavior::Detector { detection_pos }) = block.kind.signal_behavior(block.facing)
    else {
        return false;
    };
    world.is_detectable_by_detector_at(detector_pos + detection_pos)
}

