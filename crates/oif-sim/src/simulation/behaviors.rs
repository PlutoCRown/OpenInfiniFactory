use glam::IVec3;
use std::collections::HashSet;

use crate::blocks::{
    AcceptorId, BlockData, BlockKind, MaterialDestroyer, MaterialLabeler, MaterialProcessor,
    SignalBehavior,
};
use crate::world::direction::Facing;
use crate::world::grid::{ConverterMode, GeneratorMode, MaterialFace, WorldBlocks};

use super::mirror;
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

/// 阶段 1 光学探测：只点亮传感器、记录光束，不销毁材料
pub(super) fn probe_lasers(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
) -> (Vec<LaserBeam>, HashSet<IVec3>, Vec<IVec3>) {
    run_lasers(world, powered_devices, false)
}

/// 阶段 4 激光销毁：按通电路径再 trace 并立刻移除材料
pub(super) fn destroy_powered_lasers(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
) -> Vec<IVec3> {
    run_lasers(world, powered_devices, true).2
}

/// 阶段 4 钻头销毁：立刻移除材料
pub(super) fn run_drill_destroy_phase(world: &mut WorldBlocks) -> Vec<IVec3> {
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
                destroy_material_immediate(world, pos + target, &mut sparks);
            }
            MaterialDestroyer::AdjacentDrillHead => {
                for offset in signal_offsets() {
                    destroy_material_immediate(world, pos + offset, &mut sparks);
                }
            }
            MaterialDestroyer::Laser { .. } => {}
        }
    }
    sparks
}

/// 阶段 4 传送：入口材料立刻迁到出口
pub(super) fn run_material_teleport_phase(world: &mut WorldBlocks) {
    let entrances: Vec<IVec3> = world
        .system_blocks
        .iter()
        .filter_map(|(pos, block)| {
            matches!(
                block.kind.material_processor(),
                Some(MaterialProcessor::TeleportEntrance)
            )
            .then_some(*pos)
        })
        .collect();

    let mut handled = HashSet::new();
    for entrance in entrances {
        if handled.contains(&entrance) {
            continue;
        }
        let Some(exit) = resolve_teleport_pair(world, entrance) else {
            continue;
        };
        if !teleport_entrance_material(world, entrance, exit) {
            continue;
        }
        handled.insert(entrance);
        handled.insert(exit);
    }
}

/// 解析传送入口对应的有效出口（有材料锚定且配对出口存在）
fn resolve_teleport_pair(world: &WorldBlocks, entrance: IVec3) -> Option<IVec3> {
    if !world.anchors_material_at_teleport_entrance(entrance) {
        return None;
    }
    let exit = world.teleport_partner(entrance)?;
    world
        .system_blocks
        .get(&exit)
        .filter(|block| block.kind == BlockKind::TeleportExit)
        .map(|_| exit)
}

