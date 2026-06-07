use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::blocks::{
    BlockData, BlockKind, MaterialDestroyer, MaterialLabeler, MaterialSource,
};
use crate::game::world::direction::Facing;
use crate::game::world::grid::{
    ConverterMode, MaterialFace, MaterialFaceMark, MaterialFaceMarkSource, WorldBlocks,
};

use super::runtime::PendingGeneratedMaterials;
use super::signal_offsets;
use super::structure_state::StructureState;
use super::structures::{execute_structure_moves, material_structure, MovementMark, StructureMove};

pub(super) fn run_material_behavior_phase(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    structure_state: &mut StructureState,
    pending_destroyed: &mut PendingGeneratedMaterials,
    ready_turn: u64,
) -> Vec<IVec3> {
    let mut sparks =
        run_material_destroy_phase(world, powered_devices, pending_destroyed, ready_turn);
    run_material_label_phase(world);
    run_material_conversion_phase(world);
    sparks.extend(run_material_acceptance_phase(
        world,
        structure_state,
        pending_destroyed,
        ready_turn,
    ));
    sparks
}

pub(super) fn run_material_teleport_phase(
    world: &mut WorldBlocks,
    structure_state: &mut StructureState,
) {
    run_material_teleport_phase_impl(world, structure_state);
}

