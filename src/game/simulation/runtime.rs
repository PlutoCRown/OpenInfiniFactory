use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::game::state::{BuilderMode, SimulationState};
use crate::game::world::animation::{pair_block_animations, SIMULATION_TURN_SECONDS};
use crate::game::world::blocks::{
    BlockData, BlockKind, MarkerBehavior, MaterialDestroyer, MaterialKind, MaterialMover,
    MaterialSource,
};
use crate::game::world::direction::Facing;
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
    run_static_marker_phase(world);
    signal_cache.refresh(world);
    let powered_components = signal_cache.powered_components(world);
    let powered_devices = signal_cache.powered_devices(&powered_components);

    run_powered_marker_phase(world, &powered_devices);
    run_material_destroy_phase(world, &powered_devices);
    run_weld_phase(world);
    run_material_source_phase(world, turn);
    run_material_movement_phase(world, &powered_devices);
    run_gravity_phase(world);
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

fn run_static_marker_phase(world: &mut WorldBlocks) {
    world.clear_generated_markers();

    let markers: Vec<(IVec3, MarkerBehavior)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            block
                .kind
                .marker_behavior(block.facing)
                .map(|marker| (*pos, marker))
        })
        .collect();

    for (pos, marker) in markers {
        if matches!(marker, MarkerBehavior::BlockerHead { .. }) {
            continue;
        }
        place_generated_marker(world, pos, marker);
    }
}

fn run_powered_marker_phase(world: &mut WorldBlocks, powered_devices: &HashSet<IVec3>) {
    let markers: Vec<(IVec3, MarkerBehavior)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            block
                .kind
                .marker_behavior(block.facing)
                .map(|marker| (*pos, marker))
        })
        .collect();

    for (pos, marker) in markers {
        if !matches!(marker, MarkerBehavior::BlockerHead { .. }) {
            continue;
        }
        if !powered_devices.contains(&pos) {
            place_generated_marker(world, pos, marker);
        }
    }
}

fn place_generated_marker(world: &mut WorldBlocks, origin: IVec3, marker: MarkerBehavior) {
    let (offset, kind, facing, solid) = match marker {
        MarkerBehavior::WeldPoint { offset, facing } => {
            (offset, BlockKind::WeldPoint, facing, false)
        }
        MarkerBehavior::BlockerHead { offset, facing } => {
            (offset, BlockKind::BlockerHead, facing, true)
        }
        MarkerBehavior::DrillHead { offset, facing } => {
            (offset, BlockKind::DrillHead, facing, false)
        }
    };

    let pos = origin + offset;
    let can_place = if solid {
        world.can_place_solid_at(pos)
    } else {
        !world.is_occupied(pos)
    };
    if can_place {
        world.insert(pos, BlockData { kind, facing });
    }
}

fn run_material_source_phase(world: &mut WorldBlocks, turn: u64) {
    if turn == 0 {
        return;
    }

    let sources: Vec<(IVec3, MaterialSource)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            block
                .kind
                .material_source(block.facing)
                .map(|source| (*pos, source))
        })
        .collect();

    for (pos, source) in sources {
        match source {
            MaterialSource::Generator { output } => {
                let settings = world.generator_settings(pos);
                if turn % settings.period.max(1) != 0 {
                    continue;
                }

                let spawn_pos = pos + output;
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
    }
}

fn material_block_kind(material: MaterialKind) -> BlockKind {
    match material {
        MaterialKind::Basic => BlockKind::Material,
        MaterialKind::Iron => BlockKind::IronMaterial,
        MaterialKind::Copper => BlockKind::CopperMaterial,
    }
}

fn run_weld_phase(world: &mut WorldBlocks) {
    let weld_points: Vec<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            block
                .kind
                .weld_behavior()
                .is_some()
                .then_some(*pos)
        })
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
                .is_some_and(|block| block.kind.weld_behavior().is_some())
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

fn run_material_movement_phase(world: &mut WorldBlocks, powered_devices: &HashSet<IVec3>) {
    let movers: Vec<(IVec3, MaterialMover)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            block
                .kind
                .material_mover(block.facing)
                .map(|mover| (*pos, mover))
        })
        .collect();

    for (pos, mover) in movers {
        match mover {
            MaterialMover::Conveyor { source, offset } => {
                move_material_structure(world, pos + source, offset);
            }
            MaterialMover::Lifter => lift_material_structure(world, pos),
            MaterialMover::Rotator { clockwise } => {
                rotate_material_structure(world, pos, clockwise);
            }
            MaterialMover::Piston { source, offset } => {
                if powered_devices.contains(&pos) {
                    move_material_structure(world, pos + source, offset);
                }
            }
        }
    }
}

fn move_material_structure(world: &mut WorldBlocks, source: IVec3, offset: IVec3) {
    if !world.is_material_at(source) {
        return;
    }

    let structure = material_structure(world, source);
    if can_move_structure(world, &structure, offset) {
        move_structure(world, &structure, offset);
    }
}

fn lift_material_structure(world: &mut WorldBlocks, pos: IVec3) {
    let Some(source) = (1..=5)
        .map(|height| pos + IVec3::Y * height)
        .find(|candidate| world.is_material_at(*candidate))
    else {
        return;
    };

    move_material_structure(world, source, IVec3::Y);
}

fn rotate_material_structure(world: &mut WorldBlocks, pos: IVec3, clockwise: bool) {
    let source = pos + IVec3::Y;
    if !world.is_material_at(source) {
        return;
    }

    let structure = material_structure(world, source);
    if can_rotate_structure(world, &structure, pos, clockwise) {
        rotate_structure(world, &structure, pos, clockwise);
    }
}

fn run_material_destroy_phase(world: &mut WorldBlocks, powered_devices: &HashSet<IVec3>) {
    let destroyers: Vec<(IVec3, MaterialDestroyer)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            block
                .kind
                .material_destroyer(block.facing)
                .map(|destroyer| (*pos, destroyer))
        })
        .collect();

    for (pos, destroyer) in destroyers {
        match destroyer {
            MaterialDestroyer::Drill { target } => remove_material_at(world, pos + target),
            MaterialDestroyer::AdjacentDrillHead => {
                for offset in signal_offsets() {
                    remove_material_at(world, pos + offset);
                }
            }
            MaterialDestroyer::Laser { direction, range } => {
                if powered_devices.contains(&pos) {
                    fire_laser(world, pos, direction, range);
                }
            }
        }
    }
}

fn remove_material_at(world: &mut WorldBlocks, pos: IVec3) {
    if world.is_material_at(pos) {
        world.remove(&pos);
    }
}

fn fire_laser(world: &mut WorldBlocks, pos: IVec3, direction: IVec3, range: i32) {
    for distance in 1..=range {
        let target = pos + direction * distance;
        let Some(block) = world.blocks.get(&target).copied() else {
            continue;
        };
        if block.kind.is_material() {
            world.remove(&target);
            continue;
        }
        if block.kind.blocks_laser() {
            break;
        }
    }
}

fn run_gravity_phase(world: &mut WorldBlocks) {
    apply_material_gravity(world);
    apply_factory_gravity(world);
}
