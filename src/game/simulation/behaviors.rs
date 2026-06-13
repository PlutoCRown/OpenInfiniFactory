use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::blocks::{
    BlockData, BlockKind, MaterialDestroyer, MaterialLabeler, MaterialSource,
};
use crate::game::world::direction::Facing;
use crate::game::world::grid::{
    ConverterMode, MaterialFace, MaterialFaceMark, MaterialFaceMarkSource, WorldBlocks,
};

use super::signal_offsets;
use super::structure_state::StructureState;
use super::structures::material_structure;

#[derive(Clone, Copy)]
pub struct LaserBeam {
    pub pos: IVec3,
    pub direction: IVec3,
    pub range: i32,
}

pub(super) struct MaterialBehaviorEffects {
    pub sparks: Vec<IVec3>,
    pub laser_beams: Vec<LaserBeam>,
    pub weld_sparks: Vec<IVec3>,
    pub acceptance_sparks: Vec<IVec3>,
}

pub(super) fn run_material_behavior_phase(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    structure_state: &mut StructureState,
) -> MaterialBehaviorEffects {
    run_material_label_phase(world);
    run_material_conversion_phase(world);
    let weld_sparks = run_weld_phase(world);
    let destroy_effects = run_material_destroy_phase(world, powered_devices);
    run_material_teleports(world);
    let acceptance_sparks = run_material_acceptance_phase(world, structure_state);
    MaterialBehaviorEffects {
        sparks: destroy_effects.sparks,
        laser_beams: destroy_effects.laser_beams,
        weld_sparks,
        acceptance_sparks,
    }
}

