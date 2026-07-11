use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::blocks::{
    AcceptorId, BlockData, BlockKind, MaterialDestroyer, MaterialLabeler, SignalBehavior,
};
use crate::game::world::direction::Facing;
use crate::game::world::grid::{
    ConverterMode, GeneratorMode, MaterialFace, MaterialFaceMark, MaterialFaceMarkSource,
    WorldBlocks,
};

use super::mirror;
use super::runtime::PendingGeneratedMaterials;
use super::signal_offsets;
use super::structure_state::StructureState;
use super::structures::material_structure;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaserBeamStop {
    Open,
    Mirror,
    Solid,
}

#[derive(Clone, Copy)]
pub struct LaserBeam {
    pub pos: IVec3,
    pub direction: IVec3,
    pub range: i32,
    pub stop: LaserBeamStop,
    /// 镜子/分光镜反射光从方块中心发出；激光器从出射面发出
    pub emits_from_center: bool,
}

pub(super) struct MaterialBehaviorEffects {
    pub sparks: Vec<IVec3>,
    pub laser_beams: Vec<LaserBeam>,
    pub accepted_acceptors: HashSet<AcceptorId>,
}

pub(super) fn run_material_behavior_phase(
    world: &mut WorldBlocks,
    structure_state: &mut StructureState,
    pending_destroyed: &mut PendingGeneratedMaterials,
    ready_turn: u64,
) -> MaterialBehaviorEffects {
    let effects = run_material_destroy_phase(world, pending_destroyed, ready_turn);
    mark_material_teleport_phase(world, pending_destroyed, ready_turn);
    run_material_label_phase(world);
    run_material_conversion_phase(world);
    let accepted_acceptors =
        run_material_acceptance_phase(world, structure_state, pending_destroyed, ready_turn);
    MaterialBehaviorEffects {
        sparks: effects.sparks,
        laser_beams: effects.laser_beams,
        accepted_acceptors,
    }
}

// 通电激光先发射：摧毁材料，并记录打中工作面的传感器（供本回合二次供电）
pub(super) fn run_laser_phase(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
) -> (MaterialBehaviorEffects, HashSet<IVec3>) {
    let lasers: Vec<(IVec3, IVec3, i32)> = world
        .blocks
        .iter()
        .filter_map(
            |(pos, block)| match block.kind.material_destroyer(block.facing) {
                Some(MaterialDestroyer::Laser { direction, range }) => {
                    Some((*pos, direction, range))
                }
                _ => None,
            },
        )
        .collect();

    let mut sparks = Vec::new();
    let mut laser_beams = Vec::new();
    let mut hit_detectors = HashSet::new();
    for (pos, direction, range) in lasers {
        if !powered_devices.contains(&pos) {
            continue;
        }
        trace_laser(
            world,
            pos,
            direction,
            range,
            &mut laser_beams,
            &mut sparks,
            &mut hit_detectors,
            0,
        );
    }
    (
        MaterialBehaviorEffects {
            sparks,
            laser_beams,
            accepted_acceptors: HashSet::new(),
        },
        hit_detectors,
    )
}

pub(super) fn mark_material_teleport_phase(
    world: &WorldBlocks,
    pending: &mut PendingGeneratedMaterials,
    ready_turn: u64,
) {
    mark_material_teleport_phase_impl(world, pending, ready_turn);
}

