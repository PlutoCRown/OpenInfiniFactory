use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::game::state::{BuilderMode, SimulationState};
use crate::game::world::blocks::{BlockData, BlockKind, Facing};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{despawn_world, rebuild_world, BlockEntity, WorldRenderAssets};

#[derive(Default, Resource)]
pub struct SignalNetworkCache {
    topology_revision: u64,
    wire_components: HashMap<IVec3, usize>,
    component_detectors: Vec<Vec<IVec3>>,
    device_components: HashMap<IVec3, Vec<usize>>,
    initialized: bool,
}

pub fn run_turn(
    world: &mut WorldBlocks,
    signal_cache: &mut SignalNetworkCache,
    commands: &mut Commands,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
) {
    sync_generated_markers(world, signal_cache, &HashSet::new());
    signal_cache.refresh(world);
    let powered_components = signal_cache.powered_components(world);
    sync_generated_markers(world, signal_cache, &powered_components);
    drill_materials(world);
    fire_lasers(world, signal_cache, &powered_components);
    push_powered_pistons(world, signal_cache, &powered_components);
    lift_structures(world);
    rotate_structures(world);
    spawn_materials(world);
    move_conveyed_materials(world);
    apply_material_gravity(world);
    apply_factory_gravity(world);
    signal_cache.refresh(world);

    despawn_world(commands, block_entities);
    rebuild_world(commands, world, render_assets);
}

pub fn reset_simulation(world: &mut WorldBlocks) {
    world.retain(|_, block| !block.kind.is_material());
    world.clear_generated_markers();
}

pub fn tick_simulation(
    time: Res<Time>,
    builder_mode: Res<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut world: ResMut<WorldBlocks>,
    mut signal_cache: ResMut<SignalNetworkCache>,
    mut commands: Commands,
    block_entities: Query<Entity, With<BlockEntity>>,
    render_assets: Res<WorldRenderAssets>,
) {
    if *builder_mode != BuilderMode::Play || !simulation.running {
        return;
    }

    simulation.accumulator += time.delta_seconds() * simulation.speed;
    while simulation.accumulator >= 1.0 {
        simulation.turn += 1;
        simulation.accumulator -= 1.0;
        run_turn(
            &mut world,
            &mut signal_cache,
            &mut commands,
            &block_entities,
            &render_assets,
        );
    }
}

fn sync_generated_markers(
    world: &mut WorldBlocks,
    signal_cache: &SignalNetworkCache,
    powered_components: &HashSet<usize>,
) {
    world.clear_generated_markers();

    let welders: Vec<(IVec3, Facing)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            (block.kind == BlockKind::Welder).then_some((*pos, block.facing))
        })
        .collect();

    for (pos, facing) in welders {
        let point_pos = pos + facing.forward_ivec3();
        if !world.is_occupied(point_pos) {
            world.insert(
                point_pos,
                BlockData {
                    kind: BlockKind::WeldPoint,
                    facing,
                },
            );
        }
    }

    let blockers: Vec<(IVec3, Facing)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            (block.kind == BlockKind::Blocker).then_some((*pos, block.facing))
        })
        .collect();

    for (pos, facing) in blockers {
        if signal_cache.is_device_powered(pos, powered_components) {
            continue;
        }

        let head_pos = pos + facing.forward_ivec3();
        if world.can_place_solid_at(head_pos) {
            world.insert(
                head_pos,
                BlockData {
                    kind: BlockKind::BlockerHead,
                    facing,
                },
            );
        }
    }

    let drills: Vec<(IVec3, Facing)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| (block.kind == BlockKind::Drill).then_some((*pos, block.facing)))
        .collect();

    for (pos, facing) in drills {
        let head_pos = pos + facing.forward_ivec3();
        if !world.is_occupied(head_pos) {
            world.insert(
                head_pos,
                BlockData {
                    kind: BlockKind::DrillHead,
                    facing,
                },
            );
        }
    }
}