pub(super) fn run_weld_behavior_phase(world: &mut WorldBlocks) -> Vec<IVec3> {
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

fn run_material_teleport_phase_impl(world: &mut WorldBlocks, structure_state: &mut StructureState) {
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
        let Some(exit) = world.teleport_partner(entrance) else {
            continue;
        };
        if !world
            .system_blocks
            .get(&exit)
            .is_some_and(|block| block.kind == BlockKind::TeleportExit)
        {
            continue;
        }

        let structure = structure_state
            .pushable_structure_at(entrance, IVec3::ZERO)
            .unwrap_or_else(|| material_structure(world, entrance));
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
            structure_state,
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
            MaterialDestroyer::Drill { target } => mark_material_destroy(
                world,
                pending_destroyed,
                pos + target,
                ready_turn,
                &mut sparks,
            ),
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

fn run_material_acceptance_phase(
    world: &mut WorldBlocks,
    structure_state: &mut StructureState,
    pending_destroyed: &mut PendingGeneratedMaterials,
    ready_turn: u64,
) -> Vec<IVec3> {
    let mut sparks = Vec::new();
    let acceptor_count = structure_state.acceptor_structures().len();
    for index in 0..acceptor_count {
        let Some(acceptor) = structure_state.acceptor_structures().get(index) else {
            continue;
        };
        let acceptor_positions = &acceptor.positions;
        let mut matched_material = HashSet::new();
        let mut sample_material_pos = None;

        for pos in acceptor_positions {
            let Some(block) = world.blocks.get(pos) else {
                break;
            };
            let Some(material) = block.kind.material_kind() else {
                break;
            };
            if !world.accepts_material_kind_at(*pos, material) {
                break;
            }
            matched_material.insert(*pos);
            sample_material_pos = Some(*pos);
        }

        if matched_material.len() != acceptor_positions.len() {
            continue;
        }
        let Some(sample_pos) = sample_material_pos else {
            continue;
        };
        let welded_material = material_structure(world, sample_pos);
        if welded_material != matched_material {
            continue;
        }

        for pos in &welded_material {
            pending_destroyed.mark_destroyed(*pos, ready_turn);
            sparks.push(*pos);
        }
        structure_state.increment_acceptor_count(index);
    }
    sparks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::blocks::{BlockData, BlockKind, MaterialKind};
    use crate::game::world::direction::Facing;
    use crate::game::world::grid::{GoalSettings, WorldBlocks};

    fn place_goal(world: &mut WorldBlocks, pos: IVec3, material: MaterialKind) {
        world.insert(
            pos,
            BlockData {
                kind: BlockKind::Goal,
                facing: Facing::North,
            },
        );
        world.set_goal_settings(pos, GoalSettings { material });
    }

    fn place_material(world: &mut WorldBlocks, pos: IVec3, material: MaterialKind) {
        let kind = BlockKind::material_block_kind(material).unwrap();
        world.insert(
            pos,
            BlockData {
                kind,
                facing: Facing::North,
            },
        );
    }

    fn acceptor_state(world: &WorldBlocks) -> StructureState {
        let mut state = StructureState::default();
        state.rebuild_for_simulation(world);
        state
    }

    #[test]
    fn acceptance_marks_matching_material_for_next_turn_removal() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, MaterialKind::Basic);
        place_material(&mut world, IVec3::ZERO, MaterialKind::Basic);
        let mut state = acceptor_state(&world);
        let mut pending = PendingGeneratedMaterials::default();

        let sparks = run_material_acceptance_phase(&mut world, &mut state, &mut pending, 2);

        assert!(world.is_material_at(IVec3::ZERO));
        assert_eq!(pending.pending_destroy_turn(IVec3::ZERO), Some(2));
        assert_eq!(state.acceptor_structures()[0].count, 1);
        assert_eq!(sparks, vec![IVec3::ZERO]);
    }

    #[test]
    fn acceptance_ignores_wrong_material() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, MaterialKind::Basic);
        place_material(&mut world, IVec3::ZERO, MaterialKind::Iron);
        let mut state = acceptor_state(&world);
        let mut pending = PendingGeneratedMaterials::default();

        run_material_acceptance_phase(&mut world, &mut state, &mut pending, 2);

        assert!(world.is_material_at(IVec3::ZERO));
        assert!(!pending.has_pending_destruction());
        assert_eq!(state.acceptor_structures()[0].count, 0);
    }

    #[test]
    fn acceptance_requires_entire_connected_acceptor_structure() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, MaterialKind::Basic);
        place_goal(&mut world, IVec3::X, MaterialKind::Basic);
        place_material(&mut world, IVec3::ZERO, MaterialKind::Basic);
        let mut state = acceptor_state(&world);
        let mut pending = PendingGeneratedMaterials::default();

        run_material_acceptance_phase(&mut world, &mut state, &mut pending, 2);

        assert!(world.is_material_at(IVec3::ZERO));
        assert!(!pending.has_pending_destruction());
        assert_eq!(state.acceptor_structures()[0].count, 0);
    }

    #[test]
    fn acceptance_requires_material_structure_without_extra_blocks() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, MaterialKind::Basic);
        place_material(&mut world, IVec3::ZERO, MaterialKind::Basic);
        place_material(&mut world, IVec3::X, MaterialKind::Basic);
        world.weld_materials(IVec3::ZERO, IVec3::X);
        let mut state = acceptor_state(&world);
        let mut pending = PendingGeneratedMaterials::default();

        run_material_acceptance_phase(&mut world, &mut state, &mut pending, 2);

        assert!(world.is_material_at(IVec3::ZERO));
        assert!(world.is_material_at(IVec3::X));
        assert!(!pending.has_pending_destruction());
        assert_eq!(state.acceptor_structures()[0].count, 0);
    }

    #[test]
    fn acceptance_marks_entire_welded_structure_for_next_turn_removal() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, MaterialKind::Basic);
        place_goal(&mut world, IVec3::X, MaterialKind::Basic);
        place_material(&mut world, IVec3::ZERO, MaterialKind::Basic);
        place_material(&mut world, IVec3::X, MaterialKind::Basic);
        world.weld_materials(IVec3::ZERO, IVec3::X);
        let mut state = acceptor_state(&world);
        let mut pending = PendingGeneratedMaterials::default();

        run_material_acceptance_phase(&mut world, &mut state, &mut pending, 2);

        assert!(world.is_material_at(IVec3::ZERO));
        assert!(world.is_material_at(IVec3::X));
        assert_eq!(pending.pending_destroy_turn(IVec3::ZERO), Some(2));
        assert_eq!(pending.pending_destroy_turn(IVec3::X), Some(2));
        assert_eq!(state.acceptor_structures()[0].count, 1);
    }
}
