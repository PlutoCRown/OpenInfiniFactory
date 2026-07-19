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

/// 钻头/激光/验收毁掉的材料：位置 + 种类（表现层采样纹理）
#[derive(Clone, Copy, Debug)]
pub struct BreakDebris {
    pub pos: IVec3,
    pub kind: BlockKind,
}

/// 阶段 1 光学探测：只点亮传感器、记录光束，不销毁材料
pub(super) fn probe_lasers(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
) -> (Vec<LaserBeam>, HashSet<IVec3>, Vec<IVec3>) {
    let (beams, detectors, sparks, _) = run_lasers(world, powered_devices, false);
    (beams, detectors, sparks)
}

/// 阶段 4 激光销毁：按通电路径再 trace 并立刻移除材料
pub(super) fn destroy_powered_lasers(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
) -> (Vec<IVec3>, Vec<BreakDebris>) {
    let (_, _, sparks, debris) = run_lasers(world, powered_devices, true);
    (sparks, debris)
}

/// 阶段 4 钻头销毁：本回合只挂起，下一回合开始再移除（等移动动画播完）
pub(super) fn run_drill_destroy_phase(
    world: &WorldBlocks,
    pending_generated: &mut super::pending::PendingGeneratedMaterials,
    ready_turn: u64,
) {
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
            MaterialDestroyer::Drill { target } => {
                mark_material_destroy(
                    world,
                    pending_generated,
                    pos + target,
                    ready_turn,
                    super::pending::PendingDestroyReason::Drill,
                );
            }
            MaterialDestroyer::AdjacentDrillHead => {
                for offset in signal_offsets() {
                    mark_material_destroy(
                        world,
                        pending_generated,
                        pos + offset,
                        ready_turn,
                        super::pending::PendingDestroyReason::Drill,
                    );
                }
            }
            MaterialDestroyer::Laser { .. } => {}
        }
    }
}

/// 阶段 4 传送：本回合只挂起，下一回合开始再搬迁（等移动动画完全进入入口）
pub(super) fn run_material_teleport_phase(
    world: &WorldBlocks,
    pending_generated: &mut super::pending::PendingGeneratedMaterials,
    ready_turn: u64,
) {
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
        if world.is_material_at(exit) || !world.can_move_into(exit) {
            continue;
        }
        pending_generated.mark_teleport(entrance, exit, ready_turn);
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

/// 阶段 4 焊接：本回合移动结束后把相邻焊点上的材料焊成一体；返回成功焊点对
pub(super) fn run_weld_behavior_phase(world: &mut WorldBlocks) -> Vec<(IVec3, IVec3)> {
    let weld_points: Vec<IVec3> = world
        .system_blocks
        .iter()
        .filter_map(|(pos, block)| block.kind.weld_behavior().is_some().then_some(*pos))
        .collect();
    let mut pairs = Vec::new();

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
                pairs.push((weld_point, neighbor));
            }
        }
    }
    pairs
}

/// 阶段 4 装饰漆：只挂起，下一回合开始再写入（等移动动画播完）
pub(super) fn run_material_paint_phase(
    world: &WorldBlocks,
    pending_generated: &mut super::pending::PendingGeneratedMaterials,
    ready_turn: u64,
) {
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
        let connectable = target_block
            .kind
            .material_face_connectable(target_block.facing, face_normal);
        if !connectable {
            continue;
        }
        let paint = world.roller_settings(pos).paint;
        pending_generated.mark_paint(face, paint, ready_turn);
    }
}

/// 回合初落地延后漆（宿主仍在则写入）
pub(super) fn apply_pending_paints(
    world: &mut WorldBlocks,
    pending_generated: &mut super::pending::PendingGeneratedMaterials,
    turn: u64,
) -> bool {
    let mut any = false;
    for (face, paint) in pending_generated.take_ready_paints(turn) {
        if !world.blocks.values().any(|block| block.id == face.block) {
            continue;
        }
        world.material_paints.insert(face, paint);
        any = true;
    }
    any
}