fn spawn_materials(world: &mut WorldBlocks) {
    let generators: Vec<(IVec3, Facing)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            (block.kind == BlockKind::Generator).then_some((*pos, block.facing))
        })
        .collect();

    for (pos, facing) in generators {
        let spawn_pos = pos + facing.forward_ivec3();
        if world.can_place_solid_at(spawn_pos) {
            world.insert(
                spawn_pos,
                BlockData {
                    kind: BlockKind::Material,
                    facing: Facing::North,
                },
            );
        }
    }
}

fn apply_material_gravity(world: &mut WorldBlocks) {
    let mut materials: Vec<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| block.kind.is_material().then_some(*pos))
        .collect();
    materials.sort_by_key(|pos| pos.y);

    for pos in materials {
        let Some(block) = world.blocks.get(&pos).copied() else {
            continue;
        };
        let next = pos + IVec3::NEG_Y;
        if next.y >= 0 && world.can_place_solid_at(next) {
            world.remove(&pos);
            world.insert(next, block);
        }
    }
}

fn apply_factory_gravity(world: &mut WorldBlocks) {
    let mut factory_blocks: Vec<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| block.kind.is_factory().then_some(*pos))
        .collect();
    factory_blocks.sort_by_key(|pos| pos.y);

    for pos in factory_blocks {
        let Some(block) = world.blocks.get(&pos).copied() else {
            continue;
        };
        let next = pos + IVec3::NEG_Y;
        if next.y >= 0 && world.can_place_solid_at(next) {
            world.remove(&pos);
            world.insert(next, block);
        }
    }
}

fn move_conveyed_materials(world: &mut WorldBlocks) {
    let conveyors: Vec<(IVec3, Facing)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            (block.kind == BlockKind::Conveyor).then_some((*pos, block.facing))
        })
        .collect();

    for (pos, facing) in conveyors {
        let source = pos + IVec3::Y;
        let Some(block) = world.blocks.get(&source).copied() else {
            continue;
        };
        if !block.kind.is_material() {
            continue;
        }

        let target = source + facing.forward_ivec3();
        if world.can_place_solid_at(target) {
            world.remove(&source);
            world.insert(target, block);
        }
    }
}

fn lift_structures(world: &mut WorldBlocks) {
    let lifters: Vec<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| (block.kind == BlockKind::Lifter).then_some(*pos))
        .collect();

    for pos in lifters {
        let Some(source) = (1..=5)
            .map(|height| pos + IVec3::Y * height)
            .find(|candidate| {
                world
                    .blocks
                    .get(candidate)
                    .is_some_and(|block| block.kind.is_material())
            })
        else {
            continue;
        };

        let structure = material_structure(world, source);
        if can_move_structure(world, &structure, IVec3::Y) {
            move_structure(world, &structure, IVec3::Y);
        }
    }
}

fn rotate_structures(world: &mut WorldBlocks) {
    let rotators: Vec<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| (block.kind == BlockKind::Rotator).then_some(*pos))
        .collect();

    for pos in rotators {
        let source = pos + IVec3::Y;
        let Some(block) = world.blocks.get(&source) else {
            continue;
        };
        if !block.kind.is_material() {
            continue;
        }

        let structure = material_structure(world, source);
        if can_rotate_structure(world, &structure, pos) {
            rotate_structure(world, &structure, pos);
        }
    }
}

fn drill_materials(world: &mut WorldBlocks) {
    let drills: Vec<(IVec3, Facing)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| (block.kind == BlockKind::Drill).then_some((*pos, block.facing)))
        .collect();

    for (pos, facing) in drills {
        let target = pos + facing.forward_ivec3();
        if world
            .blocks
            .get(&target)
            .is_some_and(|block| block.kind.is_material())
        {
            world.remove(&target);
        }
    }

    let drill_heads: Vec<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| (block.kind == BlockKind::DrillHead).then_some(*pos))
        .collect();

    for pos in drill_heads {
        for offset in signal_offsets() {
            let target = pos + offset;
            if world
                .blocks
                .get(&target)
                .is_some_and(|block| block.kind.is_material())
            {
                world.remove(&target);
            }
        }
    }
}

