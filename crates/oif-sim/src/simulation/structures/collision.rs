/// 抬到 range 上一格（range=5 时为第 6 格）后悬停：已出抬升标记范围，靠此抑制重力，避免边缘上下弹跳
fn structure_supported_by_lifter(world: &WorldBlocks, structure: &HashSet<IVec3>) -> bool {
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
fn structure_id_gravity_grounded(
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
/// 按运行时 BlockId 查找当前坐标
fn block_pos_by_id(world: &WorldBlocks, id: BlockId) -> Option<IVec3> {
    world
        .blocks
        .iter()
        .find_map(|(pos, block)| (block.id == id).then_some(*pos))
}

fn can_move_gravity_structure(
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

fn hard_pusher_head_blocked_below(
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
fn own_extended_heads(
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
fn hard_pusher_head_blocks_move(
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
fn can_move_own_extended_heads(
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
fn with_pusher_heads(world: &WorldBlocks, structure: &HashSet<IVec3>) -> HashSet<IVec3> {
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

fn expanded_move_structure(
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

/// 展开推动链；`occupancy` 下同向离开的格视为空（不并入），异速占用则失败
pub(super) fn expanded_move_structure_with_occupancy(
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
fn with_factory_attachment_children(
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

pub(super) fn can_translate_structure(
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
pub(super) enum MovementExpansionMode {
    Normal,
    Gravity,
}

pub(super) fn movement_expansion_mode(
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

pub(super) fn move_structure(world: &mut WorldBlocks, structure: &HashSet<IVec3>, offset: IVec3) {
    let moves: Vec<(IVec3, IVec3, BlockData)> = structure
        .iter()
        .filter_map(|pos| {
            world
                .blocks
                .get(pos)
                .copied()
                .map(|block| (*pos, *pos + offset, block))
        })
        .collect();
    world.relocate_blocks(moves);
}

pub(super) fn can_rotate_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    pivot: IVec3,
    clockwise: bool,
) -> bool {
    structure.iter().all(|pos| {
        let target = rotate_pos_y(*pos, pivot, clockwise);
        target.y >= 0 && (structure.contains(&target) || world.can_place_platform_at(target))
    })
}

pub(super) fn rotate_structure(
    world: &mut WorldBlocks,
    structure: &HashSet<IVec3>,
    pivot: IVec3,
    clockwise: bool,
) {
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

    // 附着法线随结构绕 Y 旋转
    for att in world.material_attachments.values_mut() {
        if structure_ids.contains(&att.parent) {
            att.parent_face_normal = rotate_offset_y(att.parent_face_normal, clockwise);
        }
    }
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
    }
}

pub(super) fn rotate_pos_y(pos: IVec3, pivot: IVec3, clockwise: bool) -> IVec3 {
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

fn rotate_facing(facing: Facing, clockwise: bool) -> Facing {
    if clockwise {
        facing.rotate()
    } else {
        facing.rotate_counter()
    }
}