/// 阶段 4 焊接：相邻焊点上的材料焊成一体
pub(super) fn run_weld_behavior_phase(world: &mut WorldBlocks) -> Vec<IVec3> {
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

/// 阶段 4 装饰漆：滚刷机朝向可 Connectable 材料面写入油漆（印花机 L4 再处理）
pub(super) fn run_material_paint_phase(world: &mut WorldBlocks) {
    let rollers: Vec<(IVec3, IVec3)> = world
        .system_blocks
        .iter()
        .filter_map(
            |(pos, block)| match block.kind.material_labeler(block.facing) {
                Some(MaterialLabeler::Roller { target }) => Some((*pos, target)),
                Some(MaterialLabeler::Stamper { .. }) | None => None,
            },
        )
        .collect();

    for (pos, target_offset) in rollers {
        let target = pos + target_offset;
        let Some(target_block) = world.blocks.get(&target).copied() else {
            continue;
        };
        if !target_block.kind.is_material() {
            continue;
        }
        let face_normal = -target_offset;
        let face = MaterialFace::new(target_block.id, face_normal);
        let connectable = {
            #[cfg(test)]
            if world.test_unconnectable_faces.contains(&face) {
                false
            } else {
                target_block
                    .kind
                    .material_face_connectable(target_block.facing, face_normal)
            }
            #[cfg(not(test))]
            target_block
                .kind
                .material_face_connectable(target_block.facing, face_normal)
        };
        if !connectable {
            continue;
        }
        let paint = world.roller_settings(pos).paint;
        world.material_paints.insert(face, paint);
    }
}

/// 阶段 4 印花：面前宿主可 Connectable 时在机身格生成/替换印花附着
pub(super) fn run_material_stamp_phase(world: &mut WorldBlocks) {
    let stampers: Vec<(IVec3, Facing)> = world
        .system_blocks
        .iter()
        .filter_map(
            |(pos, block)| match block.kind.material_labeler(block.facing) {
                Some(MaterialLabeler::Stamper { .. }) => Some((*pos, block.facing)),
                Some(MaterialLabeler::Roller { .. }) | None => None,
            },
        )
        .collect();

    for (stamper_pos, facing) in stampers {
        let forward = facing.forward_ivec3();
        let host_pos = stamper_pos + forward;
        let Some(host) = world.blocks.get(&host_pos).copied() else {
            continue;
        };
        if !host.kind.is_material()
            || host
                .kind
                .material_props()
                .is_some_and(|props| props.is_stamp)
        {
            continue;
        }
        let face_normal = -forward;
        let connectable = {
            #[cfg(test)]
            if world
                .test_unconnectable_faces
                .contains(&MaterialFace::new(host.id, face_normal))
            {
                false
            } else {
                host.kind
                    .material_face_connectable(host.facing, face_normal)
            }
            #[cfg(not(test))]
            host.kind
                .material_face_connectable(host.facing, face_normal)
        };
        if !connectable {
            continue;
        }

        // 该面已有附着印花：非脆弱则跳过；脆弱则碎旧换新
        let existing_child = world
            .material_attachments
            .iter()
            .find(|(_, att)| att.parent == host.id && att.parent_face_normal == face_normal)
            .map(|(child, _)| *child);
        if let Some(child_id) = existing_child {
            let Some((child_pos, child_block)) = world
                .blocks
                .iter()
                .find(|(_, b)| b.id == child_id)
                .map(|(p, b)| (*p, *b))
            else {
                world.material_attachments.remove(&child_id);
                continue;
            };
            let fragile = child_block
                .kind
                .material_props()
                .is_some_and(|props| props.fragile);
            if !fragile {
                continue;
            }
            world.remove(&child_pos);
        }

        // 印花占宿主面向机身的邻格 = 印花机格；该格 blocks 须空（机身在 machine_bodies）
        if world.blocks.contains_key(&stamper_pos) {
            continue;
        }

        let stamp_facing = match (face_normal.x, face_normal.y, face_normal.z) {
            (1, 0, 0) => Facing::East,
            (-1, 0, 0) => Facing::West,
            (0, 0, 1) => Facing::South,
            (0, 0, -1) => Facing::North,
            _ => facing,
        };
        let stamp_id = world.stamper_settings(stamper_pos).stamp;
        world.insert(
            stamper_pos,
            BlockData::new(BlockKind::Stamp(stamp_id), stamp_facing),
        );
        let Some(stamp) = world.blocks.get(&stamper_pos).copied() else {
            continue;
        };
        world.material_attachments.insert(
            stamp.id,
            crate::world::grid::MaterialAttachment {
                parent: host.id,
                parent_face_normal: face_normal,
            },
        );
    }
}

/// 本回合生成判定用的材料源结果
#[derive(Clone, Copy)]
pub(super) struct GeneratedMaterial {
    pub pos: IVec3,
    pub block: BlockData,
}

/// 按生成器设定收集本回合应调度的材料
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
        .filter_map(|(pos, block)| block.kind.material_source(block.facing).map(|_| *pos))
        .collect();

    for pos in sources {
        let settings = world.generator_settings(pos);
        let should_spawn = match settings.mode {
            GeneratorMode::Period { period, offset } => {
                let period = period.max(1);
                turn % period == offset % period
            }
            GeneratorMode::Link { anchor } => anchor
                .and_then(|pos| world.acceptor_id_at(pos))
                .is_some_and(|id| accepted_acceptors.contains(&id)),
        };
        if !should_spawn {
            continue;
        }

        let spawn_pos = pos;
        if world.can_place_platform_at(spawn_pos) && !blocked_generation.contains(&spawn_pos) {
            generated.push(GeneratedMaterial {
                pos: spawn_pos,
                block: BlockData::new(
                    BlockKind::Material(settings.material),
                    Facing::North,
                ),
            });
        }
    }
    generated
}