fn fire_lasers(
    world: &mut WorldBlocks,
    signal_cache: &SignalNetworkCache,
    powered_components: &HashSet<usize>,
) {
    let lasers: Vec<(IVec3, Facing)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| (block.kind == BlockKind::Laser).then_some((*pos, block.facing)))
        .collect();

    for (pos, facing) in lasers {
        if !signal_cache.is_device_powered(pos, powered_components) {
            continue;
        }

        let forward = facing.forward_ivec3();
        for distance in 1..=30 {
            let target = pos + forward * distance;
            let Some(block) = world.blocks.get(&target).copied() else {
                continue;
            };
            if block.kind.is_material() {
                world.remove(&target);
                continue;
            }
            if block.kind.is_factory()
                || block.kind.is_scene()
                || block.kind == BlockKind::BlockerHead
            {
                break;
            }
        }
    }
}

impl SignalNetworkCache {
    fn refresh(&mut self, world: &WorldBlocks) {
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

    fn powered_components(&self, world: &WorldBlocks) -> HashSet<usize> {
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

    fn is_device_powered(&self, pos: IVec3, powered_components: &HashSet<usize>) -> bool {
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

fn push_powered_pistons(
    world: &mut WorldBlocks,
    signal_cache: &SignalNetworkCache,
    powered_components: &HashSet<usize>,
) {
    let pistons: Vec<(IVec3, Facing)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            (block.kind == BlockKind::Piston).then_some((*pos, block.facing))
        })
        .collect();

    for (pos, facing) in pistons {
        if !signal_cache.is_device_powered(pos, powered_components) {
            continue;
        }

        let source = pos + facing.forward_ivec3();
        let Some(block) = world.blocks.get(&source) else {
            continue;
        };
        if !block.kind.is_material() {
            continue;
        }

        let structure = material_structure(world, source);
        let offset = facing.forward_ivec3();
        if can_move_structure(world, &structure, offset) {
            move_structure(world, &structure, offset);
        }
    }
}

fn material_structure(world: &WorldBlocks, start: IVec3) -> HashSet<IVec3> {
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
                .blocks
                .get(&neighbor)
                .is_some_and(|block| block.kind.is_material())
            {
                structure.insert(neighbor);
                queue.push_back(neighbor);
            }
        }
    }

    structure
}

fn can_move_structure(world: &WorldBlocks, structure: &HashSet<IVec3>, offset: IVec3) -> bool {
    structure.iter().all(|pos| {
        let target = *pos + offset;
        target.y >= 0 && (structure.contains(&target) || world.can_place_solid_at(target))
    })
}

fn move_structure(world: &mut WorldBlocks, structure: &HashSet<IVec3>, offset: IVec3) {
    let blocks: Vec<(IVec3, BlockData)> = structure
        .iter()
        .filter_map(|pos| world.remove(pos).map(|block| (*pos, block)))
        .collect();

    for (pos, block) in blocks {
        world.insert(pos + offset, block);
    }
}

fn can_rotate_structure(world: &WorldBlocks, structure: &HashSet<IVec3>, pivot: IVec3) -> bool {
    structure.iter().all(|pos| {
        let target = rotate_pos_y(*pos, pivot);
        target.y >= 0 && (structure.contains(&target) || world.can_place_solid_at(target))
    })
}

fn rotate_structure(world: &mut WorldBlocks, structure: &HashSet<IVec3>, pivot: IVec3) {
    let blocks: Vec<(IVec3, BlockData)> = structure
        .iter()
        .filter_map(|pos| world.remove(pos).map(|block| (*pos, block)))
        .collect();

    for (pos, mut block) in blocks {
        block.facing = block.facing.rotate();
        world.insert(rotate_pos_y(pos, pivot), block);
    }
}

fn rotate_pos_y(pos: IVec3, pivot: IVec3) -> IVec3 {
    let rel = pos - pivot;
    pivot + IVec3::new(-rel.z, rel.y, rel.x)
}

fn is_wire_at(world: &WorldBlocks, pos: IVec3) -> bool {
    world
        .blocks
        .get(&pos)
        .is_some_and(|block| block.kind == BlockKind::Wire)
}

fn signal_offsets() -> [IVec3; 6] {
    [
        IVec3::X,
        IVec3::NEG_X,
        IVec3::Y,
        IVec3::NEG_Y,
        IVec3::Z,
        IVec3::NEG_Z,
    ]
}