pub(super) fn run_ready_material_teleports(
    world: &mut WorldBlocks,
    pending: &mut PendingGeneratedMaterials,
    turn: u64,
) {
    run_ready_material_teleports_impl(world, pending, turn);
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
    accepted_acceptors: &HashSet<AcceptorId>,
) -> Vec<GeneratedMaterial> {
    let mut generated = Vec::new();
    if turn == 0 {
        return generated;
    }

    let sources: Vec<IVec3> = world
        .system_blocks
        .iter()
        .filter_map(|(pos, block)| {
            block
                .kind
                .material_source(block.facing)
                .map(|_| *pos)
        })
        .collect();

    for pos in sources {
        let settings = world.generator_settings(pos);
        let should_spawn = match settings.mode {
            GeneratorMode::Period { period, offset } => {
                let period = period.max(1);
                turn % period == offset % period
            }
            GeneratorMode::Link { anchor } => {
                anchor
                    .and_then(|pos| world.acceptor_id_at(pos))
                    .is_some_and(|id| accepted_acceptors.contains(&id))
            }
        };
        if !should_spawn {
            continue;
        }

        let spawn_pos = pos;
        if world.can_place_platform_at(spawn_pos) && !blocked_generation.contains(&spawn_pos) {
            let Some(kind) = BlockKind::material_block_kind(settings.material) else {
                continue;
            };
            generated.push(GeneratedMaterial {
                pos: spawn_pos,
                block: BlockData::new(kind, Facing::North),
            });
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
            if world.weld_materials(weld_point, neighbor) {
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
        let Some(target_id) = world.blocks.get(&target).map(|block| block.id) else {
            continue;
        };

        let face = MaterialFace::new(target_id, -target_offset);
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
    if let Some(id) = world.blocks.get(&pos).map(|block| block.id) {
        world.material_welds.retain(|weld| !weld.contains(id));
    }
}

fn teleport_entrance_material(world: &mut WorldBlocks, entrance: IVec3, exit: IVec3) -> bool {
    if !world.anchors_material_at_teleport_entrance(entrance) {
        return false;
    }
    if world.is_material_at(exit) || !world.can_move_into(exit) {
        return false;
    }

    detach_material_block(world, entrance);

    // 面标记按 BlockId，搬迁无需改写；用 relocate 避免 remove 清掉标记
    let Some(block) = world.blocks.get(&entrance).copied() else {
        return false;
    };
    world.relocate_blocks(vec![(entrance, exit, block)]);
    true
}

fn mark_material_teleport_phase_impl(
    world: &WorldBlocks,
    pending: &mut PendingGeneratedMaterials,
    ready_turn: u64,
) {
    let entrances: Vec<IVec3> = world
        .system_blocks
        .iter()
        .filter_map(|(pos, block)| (block.kind == BlockKind::TeleportEntrance).then_some(*pos))
        .collect();

    for entrance in entrances {
        if !world.anchors_material_at_teleport_entrance(entrance) {
            pending.remove_pending_teleport(entrance);
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
        pending.mark_teleport(entrance, ready_turn);
    }
}

fn run_ready_material_teleports_impl(
    world: &mut WorldBlocks,
    pending: &mut PendingGeneratedMaterials,
    turn: u64,
) {
    let ready = pending.ready_teleport_entrances(turn);
    let mut handled = HashSet::new();

    for entrance in ready {
        if handled.contains(&entrance) {
            continue;
        }
        if !world.anchors_material_at_teleport_entrance(entrance) {
            pending.remove_pending_teleport(entrance);
            continue;
        }
        let Some(exit) = world.teleport_partner(entrance) else {
            pending.remove_pending_teleport(entrance);
            continue;
        };
        if !world
            .system_blocks
            .get(&exit)
            .is_some_and(|block| block.kind == BlockKind::TeleportExit)
        {
            pending.remove_pending_teleport(entrance);
            continue;
        }
        if !teleport_entrance_material(world, entrance, exit) {
            continue;
        }
        pending.remove_pending_teleport(entrance);
        handled.insert(entrance);
        handled.insert(exit);
    }
}

fn run_material_destroy_phase(
    world: &mut WorldBlocks,
    pending_destroyed: &mut PendingGeneratedMaterials,
    ready_turn: u64,
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
            // 激光已在 run_laser_phase 处理
            MaterialDestroyer::Laser { .. } => {}
        }
    }
    MaterialBehaviorEffects {
        sparks,
        laser_beams: Vec::new(),
        accepted_acceptors: HashSet::new(),
    }
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

fn trace_laser(
    world: &mut WorldBlocks,
    origin: IVec3,
    direction: IVec3,
    range: i32,
    beams: &mut Vec<LaserBeam>,
    sparks: &mut Vec<IVec3>,
    hit_detectors: &mut HashSet<IVec3>,
    bounce_depth: u32,
) {
    const MAX_BOUNCES: u32 = 8;
    if range <= 0 || bounce_depth > MAX_BOUNCES {
        return;
    }

    let mut traveled = 0;
    let mut stop = LaserBeamStop::Open;
    for distance in 1..=range {
        let target = origin + direction * distance;
        let Some(block) = world.blocks.get(&target).copied() else {
            traveled = distance;
            continue;
        };
        if block.kind.is_material() {
            world.remove(&target);
            sparks.push(target);
            traveled = distance;
            continue;
        }
        traveled = distance;
        // 激光打中传感器工作面：入射方向正对检测方向
        if let Some(SignalBehavior::Detector { detection_pos }) =
            block.kind.signal_behavior(block.facing)
        {
            if direction == -detection_pos {
                hit_detectors.insert(target);
            }
        }
        let reflections = mirror::reflect_laser(block.kind, block.facing, direction);
        if !reflections.is_empty() {
            sparks.push(target);
        }
        for reflected in reflections {
            // 与激光发射器相同：从镜面格出发，沿反射方向重新 trace 一整段
            trace_laser(
                world,
                target,
                reflected,
                range,
                beams,
                sparks,
                hit_detectors,
                bounce_depth + 1,
            );
        }
        if block.kind.blocks_laser() {
            stop = if matches!(
                block.kind,
                BlockKind::Mirror | BlockKind::VerticalMirror | BlockKind::Splitter
            ) {
                LaserBeamStop::Mirror
            } else {
                LaserBeamStop::Solid
            };
            break;
        }
    }
    if traveled > 0 {
        beams.push(LaserBeam {
            pos: origin,
            direction,
            range: traveled,
            stop,
            emits_from_center: bounce_depth > 0,
        });
    }
}

#[cfg(test)]
pub(crate) fn trace_laser_for_test(
    world: &mut WorldBlocks,
    origin: IVec3,
    direction: IVec3,
    range: i32,
    beams: &mut Vec<LaserBeam>,
    bounce_depth: u32,
) {
    let mut sparks = Vec::new();
    let mut hit_detectors = HashSet::new();
    trace_laser(
        world,
        origin,
        direction,
        range,
        beams,
        &mut sparks,
        &mut hit_detectors,
        bounce_depth,
    );
}

fn run_material_acceptance_phase(
    world: &mut WorldBlocks,
    structure_state: &mut StructureState,
    pending_destroyed: &mut PendingGeneratedMaterials,
    ready_turn: u64,
) -> HashSet<AcceptorId> {
    let mut accepted = HashSet::new();
    let acceptor_count = structure_state.acceptor_structures().len();
    for index in 0..acceptor_count {
        let Some(acceptor) = structure_state.acceptor_structures().get(index) else {
            continue;
        };
        let acceptor_id = acceptor.id;
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
            pending_destroyed.mark_acceptance_spark(*pos, ready_turn);
        }
        structure_state.increment_acceptor_count(index);
        if !acceptor_id.is_none() {
            accepted.insert(acceptor_id);
        }
    }
    accepted
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
            BlockData::new(BlockKind::Goal, Facing::North),
        );
        world.set_goal_settings(pos, GoalSettings { material });
    }

    fn place_material(world: &mut WorldBlocks, pos: IVec3, material: MaterialKind) {
        let kind = BlockKind::material_block_kind(material).unwrap();
        world.insert(pos, BlockData::new(kind, Facing::North));
    }

    fn acceptor_state(world: &WorldBlocks) -> StructureState {
        let mut state = StructureState::default();
        state.rebuild_for_simulation(world);
        state
    }

    fn place_teleport_pair(world: &mut WorldBlocks, entrance: IVec3, exit: IVec3) {
        world.insert(
            entrance,
            BlockData::new(BlockKind::TeleportEntrance, Facing::North),
        );
        world.insert(
            exit,
            BlockData::new(BlockKind::TeleportExit, Facing::North),
        );
        world.set_teleport_pair(entrance, Some(exit));
    }

    fn mark_and_run_teleport(
        world: &mut WorldBlocks,
        pending: &mut PendingGeneratedMaterials,
        ready_turn: u64,
    ) {
        mark_material_teleport_phase(world, pending, ready_turn);
        run_ready_material_teleports(world, pending, ready_turn);
    }

    #[test]
    fn teleport_waits_until_ready_turn() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);
        place_material(&mut world, entrance, MaterialKind::Basic);
        let mut pending = PendingGeneratedMaterials::default();

        mark_material_teleport_phase(&mut world, &mut pending, 2);
        run_ready_material_teleports(&mut world, &mut pending, 1);

        assert!(world.is_material_at(entrance));
        assert!(!world.is_material_at(exit));

        run_ready_material_teleports(&mut world, &mut pending, 2);

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
        let mut pending = PendingGeneratedMaterials::default();

        mark_and_run_teleport(&mut world, &mut pending, 2);

        assert!(!world.is_material_at(entrance));
        assert!(world.is_material_at(exit));
        assert!(world.is_material_at(entrance + IVec3::X));
        let exit_id = world.blocks[&exit].id;
        let neighbor_id = world.blocks[&(entrance + IVec3::X)].id;
        assert!(!world
            .material_welds
            .contains(&crate::game::world::grid::MaterialWeld::new(
                exit_id, neighbor_id
            )));
    }

    #[test]
    fn teleport_waits_when_exit_is_occupied() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);
        place_material(&mut world, entrance, MaterialKind::Basic);
        place_material(&mut world, exit, MaterialKind::Iron);
        let mut pending = PendingGeneratedMaterials::default();

        mark_and_run_teleport(&mut world, &mut pending, 2);

        assert!(world.is_material_at(entrance));
        assert!(world.is_material_at(exit));
        assert_eq!(
            world
                .blocks
                .get(&exit)
                .and_then(|block| block.kind.material_kind()),
            Some(MaterialKind::Iron)
        );
    }

    #[test]
    fn teleport_can_run_three_times_when_exit_clears_between() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);
        let mut pending = PendingGeneratedMaterials::default();

        for expected in [
            MaterialKind::Basic,
            MaterialKind::Iron,
            MaterialKind::Copper,
        ] {
            place_material(&mut world, entrance, expected);
            mark_and_run_teleport(&mut world, &mut pending, 2);
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
        let mut pending = PendingGeneratedMaterials::default();

        mark_and_run_teleport(&mut world, &mut pending, 2);
        assert!(world.is_material_at(entrance));

        world.remove(&exit);
        run_ready_material_teleports(&mut world, &mut pending, 2);

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
        let mut pending = PendingGeneratedMaterials::default();

        mark_and_run_teleport(&mut world, &mut pending, 2);

        assert!(!world.is_material_at(entrance));
        assert!(world.is_material_at(exit));
        assert!(world.is_material_at(entrance + IVec3::X));
        let exit_id = world.blocks[&exit].id;
        let neighbor_id = world.blocks[&(entrance + IVec3::X)].id;
        assert!(!world
            .material_welds
            .contains(&crate::game::world::grid::MaterialWeld::new(
                exit_id, neighbor_id
            )));
    }

    #[test]
    fn teleport_does_not_move_unwelded_neighbor_on_entrance() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);
        place_material(&mut world, entrance, MaterialKind::Basic);
        place_material(&mut world, entrance + IVec3::Y, MaterialKind::Basic);
        let mut pending = PendingGeneratedMaterials::default();

        mark_and_run_teleport(&mut world, &mut pending, 2);

        assert!(!world.is_material_at(entrance));
        assert!(world.is_material_at(exit));
        assert!(world.is_material_at(entrance + IVec3::Y));
    }

    #[test]
    fn acceptance_marks_matching_material_for_next_turn_removal() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, MaterialKind::Basic);
        place_material(&mut world, IVec3::ZERO, MaterialKind::Basic);
        let mut state = acceptor_state(&world);
        let mut pending = PendingGeneratedMaterials::default();

        run_material_acceptance_phase(&mut world, &mut state, &mut pending, 2);

        assert!(world.is_material_at(IVec3::ZERO));
        assert_eq!(pending.pending_destroy_turn(IVec3::ZERO), Some(2));
        assert_eq!(pending.pending_acceptance_spark_turn(IVec3::ZERO), Some(2));
        assert_eq!(state.acceptor_structures()[0].count, 1);
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

    #[test]
    fn period_offset_triggers_on_matching_turns() {
        let mut world = WorldBlocks::default();
        let pos = IVec3::new(1, 1, 0);
        world.insert(pos, BlockData::new(BlockKind::Generator, Facing::North));
        world.set_generator_settings(
            pos,
            crate::game::world::grid::GeneratorSettings {
                mode: GeneratorMode::Period {
                    period: 3,
                    offset: 1,
                },
                material: MaterialKind::Basic,
            },
        );
        let blocked = HashSet::new();
        let accepted = HashSet::new();
        assert_eq!(
            material_source_generation(&world, 1, &blocked, &accepted).len(),
            1
        );
        assert!(material_source_generation(&world, 2, &blocked, &accepted).is_empty());
        assert!(material_source_generation(&world, 3, &blocked, &accepted).is_empty());
        assert_eq!(
            material_source_generation(&world, 4, &blocked, &accepted).len(),
            1
        );
    }

    #[test]
    fn link_mode_triggers_only_for_accepted_acceptor() {
        let mut world = WorldBlocks::default();
        world.insert(IVec3::ZERO, BlockData::new(BlockKind::Goal, Facing::North));
        let acceptor = world.acceptor_id_at(IVec3::ZERO).unwrap();
        let gen = IVec3::new(2, 1, 0);
        world.insert(gen, BlockData::new(BlockKind::Generator, Facing::North));
        world.set_generator_settings(
            gen,
            crate::game::world::grid::GeneratorSettings {
                mode: GeneratorMode::Link {
                    anchor: Some(IVec3::ZERO),
                },
                material: MaterialKind::Iron,
            },
        );
        let blocked = HashSet::new();
        let none_accepted = HashSet::new();
        assert!(material_source_generation(&world, 5, &blocked, &none_accepted).is_empty());
        let accepted = HashSet::from([acceptor]);
        let generated = material_source_generation(&world, 5, &blocked, &accepted);
        assert_eq!(generated.len(), 1);
        assert_eq!(generated[0].pos, gen);
    }

    #[test]
    fn acceptance_returns_acceptor_id() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, MaterialKind::Basic);
        place_material(&mut world, IVec3::ZERO, MaterialKind::Basic);
        let expected = world.acceptor_id_at(IVec3::ZERO).unwrap();
        let mut state = acceptor_state(&world);
        let mut pending = PendingGeneratedMaterials::default();

        let accepted = run_material_acceptance_phase(&mut world, &mut state, &mut pending, 2);
        assert!(accepted.contains(&expected));
    }
}
