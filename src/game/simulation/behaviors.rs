use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::world::blocks::{
    BlockData, BlockKind, MaterialDestroyer, MaterialLabeler, MaterialSource,
};
use crate::game::world::direction::Facing;
use crate::game::world::grid::{
    ConverterMode, MaterialFace, MaterialFaceMark, MaterialFaceMarkSource, WorldBlocks,
};

use super::factory_activity::FactoryStructureState;
use super::runtime::PendingGeneratedMaterials;
use super::signal_offsets;
use super::structures::{execute_structure_moves, material_structure, MovementMark, StructureMove};

pub(super) fn run_material_behavior_phase(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    factory_structures: &mut FactoryStructureState,
    pending_destroyed: &mut PendingGeneratedMaterials,
    ready_turn: u64,
) -> Vec<IVec3> {
    run_material_acceptance_phase(world);
    let drill_sparks =
        run_material_destroy_phase(world, powered_devices, pending_destroyed, ready_turn);
    run_material_label_phase(world);
    run_material_conversion_phase(world);
    run_material_teleport_phase(world, factory_structures);
    run_material_acceptance_phase(world);
    drill_sparks
}

pub(super) fn run_weld_behavior_phase(world: &mut WorldBlocks) -> Vec<IVec3> {
    run_material_acceptance_phase(world);
    run_weld_phase(world)
}

#[derive(Clone, Copy)]
pub(super) struct GeneratedMaterial {
    pub pos: IVec3,
    pub block: BlockData,
}

pub(super) fn material_source_generation(
    world: &WorldBlocks,
    turn: u64,
    blocked_generation: &HashSet<IVec3>,
) -> Vec<GeneratedMaterial> {
    let mut generated = Vec::new();
    if turn == 0 {
        return generated;
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
                if world.can_place_platform_at(spawn_pos)
                    && !blocked_generation.contains(&spawn_pos)
                {
                    let Some(kind) = BlockKind::material_block_kind(settings.material) else {
                        continue;
                    };
                    generated.push(GeneratedMaterial {
                        pos: spawn_pos,
                        block: BlockData {
                            kind,
                            facing: Facing::North,
                        },
                    });
                }
            }
        }
    }
    generated
}

fn run_weld_phase(world: &mut WorldBlocks) -> Vec<IVec3> {
    let weld_points: Vec<IVec3> = world
        .system_blocks
        .iter()
        .filter_map(|(pos, block)| block.kind.weld_behavior().is_some().then_some(*pos))
        .collect();
    let mut sparks = Vec::new();

    for weld_point in weld_points {
        if !world.is_material_at(weld_point) {
            continue;
        }

        for offset in signal_offsets() {
            let neighbor = weld_point + offset;
            if !world
                .system_blocks
                .get(&neighbor)
                .is_some_and(|block| block.kind.weld_behavior().is_some())
            {
                continue;
            }

            if !world.is_material_at(neighbor) {
                continue;
            }
            let was_new =
                !world
                    .material_welds
                    .contains(&crate::game::world::grid::MaterialWeld::new(
                        weld_point, neighbor,
                    ));
            world.weld_materials(weld_point, neighbor);
            if was_new {
                sparks.push(weld_point);
                sparks.push(neighbor);
            }
        }
    }
    sparks
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
        let Some(input_material) = block.kind.material_kind() else {
            continue;
        };

        let settings = world.converter_settings(pos);
        if settings.mode == ConverterMode::SpecificInput && input_material != settings.input {
            continue;
        }

        let Some(output_kind) = BlockKind::material_block_kind(settings.output) else {
            continue;
        };
        block.kind = output_kind;
        world.insert(pos, block);
    }
}

fn run_material_teleport_phase(
    world: &mut WorldBlocks,
    factory_structures: &mut FactoryStructureState,
) {
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
        let _ = execute_structure_moves(
            world,
            vec![StructureMove::translate_marked(
                structure,
                offset,
                MovementMark::Push,
            )],
            factory_structures,
        );
    }
}

fn run_material_destroy_phase(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    pending_destroyed: &mut PendingGeneratedMaterials,
    ready_turn: u64,
) -> Vec<IVec3> {
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

    let mut sparks = Vec::new();
    for (pos, destroyer) in destroyers {
        match destroyer {
            MaterialDestroyer::Drill { target } => {
                mark_material_destroy(world, pending_destroyed, pos + target, ready_turn, &mut sparks)
            }
            MaterialDestroyer::AdjacentDrillHead => {
                for offset in signal_offsets() {
                    mark_material_destroy(
                        world,
                        pending_destroyed,
                        pos + offset,
                        ready_turn,
                        &mut sparks,
                    );
                }
            }
            MaterialDestroyer::Laser { direction, range } => {
                if powered_devices.contains(&pos) {
                    fire_laser(world, pos, direction, range);
                }
            }
        }
    }
    sparks
}

fn mark_material_destroy(
    world: &WorldBlocks,
    pending_destroyed: &mut PendingGeneratedMaterials,
    pos: IVec3,
    ready_turn: u64,
    sparks: &mut Vec<IVec3>,
) {
    if world.is_material_at(pos) {
        pending_destroyed.mark_destroyed(pos, ready_turn);
        sparks.push(pos);
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
            let material = block.kind.material_kind()?;
            (world.accepts_material_kind_at(*pos, material)).then_some(*pos)
        })
        .collect();

    for pos in accepted {
        if world.is_material_at(pos) {
            let structure = material_structure(world, pos);
            remove_material_structure(world, &structure);
        }
    }
}
