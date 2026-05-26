use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::game::state::{BuilderMode, SimulationState};
use crate::game::world::animation::{pair_block_animations, SIMULATION_TURN_SECONDS};
use crate::game::world::blocks::{BlockData, BlockKind, Facing, MaterialKind};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    despawn_world, rebuild_world_with_animations, BlockEntity, WorldRenderAssets,
};

use super::signal_offsets;
pub use super::signals::SignalNetworkCache;
use super::structures::{
    apply_factory_gravity, apply_material_gravity, can_move_structure, can_rotate_structure,
    material_structure, move_structure, rotate_structure,
};

pub fn run_turn(
    world: &mut WorldBlocks,
    signal_cache: &mut SignalNetworkCache,
    turn: u64,
    commands: &mut Commands,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
) {
    let before = animation_snapshot(world);
    sync_generated_markers(world, signal_cache, &HashSet::new());
    signal_cache.refresh(world);
    let powered_components = signal_cache.powered_components(world);
    sync_generated_markers(world, signal_cache, &powered_components);
    drill_materials(world);
    fire_lasers(world, signal_cache, &powered_components);
    push_powered_pistons(world, signal_cache, &powered_components);
    weld_material_structures(world);
    lift_structures(world);
    rotate_structures(world);
    spawn_materials(world, turn);
    move_conveyed_materials(world);
    apply_material_gravity(world);
    apply_factory_gravity(world);
    signal_cache.refresh(world);

    let animations = pair_block_animations(&before, &animation_snapshot(world));
    despawn_world(commands, block_entities);
    rebuild_world_with_animations(commands, world, render_assets, &animations);
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

    simulation.accumulator += time.delta_seconds() * simulation.speed / SIMULATION_TURN_SECONDS;
    while simulation.accumulator >= 1.0 {
        simulation.turn += 1;
        simulation.accumulator -= 1.0;
        run_turn(
            &mut world,
            &mut signal_cache,
            simulation.turn,
            &mut commands,
            &block_entities,
            &render_assets,
        );
    }
}

fn animation_snapshot(world: &WorldBlocks) -> HashMap<IVec3, (BlockKind, Facing)> {
    world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            (!block.kind.is_generated_marker()).then_some((*pos, (block.kind, block.facing)))
        })
        .collect()
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

fn spawn_materials(world: &mut WorldBlocks, turn: u64) {
    if turn == 0 {
        return;
    }

    let generators: Vec<(IVec3, Facing)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            (block.kind == BlockKind::Generator).then_some((*pos, block.facing))
        })
        .collect();

    for (pos, facing) in generators {
        let settings = world.generator_settings(pos);
        if turn % settings.period.max(1) != 0 {
            continue;
        }

        let spawn_pos = pos + facing.forward_ivec3();
        if world.can_place_solid_at(spawn_pos) {
            world.insert(
                spawn_pos,
                BlockData {
                    kind: material_block_kind(settings.material),
                    facing: Facing::North,
                },
            );
        }
    }
}

fn material_block_kind(material: MaterialKind) -> BlockKind {
    match material {
        MaterialKind::Basic => BlockKind::Material,
    }
}

fn weld_material_structures(world: &mut WorldBlocks) {
    let weld_points: Vec<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| (block.kind == BlockKind::WeldPoint).then_some(*pos))
        .collect();

    for weld_point in weld_points {
        let Some(material_a) = adjacent_material(world, weld_point) else {
            continue;
        };

        for offset in signal_offsets() {
            let neighbor = weld_point + offset;
            if !world
                .blocks
                .get(&neighbor)
                .is_some_and(|block| block.kind == BlockKind::WeldPoint)
            {
                continue;
            }

            let Some(material_b) = adjacent_material_except(world, neighbor, material_a) else {
                continue;
            };
            world.weld_materials(material_a, material_b);
        }
    }
}

fn adjacent_material(world: &WorldBlocks, pos: IVec3) -> Option<IVec3> {
    signal_offsets()
        .into_iter()
        .map(|offset| pos + offset)
        .find(|candidate| world.is_material_at(*candidate))
}

fn adjacent_material_except(world: &WorldBlocks, pos: IVec3, except: IVec3) -> Option<IVec3> {
    signal_offsets()
        .into_iter()
        .map(|offset| pos + offset)
        .find(|candidate| *candidate != except && world.is_material_at(*candidate))
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

        let structure = material_structure(world, source);
        let offset = facing.forward_ivec3();
        if can_move_structure(world, &structure, offset) {
            move_structure(world, &structure, offset);
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
