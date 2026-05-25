use bevy::prelude::*;

use crate::blocks::{BlockData, BlockKind, Facing};
use crate::rendering::{despawn_world, rebuild_world, BlockEntity};
use crate::state::{BuilderMode, SimulationState};
use crate::world::WorldBlocks;

pub fn run_turn(
    world: &mut WorldBlocks,
    commands: &mut Commands,
    block_entities: &Query<Entity, With<BlockEntity>>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    sync_weld_points(world);
    spawn_materials(world);
    move_conveyed_materials(world);
    apply_material_gravity(world);

    despawn_world(commands, block_entities);
    rebuild_world(commands, world, meshes, materials);
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
            &mut commands,
            &block_entities,
            &mut meshes,
            &mut materials,
        );
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
