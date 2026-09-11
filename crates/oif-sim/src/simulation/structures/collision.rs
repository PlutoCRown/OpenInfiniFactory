use super::{
    BlockData, BlockId, BlockKind, Facing, FactoryActivity, HashMap, HashSet, IVec3, MaterialFace,
    MovementMark, MovementRule, MovingOccupancy, StructureId, StructureKind, StructureState,
    SuctionLinks, VecDeque, WorldBlocks,
};

/// 抬到 range 上一格（range=5 时为第 6 格）后悬停：已出抬升标记范围，靠此抑制重力，避免边缘上下弹跳
pub(super) fn structure_supported_by_lifter(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
) -> bool {
    // Lifter range 固定 5：只查正下方第 6 格，避免全图 / 多段扫描
    structure.iter().any(|pos| {
        let candidate = *pos - IVec3::Y * 6;
        world
            .blocks
            .get(&candidate)
            .is_some_and(|block| block.kind == BlockKind::Lifter)
    })
}

/// 结构是否直接搁在场景 / Inactive 上（找到一处即可）
fn structure_id_rests_on_stable_support(
    world: &WorldBlocks,
    structures: &StructureState,
    id: StructureId,
) -> bool {
    let Some(structure) = structures.get(id) else {
        return false;
    };
    structure.positions.iter().any(|pos| {
        let below = *pos + IVec3::NEG_Y;
        if below.y < 0 || structure.positions.contains(&below) {
            return false;
        }
        world.is_scene_at(below)
            || structures.structure_id_at(below).is_some_and(|below_id| {
                structures.get(below_id).is_some_and(|s| {
                    s.kind == StructureKind::Factory && s.activity == FactoryActivity::Inactive
                })
            })
    })
}

/// 重力是否被支撑链接地（场景/Inactive，或下方 Active 已接地）；带 memo，避免重复扫
pub(super) fn structure_id_gravity_grounded(
    world: &WorldBlocks,
    structures: &StructureState,
    id: StructureId,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
    memo: &mut HashMap<StructureId, bool>,
) -> bool {
    if let Some(&cached) = memo.get(&id) {
        return cached;
    }
    // 先占位：交错支撑（A 压 B、B 又压到 A）时避免递归环把 worker 打爆
    memo.insert(id, false);
    let Some(structure) = structures.get(id) else {
        memo.insert(id, true);
        return true;
    };
    let mut grounded = false;
    for pos in &structure.positions {
        let below = *pos + IVec3::NEG_Y;
        if below.y < 0 {
            grounded = true;
            break;
        }
        if structure.positions.contains(&below) {
            continue;
        }
        if world.can_move_into_yielding_fragile(below)
            && !hard_pusher_head_occupancy.contains(&below)
        {
            continue;
        }
        if world.is_scene_at(below) {
            grounded = true;
            break;
        }
        if let Some(below_id) = structures.structure_id_at(below) {
            if below_id == id {
                continue;
            }
            if structures.get(below_id).is_some_and(|s| {
                s.kind == StructureKind::Factory && s.activity == FactoryActivity::Inactive
            }) {
                grounded = true;
                break;
            }
            if structure_id_gravity_grounded(
                world,
                structures,
                below_id,
                hard_pusher_head_occupancy,
                memo,
            ) {
                grounded = true;
                break;
            }
        } else {
            // 非结构实体（如活塞头）挡住
            grounded = true;
            break;
        }
    }
    memo.insert(id, grounded);
    grounded
}

/// 运动执行前：按计划压碎/让出冲突的脆弱材料（与钻头/激光销毁分离）
/// 返回碎裂格子与种类，供表现层生成碎片
pub(super) fn can_move_gravity_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    structures: &StructureState,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
    suction: &SuctionLinks,
) -> bool {
    if hard_pusher_head_blocked_below(world, structure, hard_pusher_head_occupancy) {
        return false;
    }
    let Some(expanded) = expanded_move_structure(
        world,
        structure,
        IVec3::NEG_Y,
        structures,
        MovementExpansionMode::Gravity,
        suction,
    ) else {
        return false;
    };
    !hard_pusher_head_blocks_move(world, &expanded, IVec3::NEG_Y, hard_pusher_head_occupancy)
        && can_move_own_extended_heads(world, &expanded, IVec3::NEG_Y, hard_pusher_head_occupancy)
}

