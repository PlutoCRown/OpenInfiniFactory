use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::game::state::{BuilderMode, SimulationState};
use crate::game::world::animation::{
    pair_block_animations, AnimationTiming, SIMULATION_TURN_SECONDS,
};
use crate::game::world::blocks::{
    BlockData, BlockKind, MarkerBehavior, MaterialDestroyer, MaterialKind, MaterialMover,
    MaterialLabeler, MaterialSource,
};
use crate::game::world::direction::Facing;
use crate::game::world::grid::{
    MaterialFace, MaterialFaceMark, MaterialFaceMarkSource, WorldBlocks,
};
use crate::game::world::rendering::{
    despawn_world, rebuild_world_with_timed_animations, BlockEntity, WorldRenderAssets,
};

use super::signal_offsets;
pub use super::signals::SignalNetworkCache;
use super::structures::{
    execute_structure_moves, factory_gravity_moves, material_gravity_moves, material_structure,
    StructureMove,
};

pub fn run_turn(
    world: &mut WorldBlocks,
    signal_cache: &mut SignalNetworkCache,
    turn: u64,
    commands: &mut Commands,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
    animation_duration: f32,
) {
    let before = animation_snapshot(world);
    world.clear_generated_markers();

    let mut movement_plan = mark_gravity_phase(world);

    signal_cache.refresh(world);
    let powered_components = signal_cache.powered_components(world);
    let powered_devices = signal_cache.powered_devices(&powered_components);

    run_powered_marker_phase(world, &powered_devices);
    movement_plan.extend(mark_material_movement_phase(world, &powered_devices));
    execute_structure_moves(world, movement_plan);

    run_static_marker_phase(world);
    run_powered_marker_phase(world, &powered_devices);
    run_material_acceptance_phase(world);
    run_material_source_phase(world, turn);
    run_weld_phase(world);
    run_material_destroy_phase(world, &powered_devices);
    run_material_label_phase(world);
    run_material_conversion_phase(world);
    run_material_teleport_phase(world);
    run_material_acceptance_phase(world);

    signal_cache.refresh(world);

    let animations = pair_block_animations(&before, &animation_snapshot(world));
    despawn_world(commands, block_entities);
    rebuild_world_with_timed_animations(
        commands,
        world,
        render_assets,
        &animations,
        AnimationTiming::simulation(animation_duration),
    );
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
            SIMULATION_TURN_SECONDS / simulation.speed.max(0.001),
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
        .system_blocks
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
            MaterialSource::Generator => {
                let settings = world.generator_settings(pos);
                if turn % settings.period.max(1) != 0 {
                    continue;
                }

                let spawn_pos = pos;
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

fn block_material_kind(kind: BlockKind) -> Option<MaterialKind> {
    match kind {
        BlockKind::Material => Some(MaterialKind::Basic),
        BlockKind::IronMaterial => Some(MaterialKind::Iron),
        BlockKind::CopperMaterial => Some(MaterialKind::Copper),
        _ => None,
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

fn mark_gravity_phase(world: &WorldBlocks) -> Vec<StructureMove> {
    let mut moves = material_gravity_moves(world);
    moves.extend(factory_gravity_moves(world));
    moves
}

fn mark_material_movement_phase(
    world: &WorldBlocks,
    powered_devices: &HashSet<IVec3>,
) -> Vec<StructureMove> {
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
    let mut moves = Vec::new();

    for (pos, mover) in movers {
        match mover {
            MaterialMover::Conveyor { source, offset } => {
                if let Some(movement) = mark_material_translate(world, pos + source, offset) {
                    moves.push(movement);
                }
            }
            MaterialMover::Lifter => {
                if let Some(movement) = mark_lift_material_structure(world, pos) {
                    moves.push(movement);
                }
            }
            MaterialMover::Rotator { clockwise } => {
                if let Some(movement) = mark_rotate_material_structure(world, pos, clockwise) {
                    moves.push(movement);
                }
            }
            MaterialMover::Piston { source, offset } => {
                if powered_devices.contains(&pos) {
                    if let Some(movement) = mark_material_translate(world, pos + source, offset) {
                        moves.push(movement);
                    }
                }
            }
        }
    }
    moves
}

fn mark_material_translate(
    world: &WorldBlocks,
    source: IVec3,
    offset: IVec3,
) -> Option<StructureMove> {
    if !world.is_material_at(source) {
        return None;
    }

    let structure = material_structure(world, source);
    Some(StructureMove::translate(structure, offset))
}

fn remove_material_structure(world: &mut WorldBlocks, structure: &HashSet<IVec3>) {
    for pos in structure {
        world.remove(pos);
    }
}

fn run_material_label_phase(world: &mut WorldBlocks) {
    let labelers: Vec<(IVec3, MaterialLabeler)> = world
        .system_blocks
        .iter()
        .filter_map(|(pos, block)| {
            block
                .kind
                .material_labeler(block.facing)
                .map(|labeler| (*pos, labeler))
        })
        .collect();

    for (pos, labeler) in labelers {
        let (target_offset, source) = match labeler {
            MaterialLabeler::Stamper { target } => (target, MaterialFaceMarkSource::Stamper),
            MaterialLabeler::Roller { target } => (target, MaterialFaceMarkSource::Roller),
        };
        let target = pos + target_offset;
        if !world.is_material_at(target) {
            continue;
        }

        let face = MaterialFace::new(target, -target_offset);
        if world
            .material_face_marks
            .get(&face)
            .is_some_and(|mark| mark.source == MaterialFaceMarkSource::Stamper)
        {
            continue;
        }

        let settings = world.labeler_settings(pos);
        world.set_material_face_mark(
            face,
            MaterialFaceMark {
                color: settings.color,
                source,
            },
        );
    }
}

fn run_material_conversion_phase(world: &mut WorldBlocks) {
    let converters: Vec<IVec3> = world
        .system_blocks
        .iter()
        .filter_map(|(pos, block)| (block.kind == BlockKind::Converter).then_some(*pos))
        .collect();

    for pos in converters {
        let Some(mut block) = world.blocks.get(&pos).copied() else {
            continue;
        };
        let Some(input_material) = block_material_kind(block.kind) else {
            continue;
        };

        let settings = world.converter_settings(pos);
        if settings.mode == crate::game::world::grid::ConverterMode::SpecificInput
            && input_material != settings.input
        {
            continue;
        }

        block.kind = material_block_kind(settings.output);
        world.insert(pos, block);
    }
}

fn run_material_teleport_phase(world: &mut WorldBlocks) {
    let entrances: Vec<IVec3> = world
        .system_blocks
        .iter()
        .filter_map(|(pos, block)| (block.kind == BlockKind::TeleportEntrance).then_some(*pos))
        .collect();
    let mut handled = HashSet::new();

    for entrance in entrances {
        if handled.contains(&entrance) || !world.is_material_at(entrance) {
            continue;
        }
        let Some(exit) = world.teleport_settings(entrance).pair else {
            continue;
        };
        if !world
            .system_blocks
            .get(&exit)
            .is_some_and(|block| block.kind == BlockKind::TeleportExit)
        {
            continue;
        }

        let structure = material_structure(world, entrance);
        let offset = exit - entrance;
        handled.extend(structure.iter().copied());
        handled.extend(structure.iter().map(|pos| *pos + offset));
        execute_structure_moves(world, vec![StructureMove::translate(structure, offset)]);
    }
}

fn mark_lift_material_structure(world: &WorldBlocks, pos: IVec3) -> Option<StructureMove> {
    let Some(source) = (1..=5)
        .map(|height| pos + IVec3::Y * height)
        .find(|candidate| world.is_material_at(*candidate))
    else {
        return None;
    };

    mark_material_translate(world, source, IVec3::Y)
}

fn mark_rotate_material_structure(
    world: &WorldBlocks,
    pos: IVec3,
    clockwise: bool,
) -> Option<StructureMove> {
    let source = pos + IVec3::Y;
    if !world.is_material_at(source) {
        return None;
    }

    let structure = material_structure(world, source);
    Some(StructureMove::rotate(structure, pos, clockwise))
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

fn run_material_acceptance_phase(world: &mut WorldBlocks) {
    let accepted: Vec<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            (block.kind.is_material() && world.accepts_material_at(*pos)).then_some(*pos)
        })
        .collect();

    for pos in accepted {
        if world.is_material_at(pos) {
            let structure = material_structure(world, pos);
            remove_material_structure(world, &structure);
        }
    }
}