/// 阶段 4 转换：转换器格上的材料立刻变成输出种类
pub(super) fn run_material_conversion_phase(world: &mut WorldBlocks) {
    let converters: Vec<IVec3> = world
        .system_blocks
        .iter()
        .filter_map(|(pos, block)| {
            matches!(
                block.kind.material_processor(),
                Some(MaterialProcessor::Converter)
            )
            .then_some(*pos)
        })
        .collect();

    for pos in converters {
        let Some(mut block) = world.blocks.get(&pos).copied() else {
            continue;
        };
        let Some(input_material) = block.kind.material_id() else {
            continue;
        };

        let settings = world.converter_settings(pos);
        if settings.mode == ConverterMode::SpecificInput && input_material != settings.input {
            continue;
        }

        block.kind = BlockKind::Material(settings.output);
        world.insert(pos, block);
    }
}

/// 阶段 4 验收：匹配的材料立刻移除，返回验收器 id 与火花位置
pub(super) fn run_material_acceptance_phase(
    world: &mut WorldBlocks,
    structure_state: &mut StructureState,
) -> (HashSet<AcceptorId>, Vec<IVec3>) {
    let mut accepted = HashSet::new();
    let mut sparks = Vec::new();
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
            let Some(material) = block.kind.material_id() else {
                break;
            };
            if !world.accepts_material_id_at(*pos, material) {
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
                world.remove(pos);
                sparks.push(*pos);
            }
        }
        structure_state.increment_acceptor_count(index);
        if !acceptor_id.is_none() {
            accepted.insert(acceptor_id);
        }
    }
    (accepted, sparks)
}

fn run_lasers(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    destroy: bool,
) -> (Vec<LaserBeam>, HashSet<IVec3>, Vec<IVec3>) {
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
            destroy,
            &mut laser_beams,
            &mut sparks,
            &mut hit_detectors,
            0,
        );
    }
    (laser_beams, hit_detectors, sparks)
}