pub(super) fn hard_pusher_head_blocked_below(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> bool {
    structure.iter().any(|pos| {
        let Some(block) = world.blocks.get(pos) else {
            return false;
        };
        if !matches!(
            block.kind.movement_rule(block.facing),
            Some(crate::blocks::MovementRule::PoweredTranslate { .. })
        ) {
            return false;
        }
        let head = *pos + block.facing.forward_ivec3();
        if !hard_pusher_head_occupancy.contains(&head) {
            return false;
        }
        let target = head + IVec3::NEG_Y;
        // 活塞头是实体：下方有方块或其它活塞头都算挡住
        target.y < 0
            || (!structure.contains(&target)
                && !own_extended_heads(world, structure, hard_pusher_head_occupancy)
                    .contains(&target)
                && (!world.can_move_into_yielding_fragile(target)
                    || hard_pusher_head_occupancy.contains(&target)))
    })
}

/// 结构内已伸出推杆的头格（随结构一起平移）
pub(super) fn own_extended_heads(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> HashSet<IVec3> {
    structure
        .iter()
        .filter_map(|pos| {
            let block = world.blocks.get(pos)?;
            if !matches!(
                block.kind.movement_rule(block.facing),
                Some(crate::blocks::MovementRule::PoweredTranslate { .. })
            ) {
                return None;
            }
            let head = *pos + block.facing.forward_ivec3();
            if !hard_pusher_head_occupancy.contains(&head) {
                return None;
            }
            // 必须是本杆的头：面对面时对方头也可能落在同一前格方向上
            let head_block = world.blocks.get(&head)?;
            (head_block.kind == BlockKind::PusherHead
                && head - head_block.facing.forward_ivec3() == *pos)
                .then_some(head)
        })
        .collect()
}

/// 平移体积（体格 ∪ 自带头）是否会撞上外来活塞头
pub(super) fn hard_pusher_head_blocks_move(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    offset: IVec3,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> bool {
    let own = own_extended_heads(world, structure, hard_pusher_head_occupancy);
    let volume: HashSet<IVec3> = structure.iter().chain(own.iter()).copied().collect();
    volume.iter().any(|pos| {
        let target = *pos + offset;
        !volume.contains(&target) && hard_pusher_head_occupancy.contains(&target)
    })
}

/// 自带头平移后是否有落脚处（不钻进实心/外来头）
pub(super) fn can_move_own_extended_heads(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    offset: IVec3,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> bool {
    let own = own_extended_heads(world, structure, hard_pusher_head_occupancy);
    let volume: HashSet<IVec3> = structure.iter().chain(own.iter()).copied().collect();
    own.iter().all(|head| {
        let target = *head + offset;
        volume.contains(&target)
            || (target.y >= 0
                && world.can_move_into_yielding_fragile(target)
                && !hard_pusher_head_occupancy.contains(&target))
    })
}

/// 把结构内推杆的真实伸出头并入移动集合
pub(super) fn with_pusher_heads(world: &WorldBlocks, structure: &HashSet<IVec3>) -> HashSet<IVec3> {
    let mut expanded = structure.clone();
    for &pos in structure {
        let Some(block) = world.blocks.get(&pos) else {
            continue;
        };
        if !matches!(
            block.kind.movement_rule(block.facing),
            Some(MovementRule::PoweredTranslate { .. })
        ) {
            continue;
        }
        let head = pos + block.facing.forward_ivec3();
        // 仅并入「属于该本体」的头；面对面时前格可能是对面杆的头
        if world.blocks.get(&head).is_some_and(|head_block| {
            head_block.kind == BlockKind::PusherHead
                && head - head_block.facing.forward_ivec3() == pos
        }) {
            expanded.insert(head);
        }
    }
    expanded
}

/// 格上是否为真实伸出头（结构平移规划时当空格，碰撞仍走 has_collision）
fn is_pusher_head_at(world: &WorldBlocks, pos: IVec3) -> bool {
    world
        .blocks
        .get(&pos)
        .is_some_and(|block| block.kind == BlockKind::PusherHead)
}

pub(super) fn expanded_move_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    offset: IVec3,
    structures: &StructureState,
    mode: MovementExpansionMode,
    suction: &SuctionLinks,
) -> Option<HashSet<IVec3>> {
    expanded_move_structure_with_occupancy(
        world, structure, offset, structures, mode, suction, None,
    )
}

/// 移动后印花凸出的一格是否撞上外部方块；印花本身不进入世界占用表
#[derive(Clone, Copy)]
pub(in crate::simulation) struct StampCollision {
    pub face: MaterialFace,
    pub target: IVec3,
}

/// 收集结构平移后会撞上的印花面
pub(in crate::simulation) fn stamp_collisions(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    offset: IVec3,
    occupancy: Option<&MovingOccupancy>,
) -> Vec<StampCollision> {
    if offset == IVec3::ZERO || world.material_stamps.is_empty() {
        return Vec::new();
    }
    let parent_ids: HashSet<BlockId> = structure
        .iter()
        .filter_map(|pos| world.blocks.get(pos).map(|block| block.id))
        .filter(|id| !id.is_none())
        .collect();
    let stamp_positions: HashSet<IVec3> = world
        .material_stamps
        .keys()
        .filter(|face| parent_ids.contains(&face.block))
        .filter_map(|face| {
            let parent_pos = structure.iter().find(|pos| {
                world
                    .blocks
                    .get(pos)
                    .is_some_and(|block| block.id == face.block)
            })?;
            Some(*parent_pos + face.normal)
        })
        .collect();
    let moving_volume: HashSet<IVec3> = structure
        .iter()
        .copied()
        .chain(stamp_positions.iter().copied())
        .collect();

    world
        .material_stamps
        .iter()
        .filter(|(face, _)| parent_ids.contains(&face.block))
        .filter_map(|(face, _stamp)| {
            let parent_pos = structure.iter().find(|pos| {
                world
                    .blocks
                    .get(pos)
                    .is_some_and(|block| block.id == face.block)
            })?;
            let stamp_pos = *parent_pos + face.normal;
            let target = stamp_pos + offset;
            if moving_volume.contains(&target)
                || occupancy
                    .and_then(|claims| claims.velocity_at(target))
                    .is_some_and(|velocity| velocity == offset)
                || world.can_move_into(target)
            {
                return None;
            }
            Some(StampCollision {
                face: *face,
                target,
            })
        })
        .collect()
}

/// 展开推动链；`occupancy` 下同向离开的格视为空（不并入），异速占用则失败
pub(in crate::simulation) fn expanded_move_structure_with_occupancy(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    offset: IVec3,
    structures: &StructureState,
    mode: MovementExpansionMode,
    suction: &SuctionLinks,
    occupancy: Option<&MovingOccupancy>,
) -> Option<HashSet<IVec3>> {
    let structure = structures.linked_expand_pusher_subset(suction, structure, offset)?;
    let structure = with_factory_attachment_children(world, &structure);
    let structure = with_pusher_heads(world, &structure);

    let stamp_collisions = stamp_collisions(world, &structure, offset, occupancy);
    if stamp_collisions
        .iter()
        .any(|collision| !crate::blocks::stamp_def(world.material_stamps[&collision.face]).fragile)
    {
        return None;
    }

    if offset.abs().element_sum() != 1 {
        return can_move_structure_without_push_occupancy(world, &structure, offset, occupancy)
            .then_some(structure);
    }

    let mut expanded = structure.clone();
    let mut queue: VecDeque<IVec3> = structure.iter().copied().collect();
    while let Some(pos) = queue.pop_front() {
        let target = pos + offset;
        if target.y < 0 || expanded.contains(&target) {
            continue;
        }
        if world.cell_accepts_move_from(pos, target) {
            continue;
        }
        if is_pusher_head_at(world, target) {
            continue;
        }
        if let Some(occ) = occupancy {
            if let Some(v) = occ.velocity_at(target) {
                if v == offset {
                    // 独立结构同向离开：不并入，视为可进入
                    continue;
                }
                return None;
            }
        }

        if mode == MovementExpansionMode::Gravity {
            if let Some(sid) = structures.structure_id_at(target) {
                // 下方结构已接地则整条重力链失败；先查 id 再克隆
                if structure_id_rests_on_stable_support(world, structures, sid)
                    || structures
                        .get(sid)
                        .is_some_and(|s| structure_supported_by_lifter(world, &s.positions))
                {
                    return None;
                }
            }
        }
        let pushed = pushable_structure_at(world, structures, target, offset, suction)?;
        let pushed = with_factory_attachment_children(world, &pushed);
        let pushed = with_pusher_heads(world, &pushed);
        for pushed_pos in pushed {
            if expanded.insert(pushed_pos) {
                queue.push_back(pushed_pos);
            }
        }
    }

    can_move_structure_without_push_occupancy(world, &expanded, offset, occupancy)
        .then_some(expanded)
}

fn can_move_structure_without_push_occupancy(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    offset: IVec3,
    occupancy: Option<&MovingOccupancy>,
) -> bool {
    structure.iter().all(|pos| {
        let target = *pos + offset;
        if target.y < 0 {
            return false;
        }
        if structure.contains(&target)
            || world.cell_accepts_move_from(*pos, target)
            || is_pusher_head_at(world, target)
        {
            return true;
        }
        if let Some(occ) = occupancy {
            if let Some(v) = occ.velocity_at(target) {
                return v == offset;
            }
        }
        world.is_fragile_material_at(*pos)
    })
}

/// 把工厂附着子格（告示等）并入待移动集合
pub(super) fn with_factory_attachment_children(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
) -> HashSet<IVec3> {
    if world.factory_attachments.is_empty() {
        return structure.clone();
    }
    let id_to_pos: HashMap<BlockId, IVec3> = world
        .blocks
        .iter()
        .filter(|(_, block)| !block.id.is_none())
        .map(|(pos, block)| (block.id, *pos))
        .collect();
    let parent_ids: HashSet<BlockId> = structure
        .iter()
        .filter_map(|pos| world.blocks.get(pos).map(|block| block.id))
        .filter(|id| !id.is_none())
        .collect();
    let mut expanded = structure.clone();
    for (child_id, att) in &world.factory_attachments {
        if !parent_ids.contains(&att.parent) {
            continue;
        }
        if let Some(&child_pos) = id_to_pos.get(child_id) {
            expanded.insert(child_pos);
        }
    }
    expanded
}

pub(in crate::simulation) fn can_translate_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    offset: IVec3,
    structures: &StructureState,
    suction: &SuctionLinks,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> bool {
    let Some(expanded) = expanded_move_structure(
        world,
        structure,
        offset,
        structures,
        MovementExpansionMode::Normal,
        suction,
    ) else {
        return false;
    };
    !hard_pusher_head_blocks_move(world, &expanded, offset, hard_pusher_head_occupancy)
        && can_move_own_extended_heads(world, &expanded, offset, hard_pusher_head_occupancy)
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(in crate::simulation) enum MovementExpansionMode {
    Normal,
    Gravity,
}

pub(in crate::simulation) fn movement_expansion_mode(
    mark: MovementMark,
    source: Option<BlockId>,
) -> MovementExpansionMode {
    if mark == MovementMark::Vertical && source.is_none() {
        MovementExpansionMode::Gravity
    } else {
        MovementExpansionMode::Normal
    }
}

fn pushable_structure_at(
    world: &WorldBlocks,
    structures: &StructureState,
    pos: IVec3,
    offset: IVec3,
    suction: &SuctionLinks,
) -> Option<HashSet<IVec3>> {
    let block = world.blocks.get(&pos)?;
    // PusherHead 不是货物；顶头推挤由 mark_pusher 的 body_at_extended_head 路径处理
    if block.kind == BlockKind::PusherHead {
        return None;
    }
    if block.kind.is_material() || block.kind.is_factory() {
        return structures.linked_pushable_at(suction, pos, offset);
    }
    None
}

pub(in crate::simulation) fn can_rotate_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    pivot: IVec3,
    clockwise: bool,
) -> bool {
    if stamp_collisions_for_rotation(world, structure, pivot, clockwise)
        .iter()
        .any(|collision| !crate::blocks::stamp_def(world.material_stamps[&collision.face]).fragile)
    {
        return false;
    }
    structure.iter().all(|pos| {
        let target = rotate_pos_y(*pos, pivot, clockwise);
        target.y >= 0 && (structure.contains(&target) || world.can_place_platform_at(target))
    })
}

/// 收集结构旋转后会撞上的印花面
pub(in crate::simulation) fn stamp_collisions_for_rotation(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    pivot: IVec3,
    clockwise: bool,
) -> Vec<StampCollision> {
    if world.material_stamps.is_empty() {
        return Vec::new();
    }
    let parent_ids: HashSet<BlockId> = structure
        .iter()
        .filter_map(|pos| world.blocks.get(pos).map(|block| block.id))
        .filter(|id| !id.is_none())
        .collect();
    let moving_volume: HashSet<IVec3> = structure
        .iter()
        .map(|pos| rotate_pos_y(*pos, pivot, clockwise))
        .chain(world.material_stamps.keys().filter_map(|face| {
            if !parent_ids.contains(&face.block) {
                return None;
            }
            let parent_pos = structure.iter().find(|pos| {
                world
                    .blocks
                    .get(pos)
                    .is_some_and(|block| block.id == face.block)
            })?;
            Some(
                rotate_pos_y(*parent_pos, pivot, clockwise)
                    + rotate_offset_y(face.normal, clockwise),
            )
        }))
        .collect();

    world
        .material_stamps
        .iter()
        .filter(|(face, _)| parent_ids.contains(&face.block))
        .filter_map(|(face, _)| {
            let parent_pos = structure.iter().find(|pos| {
                world
                    .blocks
                    .get(pos)
                    .is_some_and(|block| block.id == face.block)
            })?;
            let target = rotate_pos_y(*parent_pos, pivot, clockwise)
                + rotate_offset_y(face.normal, clockwise);
            if moving_volume.contains(&target) || world.can_move_into(target) {
                return None;
            }
            Some(StampCollision {
                face: *face,
                target,
            })
        })
        .collect()
}

/// 同步提交旋转后的世界占格、面附着与结构索引
pub(in crate::simulation) fn rotate_structure(
    world: &mut WorldBlocks,
    structures: &mut StructureState,
    structure: &HashSet<IVec3>,
    pivot: IVec3,
    clockwise: bool,
) {
    let was_synced = structures
        .material_topology
        .as_ref()
        .is_some_and(|revision| std::sync::Arc::ptr_eq(revision, &world.material_topology));
    let structure_ids: HashSet<BlockId> = structure
        .iter()
        .filter_map(|pos| world.blocks.get(pos).map(|block| block.id))
        .collect();
    let moves: Vec<(IVec3, IVec3, BlockData)> = structure
        .iter()
        .filter_map(|pos| {
            world.blocks.get(pos).copied().map(|mut block| {
                block.facing = rotate_facing(block.facing, clockwise);
                (*pos, rotate_pos_y(*pos, pivot, clockwise), block)
            })
        })
        .collect();
    world.relocate_blocks(moves);

    // 焊接按 BlockId 无需改写；旋转只更新面附着法线
    let updated_paints: HashMap<_, _> = world
        .material_paints
        .iter()
        .map(|(face, color)| {
            if structure_ids.contains(&face.block) {
                (
                    MaterialFace {
                        block: face.block,
                        normal: rotate_offset_y(face.normal, clockwise),
                    },
                    *color,
                )
            } else {
                (*face, *color)
            }
        })
        .collect();
    world.material_paints = updated_paints;

    // 印花面法线随宿主结构绕 Y 旋转
    let updated_stamps: HashMap<_, _> = world
        .material_stamps
        .iter()
        .map(|(face, stamp)| {
            if structure_ids.contains(&face.block) {
                (
                    MaterialFace {
                        block: face.block,
                        normal: rotate_offset_y(face.normal, clockwise),
                    },
                    *stamp,
                )
            } else {
                (*face, *stamp)
            }
        })
        .collect();
    world.material_stamps = updated_stamps;
    for att in world.factory_attachments.values_mut() {
        if structure_ids.contains(&att.parent) {
            att.parent_face_normal = rotate_offset_y(att.parent_face_normal, clockwise);
        }
    }

    let updated_panels: HashSet<_> = world
        .wire_face_panels
        .iter()
        .map(|face| {
            if structure_ids.contains(&face.block) {
                MaterialFace {
                    block: face.block,
                    normal: rotate_offset_y(face.normal, clockwise),
                }
            } else {
                *face
            }
        })
        .collect();
    if updated_panels != world.wire_face_panels {
        world.wire_face_panels = updated_panels;
        world.topology_revision = world.topology_revision.wrapping_add(1);
        world.invalidate_signal_topology();
    }
    let target_positions = structure
        .iter()
        .map(|pos| rotate_pos_y(*pos, pivot, clockwise))
        .collect();
    structures.replace_structure_positions(world, structure, target_positions);
    if was_synced {
        structures.material_topology = Some(world.material_topology.clone());
    }
}

pub(in crate::simulation) fn rotate_pos_y(pos: IVec3, pivot: IVec3, clockwise: bool) -> IVec3 {
    let rel = pos - pivot;
    pivot + rotate_offset_y(rel, clockwise)
}

fn rotate_offset_y(offset: IVec3, clockwise: bool) -> IVec3 {
    if clockwise {
        IVec3::new(-offset.z, offset.y, offset.x)
    } else {
        IVec3::new(offset.z, offset.y, -offset.x)
    }
}

pub(super) fn rotate_facing(facing: Facing, clockwise: bool) -> Facing {
    if clockwise {
        facing.rotate()
    } else {
        facing.rotate_counter()
    }
}