/// 阶段 4 印花：只挂起，下一回合开始再生成附着（等移动动画播完）
pub(super) fn run_material_stamp_phase(
    world: &WorldBlocks,
    pending_generated: &mut super::pending::PendingGeneratedMaterials,
    ready_turn: u64,
) {
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
        let connectable = host
            .kind
            .material_face_connectable(host.facing, face_normal);
        if !connectable {
            continue;
        }

        let existing_child = world
            .material_attachments
            .iter()
            .find(|(_, att)| att.parent == host.id && att.parent_face_normal == face_normal)
            .map(|(child, _)| *child);
        if let Some(child_id) = existing_child {
            let Some((_, child_block)) = world
                .blocks
                .iter()
                .find(|(_, b)| b.id == child_id)
                .map(|(p, b)| (*p, *b))
            else {
                continue;
            };
            let fragile = child_block
                .kind
                .material_props()
                .is_some_and(|props| props.fragile);
            if !fragile {
                continue;
            }
        }

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
        pending_generated.mark_stamp(
            stamper_pos,
            host.id,
            face_normal,
            stamp_id,
            stamp_facing,
            ready_turn,
        );
    }
}

/// 回合初落地延后印花
pub(super) fn apply_pending_stamps(
    world: &mut WorldBlocks,
    pending_generated: &mut super::pending::PendingGeneratedMaterials,
    turn: u64,
) -> bool {
    let mut any = false;
    for (stamper_pos, pending) in pending_generated.take_ready_stamps(turn) {
        if !world.blocks.values().any(|block| block.id == pending.host) {
            continue;
        }
        // 该面已有附着：非脆弱跳过；脆弱碎旧换新
        let existing_child = world
            .material_attachments
            .iter()
            .find(|(_, att)| {
                att.parent == pending.host && att.parent_face_normal == pending.face_normal
            })
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
        if world.blocks.contains_key(&stamper_pos) {
            continue;
        }
        world.insert(
            stamper_pos,
            BlockData::new(BlockKind::Stamp(pending.stamp), pending.stamp_facing),
        );
        let Some(stamp) = world.blocks.get(&stamper_pos).copied() else {
            continue;
        };
        world.material_attachments.insert(
            stamp.id,
            crate::world::grid::MaterialAttachment {
                parent: pending.host,
                parent_face_normal: pending.face_normal,
            },
        );
        any = true;
    }
    any
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
                block: BlockData::new(BlockKind::Material(settings.material), settings.facing),
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

/// 阶段 4 验收：匹配则挂起销毁（下一回合开始移除），验收计数立刻生效
pub(super) fn run_material_acceptance_phase(
    world: &WorldBlocks,
    structure_state: &mut StructureState,
    pending_generated: &mut super::pending::PendingGeneratedMaterials,
    turn: u64,
) -> HashSet<AcceptorId> {
    let ready_turn = turn + 1;
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
            let Some(material) = block.kind.material_id() else {
                break;
            };
            if !world.accepts_material_id_at(*pos, material, block.facing, block.id) {
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
        // 印花附着会并入材料结构，但验收格只要求宿主材料重合
        let host_material: HashSet<_> = welded_material
            .iter()
            .copied()
            .filter(|pos| {
                world
                    .blocks
                    .get(pos)
                    .is_some_and(|block| block.kind.material_id().is_some())
            })
            .collect();
        if host_material != matched_material {
            continue;
        }

        for pos in &welded_material {
            mark_material_destroy(
                world,
                pending_generated,
                *pos,
                ready_turn,
                super::pending::PendingDestroyReason::Accept,
            );
        }
        structure_state.increment_acceptor_count(index);
        eprintln!("[sim turn={turn}] 验收成功");
        if !acceptor_id.is_none() {
            accepted.insert(acceptor_id);
        }
    }
    accepted
}

fn run_lasers(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    destroy: bool,
) -> (Vec<LaserBeam>, HashSet<IVec3>, Vec<IVec3>, Vec<BreakDebris>) {
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
    let mut debris = Vec::new();
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
            &mut debris,
            &mut hit_detectors,
            0,
        );
    }
    (laser_beams, hit_detectors, sparks, debris)
}

/// 挂起销毁：本回合不删格，下一回合开始再落地
fn mark_material_destroy(
    world: &WorldBlocks,
    pending_generated: &mut super::pending::PendingGeneratedMaterials,
    pos: IVec3,
    ready_turn: u64,
    reason: super::pending::PendingDestroyReason,
) {
    let Some(block) = world.blocks.get(&pos) else {
        return;
    };
    if !block.kind.is_material() {
        return;
    }
    pending_generated.mark_destroyed(pos, block.kind, ready_turn, reason);
}

/// 落地一笔挂起的传送（入口仍有材料且出口可进）
pub(super) fn apply_pending_teleport(
    world: &mut WorldBlocks,
    entrance: IVec3,
    exit: IVec3,
) -> bool {
    teleport_entrance_material(world, entrance, exit)
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
    debris: &mut Vec<BreakDebris>,
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
                debris.push(BreakDebris {
                    pos: target,
                    kind: block.kind,
                });
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
                debris,
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