fn destroy_material_immediate(world: &mut WorldBlocks, pos: IVec3, sparks: &mut Vec<IVec3>) {
    if world.is_material_at(pos) {
        world.remove(&pos);
        sparks.push(pos);
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

    // 面附着按 BlockId，搬迁无需改写；用 relocate 避免 remove 清掉附着
    let Some(block) = world.blocks.get(&entrance).copied() else {
        return false;
    };
    world.relocate_blocks(vec![(entrance, exit, block)]);
    true
}

fn trace_laser(
    world: &mut WorldBlocks,
    origin: IVec3,
    direction: IVec3,
    range: i32,
    destroy: bool,
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
            if destroy {
                world.remove(&target);
                sparks.push(target);
            }
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
        let reflections = block
            .kind
            .laser_optics()
            .map(|optics| mirror::reflect_laser(optics, block.facing, direction))
            .unwrap_or_default();
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
                destroy,
                beams,
                sparks,
                hit_detectors,
                bounce_depth + 1,
            );
        }
        if block.kind.blocks_laser() {
            stop = if block.kind.laser_optics().is_some() {
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
        true,
        beams,
        &mut sparks,
        &mut hit_detectors,
        bounce_depth,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{BlockData, BlockKind, MaterialBlockId};
    use crate::simulation::structures::material_structure;
    use crate::world::direction::Facing;
    use crate::world::grid::{GoalSettings, WorldBlocks};

    fn material_id(string_id: &str) -> MaterialBlockId {
        BlockKind::material(string_id).material_id().unwrap()
    }

    fn place_goal(world: &mut WorldBlocks, pos: IVec3, material: &str) {
        world.insert(pos, BlockData::new(BlockKind::Goal, Facing::North));
        world.set_goal_settings(
            pos,
            GoalSettings {
                material: material_id(material),
            },
        );
    }

    fn place_material(world: &mut WorldBlocks, pos: IVec3, material: &str) {
        world.insert(
            pos,
            BlockData::new(BlockKind::material(material), Facing::North),
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
            BlockData::new(BlockKind::TeleportEntrance, Facing::North),
        );
        world.insert(exit, BlockData::new(BlockKind::TeleportExit, Facing::North));
        world.set_teleport_pair(entrance, Some(exit));
    }

    #[test]
    fn teleport_moves_immediately() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);
        place_material(&mut world, entrance, "basic");

        run_material_teleport_phase(&mut world);

        assert!(!world.is_material_at(entrance));
        assert!(world.is_material_at(exit));
    }

    #[test]
    fn teleport_moves_only_entrance_block_from_welded_structure() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);
        place_material(&mut world, entrance, "basic");
        place_material(&mut world, entrance + IVec3::X, "basic");
        world.weld_materials(entrance, entrance + IVec3::X);

        run_material_teleport_phase(&mut world);

        assert!(!world.is_material_at(entrance));
        assert!(world.is_material_at(exit));
        assert!(world.is_material_at(entrance + IVec3::X));
        let exit_id = world.blocks[&exit].id;
        let neighbor_id = world.blocks[&(entrance + IVec3::X)].id;
        assert!(
            !world
                .material_welds
                .contains(&crate::world::grid::MaterialWeld::new(exit_id, neighbor_id))
        );
    }

    #[test]
    fn teleport_waits_when_exit_is_occupied() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);
        place_material(&mut world, entrance, "basic");
        place_material(&mut world, exit, "iron");

        run_material_teleport_phase(&mut world);

        assert!(world.is_material_at(entrance));
        assert!(world.is_material_at(exit));
        assert_eq!(
            world
                .blocks
                .get(&exit)
                .and_then(|block| block.kind.material_id()),
            Some(material_id("iron"))
        );
    }

    #[test]
    fn teleport_can_run_three_times_when_exit_clears_between() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);

        for expected in ["basic", "iron", "copper"] {
            place_material(&mut world, entrance, expected);
            run_material_teleport_phase(&mut world);
            assert!(!world.is_material_at(entrance));
            assert_eq!(
                world
                    .blocks
                    .get(&exit)
                    .and_then(|block| block.kind.material_id()),
                Some(material_id(expected))
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
        place_material(&mut world, entrance, "basic");
        place_material(&mut world, exit, "iron");

        run_material_teleport_phase(&mut world);
        assert!(world.is_material_at(entrance));

        world.remove(&exit);
        run_material_teleport_phase(&mut world);

        assert!(!world.is_material_at(entrance));
        assert_eq!(
            world
                .blocks
                .get(&exit)
                .and_then(|block| block.kind.material_id()),
            Some(material_id("basic"))
        );
    }

    #[test]
    fn anchored_entrance_material_is_not_pushed_with_welded_neighbor() {
        use crate::simulation::structures::can_translate_structure;

        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let neighbor = IVec3::new(0, 0, 0);
        place_teleport_pair(&mut world, entrance, IVec3::new(5, 0, 0));
        place_material(&mut world, entrance, "basic");
        place_material(&mut world, neighbor, "basic");
        world.weld_materials(entrance, neighbor);
        let state = acceptor_state(&world);
        let structure = material_structure(&world, neighbor);

        assert!(!can_translate_structure(
            &world,
            &structure,
            IVec3::X,
            &state,
            &crate::simulation::SuctionLinks::default(),
        ));
    }

    #[test]
    fn teleport_detaches_before_moving_to_exit() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);
        place_material(&mut world, entrance, "basic");
        place_material(&mut world, entrance + IVec3::X, "basic");
        world.weld_materials(entrance, entrance + IVec3::X);

        run_material_teleport_phase(&mut world);

        assert!(!world.is_material_at(entrance));
        assert!(world.is_material_at(exit));
        assert!(world.is_material_at(entrance + IVec3::X));
        let exit_id = world.blocks[&exit].id;
        let neighbor_id = world.blocks[&(entrance + IVec3::X)].id;
        assert!(
            !world
                .material_welds
                .contains(&crate::world::grid::MaterialWeld::new(exit_id, neighbor_id))
        );
    }

    #[test]
    fn teleport_does_not_move_unwelded_neighbor_on_entrance() {
        let mut world = WorldBlocks::default();
        let entrance = IVec3::new(1, 0, 0);
        let exit = IVec3::new(5, 0, 0);
        place_teleport_pair(&mut world, entrance, exit);
        place_material(&mut world, entrance, "basic");
        place_material(&mut world, entrance + IVec3::Y, "basic");

        run_material_teleport_phase(&mut world);

        assert!(!world.is_material_at(entrance));
        assert!(world.is_material_at(exit));
        assert!(world.is_material_at(entrance + IVec3::Y));
    }

    #[test]
    fn acceptance_removes_matching_material_immediately() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, "basic");
        place_material(&mut world, IVec3::ZERO, "basic");
        let mut state = acceptor_state(&world);

        let (_accepted, sparks) = run_material_acceptance_phase(&mut world, &mut state);

        assert!(!world.is_material_at(IVec3::ZERO));
        assert_eq!(sparks, vec![IVec3::ZERO]);
        assert_eq!(state.acceptor_structures()[0].count, 1);
    }

    #[test]
    fn acceptance_ignores_wrong_material() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, "basic");
        place_material(&mut world, IVec3::ZERO, "iron");
        let mut state = acceptor_state(&world);

        let (_accepted, sparks) = run_material_acceptance_phase(&mut world, &mut state);

        assert!(world.is_material_at(IVec3::ZERO));
        assert!(sparks.is_empty());
        assert_eq!(state.acceptor_structures()[0].count, 0);
    }

    #[test]
    fn acceptance_requires_entire_connected_acceptor_structure() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, "basic");
        place_goal(&mut world, IVec3::X, "basic");
        place_material(&mut world, IVec3::ZERO, "basic");
        let mut state = acceptor_state(&world);

        let (_accepted, sparks) = run_material_acceptance_phase(&mut world, &mut state);

        assert!(world.is_material_at(IVec3::ZERO));
        assert!(sparks.is_empty());
        assert_eq!(state.acceptor_structures()[0].count, 0);
    }

    #[test]
    fn acceptance_requires_material_structure_without_extra_blocks() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, "basic");
        place_material(&mut world, IVec3::ZERO, "basic");
        place_material(&mut world, IVec3::X, "basic");
        world.weld_materials(IVec3::ZERO, IVec3::X);
        let mut state = acceptor_state(&world);

        let (_accepted, sparks) = run_material_acceptance_phase(&mut world, &mut state);

        assert!(world.is_material_at(IVec3::ZERO));
        assert!(world.is_material_at(IVec3::X));
        assert!(sparks.is_empty());
        assert_eq!(state.acceptor_structures()[0].count, 0);
    }

    #[test]
    fn acceptance_removes_entire_welded_structure_immediately() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, "basic");
        place_goal(&mut world, IVec3::X, "basic");
        place_material(&mut world, IVec3::ZERO, "basic");
        place_material(&mut world, IVec3::X, "basic");
        world.weld_materials(IVec3::ZERO, IVec3::X);
        let mut state = acceptor_state(&world);

        let (_accepted, sparks) = run_material_acceptance_phase(&mut world, &mut state);

        assert!(!world.is_material_at(IVec3::ZERO));
        assert!(!world.is_material_at(IVec3::X));
        assert_eq!(sparks.len(), 2);
        assert_eq!(state.acceptor_structures()[0].count, 1);
    }

    #[test]
    fn period_offset_triggers_on_matching_turns() {
        let mut world = WorldBlocks::default();
        let pos = IVec3::new(1, 1, 0);
        world.insert(pos, BlockData::new(BlockKind::Generator, Facing::North));
        world.set_generator_settings(
            pos,
            crate::world::grid::GeneratorSettings {
                mode: GeneratorMode::Period {
                    period: 3,
                    offset: 1,
                },
                material: material_id("basic"),
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
        let gen_pos = IVec3::new(2, 1, 0);
        world.insert(gen_pos, BlockData::new(BlockKind::Generator, Facing::North));
        world.set_generator_settings(
            gen_pos,
            crate::world::grid::GeneratorSettings {
                mode: GeneratorMode::Link {
                    anchor: Some(IVec3::ZERO),
                },
                material: material_id("iron"),
            },
        );
        let blocked = HashSet::new();
        let none_accepted = HashSet::new();
        assert!(material_source_generation(&world, 5, &blocked, &none_accepted).is_empty());
        let accepted = HashSet::from([acceptor]);
        let generated = material_source_generation(&world, 5, &blocked, &accepted);
        assert_eq!(generated.len(), 1);
        assert_eq!(generated[0].pos, gen_pos);
    }

    #[test]
    fn acceptance_returns_acceptor_id() {
        let mut world = WorldBlocks::default();
        place_goal(&mut world, IVec3::ZERO, "basic");
        place_material(&mut world, IVec3::ZERO, "basic");
        let expected = world.acceptor_id_at(IVec3::ZERO).unwrap();
        let mut state = acceptor_state(&world);

        let (accepted, _sparks) = run_material_acceptance_phase(&mut world, &mut state);
        assert!(accepted.contains(&expected));
    }

    #[test]
    fn probe_lasers_do_not_remove_materials() {
        let mut world = WorldBlocks::default();
        let laser = IVec3::new(0, 0, 0);
        let material = IVec3::new(1, 0, 0);
        world.insert(laser, BlockData::new(BlockKind::Laser, Facing::East));
        place_material(&mut world, material, "basic");
        let powered = HashSet::from([laser]);

        let (beams, _detectors, sparks) = probe_lasers(&mut world, &powered);

        assert!(world.is_material_at(material));
        assert!(sparks.is_empty());
        assert!(!beams.is_empty());
    }

    #[test]
    fn destroy_powered_lasers_remove_materials() {
        let mut world = WorldBlocks::default();
        let laser = IVec3::new(0, 0, 0);
        let material = IVec3::new(1, 0, 0);
        world.insert(laser, BlockData::new(BlockKind::Laser, Facing::East));
        place_material(&mut world, material, "basic");
        let powered = HashSet::from([laser]);

        let sparks = destroy_powered_lasers(&mut world, &powered);

        assert!(!world.is_material_at(material));
        assert_eq!(sparks, vec![material]);
    }

    #[test]
    fn roller_paints_connectable_material_face() {
        let mut world = WorldBlocks::default();
        let roller = IVec3::ZERO;
        let material = IVec3::X;
        world.insert(roller, BlockData::new(BlockKind::Roller, Facing::East));
        place_material(&mut world, material, "basic");
        let material_id = world.blocks[&material].id;

        run_material_paint_phase(&mut world);

        let face = MaterialFace::new(material_id, IVec3::NEG_X);
        assert_eq!(
            world.material_paints.get(&face).copied(),
            Some(world.roller_settings(roller).paint)
        );
    }

    #[test]
    fn roller_paint_survives_full_simulate_turn() {
        use crate::simulation::core::simulate_turn;
        use crate::simulation::movement::PusherState;
        use crate::simulation::pending::PendingGeneratedMaterials;
        use crate::simulation::signals::SignalNetworkCache;
        use crate::simulation::structure_state::StructureState;
        use crate::simulation::structures::MovementInfluenceCache;

        let mut world = WorldBlocks::default();
        let roller = IVec3::new(0, 1, 0);
        let material = IVec3::new(1, 1, 0);
        // 支撑，避免重力先把材料挪开再刷漆
        world.insert(
            IVec3::new(1, 0, 0),
            BlockData::new(BlockKind::scene("stone"), Facing::North),
        );
        world.insert(roller, BlockData::new(BlockKind::Roller, Facing::East));
        place_material(&mut world, material, "basic");
        let material_id = world.blocks[&material].id;
        crate::simulation::markers::refresh_static_generated_markers(&mut world);

        let mut pending = PendingGeneratedMaterials::default();
        let mut signal_cache = SignalNetworkCache::default();
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut influence = MovementInfluenceCache::default();
        let mut pusher_state = PusherState::rebuild_from_world(&world);

        simulate_turn(
            &mut world,
            &mut pending,
            &mut signal_cache,
            1,
            &mut structures,
            &mut influence,
            &mut pusher_state,
            None,
            None,
        );

        let face = MaterialFace::new(material_id, IVec3::NEG_X);
        assert_eq!(
            world.material_paints.get(&face).copied(),
            Some(world.roller_settings(roller).paint),
            "full turn must keep roller paint on the contact face"
        );
        assert!(
            world.is_material_at(material),
            "supported material must stay in front of roller"
        );
    }

    #[test]
    fn roller_skips_non_connectable_face() {
        let mut world = WorldBlocks::default();
        let roller = IVec3::ZERO;
        let material = IVec3::X;
        world.insert(roller, BlockData::new(BlockKind::Roller, Facing::East));
        place_material(&mut world, material, "basic");
        let material_id = world.blocks[&material].id;
        let face = MaterialFace::new(material_id, IVec3::NEG_X);
        world.test_unconnectable_faces.insert(face);

        run_material_paint_phase(&mut world);

        assert!(world.material_paints.is_empty());
    }

    #[test]
    fn stamper_does_not_paint() {
        let mut world = WorldBlocks::default();
        let stamper = IVec3::ZERO;
        let material = IVec3::X;
        world.insert(stamper, BlockData::new(BlockKind::Stamper, Facing::East));
        place_material(&mut world, material, "basic");

        run_material_paint_phase(&mut world);

        assert!(world.material_paints.is_empty());
    }

    #[test]
    fn stamper_spawns_stamp_attachment_on_connectable_face() {
        let mut world = WorldBlocks::default();
        let stamper = IVec3::new(1, 1, 0);
        let host = IVec3::new(2, 1, 0);
        world.insert(stamper, BlockData::new(BlockKind::Stamper, Facing::East));
        place_material(&mut world, host, "basic");
        let host_id = world.blocks[&host].id;
        crate::simulation::markers::refresh_static_generated_markers(&mut world);

        run_material_stamp_phase(&mut world);

        let stamp = world.blocks.get(&stamper).expect("stamp in stamper cell");
        assert_eq!(stamp.kind, BlockKind::stamp("red"));
        assert_eq!(stamp.kind.stamp_id(), Some(world.stamper_settings(stamper).stamp));
        let att = world.material_attachments.get(&stamp.id).unwrap();
        assert_eq!(att.parent, host_id);
        assert_eq!(att.parent_face_normal, IVec3::NEG_X);
        assert!(world.machine_bodies.contains_key(&stamper));
    }

    #[test]
    fn stamper_skips_non_connectable_host_face() {
        let mut world = WorldBlocks::default();
        let stamper = IVec3::new(1, 1, 0);
        let host = IVec3::new(2, 1, 0);
        world.insert(stamper, BlockData::new(BlockKind::Stamper, Facing::East));
        place_material(&mut world, host, "basic");
        let host_id = world.blocks[&host].id;
        world
            .test_unconnectable_faces
            .insert(MaterialFace::new(host_id, IVec3::NEG_X));

        run_material_stamp_phase(&mut world);

        assert!(!world.blocks.contains_key(&stamper));
        assert!(world.material_attachments.is_empty());
    }

    #[test]
    fn stamp_attachment_joins_host_material_structure() {
        let mut world = WorldBlocks::default();
        let host = IVec3::new(0, 1, 0);
        let stamp_pos = IVec3::new(1, 1, 0);
        place_material(&mut world, host, "basic");
        world.insert(
            stamp_pos,
            BlockData::new(BlockKind::stamp("red"), Facing::West),
        );
        let host_id = world.blocks[&host].id;
        let stamp_id = world.blocks[&stamp_pos].id;
        world.material_attachments.insert(
            stamp_id,
            crate::world::grid::MaterialAttachment {
                parent: host_id,
                parent_face_normal: IVec3::X,
            },
        );

        let structure = material_structure(&world, host);
        assert!(structure.contains(&host));
        assert!(structure.contains(&stamp_pos));
    }

    #[test]
    fn aligned_stamp_can_enter_stamper_body_cell() {
        let mut world = WorldBlocks::default();
        let stamper = IVec3::new(1, 0, 0);
        let stamp_pos = IVec3::new(1, 1, 0);
        let host = IVec3::new(0, 1, 0);
        world.insert(stamper, BlockData::new(BlockKind::Stamper, Facing::West));
        place_material(&mut world, host, "basic");
        world.insert(
            stamp_pos,
            BlockData::new(BlockKind::stamp("red"), Facing::West),
        );
        let host_id = world.blocks[&host].id;
        let stamp_id = world.blocks[&stamp_pos].id;
        world.material_attachments.insert(
            stamp_id,
            crate::world::grid::MaterialAttachment {
                parent: host_id,
                parent_face_normal: IVec3::X,
            },
        );
        crate::simulation::markers::refresh_static_generated_markers(&mut world);

        assert!(world.stamper_body_allows_stamp(stamper, &world.blocks[&stamp_pos]));
        assert!(world.cell_accepts_move_from(stamp_pos, stamper));
    }

    #[test]
    fn misaligned_stamper_blocks_non_fragile_stamp() {
        let mut world = WorldBlocks::default();
        let stamper = IVec3::new(1, 0, 0);
        let stamp_pos = IVec3::new(1, 1, 0);
        let host = IVec3::new(0, 1, 0);
        // 朝北：与印花附着（东向）不对齐
        world.insert(stamper, BlockData::new(BlockKind::Stamper, Facing::North));
        place_material(&mut world, host, "basic");
        world.insert(
            stamp_pos,
            BlockData::new(BlockKind::stamp("red"), Facing::West),
        );
        let host_id = world.blocks[&host].id;
        let stamp_id = world.blocks[&stamp_pos].id;
        world.material_attachments.insert(
            stamp_id,
            crate::world::grid::MaterialAttachment {
                parent: host_id,
                parent_face_normal: IVec3::X,
            },
        );
        crate::simulation::markers::refresh_static_generated_markers(&mut world);

        assert!(!world.stamper_body_allows_stamp(stamper, &world.blocks[&stamp_pos]));
        assert!(!world.cell_accepts_move_from(stamp_pos, stamper));
        assert!(!world.can_move_into(stamper));
    }

    #[test]
    fn stamper_skips_when_non_fragile_stamp_already_on_face() {
        let mut world = WorldBlocks::default();
        let stamper = IVec3::new(1, 1, 0);
        let host = IVec3::new(2, 1, 0);
        world.insert(stamper, BlockData::new(BlockKind::Stamper, Facing::East));
        place_material(&mut world, host, "basic");
        let host_id = world.blocks[&host].id;
        world.insert(
            stamper,
            BlockData::new(BlockKind::stamp("red"), Facing::West),
        );
        let stamp_id = world.blocks[&stamper].id;
        world.material_attachments.insert(
            stamp_id,
            crate::world::grid::MaterialAttachment {
                parent: host_id,
                parent_face_normal: IVec3::NEG_X,
            },
        );

        run_material_stamp_phase(&mut world);

        assert_eq!(world.blocks[&stamper].id, stamp_id);
        assert_eq!(world.material_attachments.len(), 1);
    }

    #[test]
    fn sign_cannot_place_on_conveyor_top() {
        let mut world = WorldBlocks::default();
        let host = IVec3::new(0, 1, 0);
        world.insert(host, BlockData::new(BlockKind::Conveyor, Facing::North));
        assert!(!world.can_place_sign_on_face(host, IVec3::Y));
        assert!(world.can_place_sign_on_face(host, Facing::North.forward_ivec3()));
    }

    #[test]
    fn sign_can_place_on_scene_stone_side() {
        let mut world = WorldBlocks::default();
        let host = IVec3::new(0, 1, 0);
        world.insert(host, BlockData::new(BlockKind::scene("stone"), Facing::North));
        assert!(world.can_place_sign_on_face(host, IVec3::X));
        assert!(world.can_place_sign_on_face(host, IVec3::Y));
    }

    #[test]
    fn sign_can_place_on_platform_side() {
        let mut world = WorldBlocks::default();
        let host = IVec3::new(0, 1, 0);
        world.insert(host, BlockData::new(BlockKind::Platform, Facing::North));
        assert!(world.can_place_sign_on_face(host, IVec3::X));
        assert!(world.can_place_sign_on_face(host, IVec3::Y));
    }

    #[test]
    fn sign_attachment_destroyed_with_host() {
        let mut world = WorldBlocks::default();
        let host = IVec3::new(0, 1, 0);
        let sign_pos = IVec3::new(1, 1, 0);
        world.insert(host, BlockData::new(BlockKind::Platform, Facing::North));
        world.insert(sign_pos, BlockData::new(BlockKind::Sign, Facing::East));
        let host_id = world.blocks[&host].id;
        let sign_id = world.blocks[&sign_pos].id;
        world.attach_factory_child(sign_id, host_id, IVec3::X);
        world.remove(&host);
        assert!(!world.blocks.contains_key(&sign_pos));
        assert!(world.factory_attachments.is_empty());
    }

    #[test]
    fn sign_moves_with_material_host() {
        use crate::simulation::structure_state::StructureState;
        use crate::simulation::structures::{can_translate_structure, move_structure};
        use crate::simulation::suction::SuctionLinks;

        let mut world = WorldBlocks::default();
        let host = IVec3::new(0, 1, 0);
        let sign_pos = IVec3::new(1, 1, 0);
        place_material(&mut world, host, "basic");
        world.insert(sign_pos, BlockData::new(BlockKind::Sign, Facing::East));
        let host_id = world.blocks[&host].id;
        let sign_id = world.blocks[&sign_pos].id;
        world.attach_factory_child(sign_id, host_id, IVec3::X);

        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let seed = HashSet::from([host]);
        let suction = SuctionLinks::default();
        assert!(can_translate_structure(
            &world,
            &seed,
            IVec3::X,
            &structures,
            &suction
        ));
        // 展开应把告示并入；此处直接搬迁宿主+附着子格
        let moving = HashSet::from([host, sign_pos]);
        move_structure(&mut world, &moving, IVec3::X);
        assert!(!world.blocks.contains_key(&host));
        assert_eq!(world.blocks[&(host + IVec3::X)].id, host_id);
        assert_eq!(world.blocks[&(sign_pos + IVec3::X)].id, sign_id);
    }
}
