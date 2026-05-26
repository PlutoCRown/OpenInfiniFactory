use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

use crate::game::state::{BuilderMode, SimulationState};
use crate::game::world::blocks::{BlockData, BlockKind, Facing};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{despawn_world, rebuild_world, BlockEntity, WorldRenderAssets};

pub fn run_turn(
    world: &mut WorldBlocks,
    commands: &mut Commands,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
) {
    sync_weld_points(world);
    let powered_wires = powered_wires(world);
    push_powered_pistons(world, &powered_wires);
    spawn_materials(world);
    move_conveyed_materials(world);
    apply_material_gravity(world);

    despawn_world(commands, block_entities);
    rebuild_world(commands, world, render_assets);
}

pub fn reset_simulation(world: &mut WorldBlocks) {
    world.blocks.retain(|_, block| !block.kind.is_material());
    world.clear_generated_markers();
}

pub fn tick_simulation(
    time: Res<Time>,
    builder_mode: Res<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut world: ResMut<WorldBlocks>,
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
        run_turn(&mut world, &mut commands, &block_entities, &render_assets);
    }
}

fn sync_weld_points(world: &mut WorldBlocks) {
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
            world.blocks.insert(
                point_pos,
                BlockData {
                    kind: BlockKind::WeldPoint,
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
            world.blocks.insert(
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
            world.blocks.remove(&pos);
            world.blocks.insert(next, block);
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
            world.blocks.remove(&source);
            world.blocks.insert(target, block);
        }
    }
}

fn powered_wires(world: &WorldBlocks) -> HashSet<IVec3> {
    let mut powered = HashSet::new();
    let mut queue = VecDeque::new();

    for (pos, block) in &world.blocks {
        if block.kind != BlockKind::Detector {
            continue;
        }

        let detected_pos = *pos + block.facing.forward_ivec3();
        if !world
            .blocks
            .get(&detected_pos)
            .is_some_and(|detected| detected.kind.is_material())
        {
            continue;
        }

        for offset in signal_offsets() {
            if offset == block.facing.forward_ivec3() {
                continue;
            }

            let wire_pos = *pos + offset;
            if is_wire_at(world, wire_pos) && powered.insert(wire_pos) {
                queue.push_back(wire_pos);
            }
        }
    }

    while let Some(pos) = queue.pop_front() {
        for offset in signal_offsets() {
            let neighbor = pos + offset;
            if is_wire_at(world, neighbor) && powered.insert(neighbor) {
                queue.push_back(neighbor);
            }
        }
    }

    powered
}

fn push_powered_pistons(world: &mut WorldBlocks, powered_wires: &HashSet<IVec3>) {
    let pistons: Vec<(IVec3, Facing)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            (block.kind == BlockKind::Piston).then_some((*pos, block.facing))
        })
        .collect();

    for (pos, facing) in pistons {
        if !is_piston_powered(world, powered_wires, pos, facing) {
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

fn is_piston_powered(
    world: &WorldBlocks,
    powered_wires: &HashSet<IVec3>,
    pos: IVec3,
    facing: Facing,
) -> bool {
    signal_offsets()
        .into_iter()
        .filter(|offset| *offset != facing.forward_ivec3())
        .map(|offset| pos + offset)
        .any(|wire_pos| is_wire_at(world, wire_pos) && powered_wires.contains(&wire_pos))
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
        .filter_map(|pos| world.blocks.remove(pos).map(|block| (*pos, block)))
        .collect();

    for (pos, block) in blocks {
        world.blocks.insert(pos + offset, block);
    }
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