pub(super) fn run_material_teleports(world: &mut WorldBlocks) {
    let entrances: Vec<IVec3> = world
        .system_blocks
        .iter()
        .filter_map(|(pos, block)| (block.kind == BlockKind::TeleportEntrance).then_some(*pos))
        .collect();

    for entrance in entrances {
        if !world.anchors_material_at_teleport_entrance(entrance) {
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
        teleport_entrance_material(world, entrance, exit);
    }
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

fn detach_material_block(world: &mut WorldBlocks, pos: IVec3) {
    world.material_welds.retain(|weld| !weld.contains(pos));
}

fn teleport_entrance_material(world: &mut WorldBlocks, entrance: IVec3, exit: IVec3) -> bool {
    if !world.anchors_material_at_teleport_entrance(entrance) {
        return false;
    }
    if world.is_material_at(exit) || !world.can_move_into(exit) {
        return false;
    }

    detach_material_block(world, entrance);

    let updated_marks = world
        .material_face_marks
        .iter()
        .map(|(face, mark)| {
            let face = if face.pos == entrance {
                MaterialFace {
                    pos: exit,
                    normal: face.normal,
                }
            } else {
                *face
            };
            (face, *mark)
        })
        .collect();

    let Some(block) = world.remove(&entrance) else {
        return false;
    };
    world.insert(exit, block);
    world.replace_material_face_marks(updated_marks);
    true
}

fn run_material_destroy_phase(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
) -> MaterialBehaviorEffects {
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
    let mut laser_beams = Vec::new();
    for (pos, destroyer) in destroyers {
        match destroyer {
            MaterialDestroyer::Drill { target } => {
                destroy_material_at(world, pos + target, &mut sparks);
            }
            MaterialDestroyer::AdjacentDrillHead => {
                for offset in signal_offsets() {
                    destroy_material_at(world, pos + offset, &mut sparks);
                }
            }
            MaterialDestroyer::Laser { direction, range } => {
                if powered_devices.contains(&pos) {
                    laser_beams.push(LaserBeam {
                        pos,
                        direction,
                        range,
                    });
                    fire_laser(world, pos, direction, range);
                }
            }
        }
    }
    MaterialBehaviorEffects {
        sparks,
        laser_beams,
        weld_sparks: Vec::new(),
        acceptance_sparks: Vec::new(),
    }
}

fn destroy_material_at(world: &mut WorldBlocks, pos: IVec3, sparks: &mut Vec<IVec3>) {
    if world.is_material_at(pos) {
        detach_material_block(world, pos);
        world.remove(&pos);
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
) -> Vec<IVec3> {
    let mut acceptance_sparks = Vec::new();
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
            if world.is_material_at(*pos) {
                detach_material_block(world, *pos);
                world.remove(pos);
                acceptance_sparks.push(*pos);
            }
        }
        structure_state.increment_acceptor_count(index);
    }
    acceptance_sparks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::blocks::{BlockData, BlockKind, MaterialKind};
    use crate::game::simulation::structures::material_structure;
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

    fn place_teleport_pair(world: &mut WorldBlocks, entrance: IVec3, exit: IVec3) {
        world.insert(
            entrance,
            BlockData {
                kind: BlockKind::TeleportEntrance,
                facing: Facing::North,
            },
        );
        world.insert(
            exit,
            BlockData {
                kind: BlockKind::TeleportExit,
                facing: Facing::North,
            },
        );
        world.set_teleport_pair(entrance, Some(exit));
    }

    fn run_teleport(world: &mut WorldBlocks) {
        run_material_teleports(world);
    }

    #[test]
    fn teleport_moves_material_immediately() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);
        place_material(&mut world, entrance, MaterialKind::Basic);

        run_teleport(&mut world);

        assert!(!world.is_material_at(entrance));
        assert!(world.is_material_at(exit));
    }

    #[test]
    fn teleport_moves_only_entrance_block_from_welded_structure() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);
        place_material(&mut world, entrance, MaterialKind::Basic);
        place_material(&mut world, entrance + IVec3::X, MaterialKind::Basic);
        world.weld_materials(entrance, entrance + IVec3::X);

        run_teleport(&mut world);

        assert!(!world.is_material_at(entrance));
        assert!(world.is_material_at(exit));
        assert!(world.is_material_at(entrance + IVec3::X));
    }

    #[test]
    fn teleport_waits_when_exit_is_occupied() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);
        place_material(&mut world, entrance, MaterialKind::Basic);
        place_material(&mut world, exit, MaterialKind::Iron);

        run_teleport(&mut world);

        assert!(world.is_material_at(entrance));
        assert!(world.is_material_at(exit));
    }

    #[test]
    fn teleport_can_run_three_times_when_exit_clears_between() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);

        for expected in [
            MaterialKind::Basic,
            MaterialKind::Iron,
            MaterialKind::Copper,
        ] {
            place_material(&mut world, entrance, expected);
            run_teleport(&mut world);
            assert!(!world.is_material_at(entrance));
            assert_eq!(
                world
                    .blocks
                    .get(&exit)
                    .and_then(|block| block.kind.material_kind()),
                Some(expected)
            );
            world.remove(&exit);
        }
    }

    #[test]
    fn teleport_retries_after_exit_clears() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);
        place_material(&mut world, entrance, MaterialKind::Basic);
        place_material(&mut world, exit, MaterialKind::Iron);

        run_teleport(&mut world);
        assert!(world.is_material_at(entrance));

        world.remove(&exit);
        run_teleport(&mut world);

        assert!(!world.is_material_at(entrance));
        assert_eq!(
            world
                .blocks
                .get(&exit)
                .and_then(|block| block.kind.material_kind()),
            Some(MaterialKind::Basic)
        );
    }

    #[test]
    fn anchored_entrance_material_is_not_pushed_with_welded_neighbor() {
        use crate::game::simulation::structures::can_translate_structure;

        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let neighbor = IVec3::new(0, 0, 0);
        place_teleport_pair(&mut world, entrance, IVec3::new(5, 0, 0));
        place_material(&mut world, entrance, MaterialKind::Basic);
        place_material(&mut world, neighbor, MaterialKind::Basic);
        world.weld_materials(entrance, neighbor);
        let state = acceptor_state(&world);
        let structure = material_structure(&world, neighbor);

        assert!(!can_translate_structure(
            &world,
            &structure,
            IVec3::X,
            &state
        ));
    }

    #[test]
    fn teleport_detaches_before_moving_to_exit() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);
        place_material(&mut world, entrance, MaterialKind::Basic);
        place_material(&mut world, entrance + IVec3::X, MaterialKind::Basic);
        world.weld_materials(entrance, entrance + IVec3::X);

        run_teleport(&mut world);

        assert!(!world.is_material_at(entrance));
        assert!(world.is_material_at(exit));
        assert!(world.is_material_at(entrance + IVec3::X));
    }

    #[test]
    fn teleport_does_not_move_unwelded_neighbor_on_entrance() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);
        place_material(&mut world, entrance, MaterialKind::Basic);
        place_material(&mut world, entrance + IVec3::Y, MaterialKind::Basic);

        run_teleport(&mut world);

        assert!(!world.is_material_at(entrance));
        assert!(world.is_material_at(exit));
        assert!(world.is_material_at(entrance + IVec3::Y));
    }

    #[test]
    fn acceptance_removes_matching_material_immediately() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, MaterialKind::Basic);
        place_material(&mut world, IVec3::ZERO, MaterialKind::Basic);
        let mut state = acceptor_state(&world);

        let sparks = run_material_acceptance_phase(&mut world, &mut state);

        assert!(!world.is_material_at(IVec3::ZERO));
        assert!(sparks.contains(&IVec3::ZERO));
        assert_eq!(state.acceptor_structures()[0].count, 1);
    }

    #[test]
    fn acceptance_ignores_wrong_material() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, MaterialKind::Basic);
        place_material(&mut world, IVec3::ZERO, MaterialKind::Iron);
        let mut state = acceptor_state(&world);

        let sparks = run_material_acceptance_phase(&mut world, &mut state);

        assert!(world.is_material_at(IVec3::ZERO));
        assert!(sparks.is_empty());
        assert_eq!(state.acceptor_structures()[0].count, 0);
    }

    #[test]
    fn acceptance_requires_entire_connected_acceptor_structure() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, MaterialKind::Basic);
        place_goal(&mut world, IVec3::X, MaterialKind::Basic);
        place_material(&mut world, IVec3::ZERO, MaterialKind::Basic);
        let mut state = acceptor_state(&world);

        let sparks = run_material_acceptance_phase(&mut world, &mut state);

        assert!(world.is_material_at(IVec3::ZERO));
        assert!(sparks.is_empty());
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

        let sparks = run_material_acceptance_phase(&mut world, &mut state);

        assert!(world.is_material_at(IVec3::ZERO));
        assert!(world.is_material_at(IVec3::X));
        assert!(sparks.is_empty());
        assert_eq!(state.acceptor_structures()[0].count, 0);
    }

    #[test]
    fn acceptance_removes_entire_welded_structure_immediately() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, MaterialKind::Basic);
        place_goal(&mut world, IVec3::X, MaterialKind::Basic);
        place_material(&mut world, IVec3::ZERO, MaterialKind::Basic);
        place_material(&mut world, IVec3::X, MaterialKind::Basic);
        world.weld_materials(IVec3::ZERO, IVec3::X);
        let mut state = acceptor_state(&world);

        let sparks = run_material_acceptance_phase(&mut world, &mut state);

        assert!(!world.is_material_at(IVec3::ZERO));
        assert!(!world.is_material_at(IVec3::X));
        assert!(sparks.contains(&IVec3::ZERO));
        assert!(sparks.contains(&IVec3::X));
        assert_eq!(state.acceptor_structures()[0].count, 1);
    }
}
