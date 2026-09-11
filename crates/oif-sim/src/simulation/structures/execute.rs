use super::{
    BlockId, BlockKind, BlockMotion, BlockMotionKind, ExecutedMovement, HashMap, HashSet, IVec3,
    MovementHistory, MovementMark, PusherActor, PusherAnimationKind, PusherMotion, StructureId,
    StructureMove, StructureState, SuctionLinks, WorldBlocks, can_move_own_extended_heads,
    can_rotate_structure, expanded_move_structure, hard_pusher_head_blocked_below,
    hard_pusher_head_blocks_move, movement_expansion_mode, own_extended_heads, rotate_facing,
    rotate_pos_y, rotate_structure, stamp_collisions, stamp_collisions_for_rotation,
    with_factory_attachment_children, with_pusher_heads,
};

pub(in crate::simulation) fn apply_fragile_shatter_before_execute(
    world: &mut WorldBlocks,
    moves: &mut [StructureMove],
    structures: &mut StructureState,
) -> Vec<(IVec3, crate::blocks::BlockKind)> {
    let mut shatter = HashSet::new();
    let mut shatter_stamps = HashMap::new();
    for movement in moves.iter() {
        match movement {
            StructureMove::Translate {
                structure,
                offset,
                actors,
                ..
            } => {
                for actor in actors {
                    if matches!(actor.animation, PusherAnimationKind::Extend) {
                        if let Some(block) = world.blocks.get(&actor.pos) {
                            let head = actor.pos + block.facing.forward_ivec3();
                            if world.is_fragile_material_at(head) {
                                shatter.insert(head);
                            }
                        }
                    }
                }
                if *offset == IVec3::ZERO {
                    continue;
                }
                for collision in stamp_collisions(world, structure, *offset, None) {
                    if let Some(stamp) = world.material_stamps.get(&collision.face).copied() {
                        if crate::blocks::stamp_def(stamp).fragile {
                            shatter_stamps.insert(
                                collision.face,
                                (collision.target, crate::blocks::BlockKind::Stamp(stamp)),
                            );
                        }
                    }
                }
                for pos in structure {
                    let target = *pos + *offset;
                    if !structure.contains(&target) && world.is_fragile_material_at(target) {
                        shatter.insert(target);
                    }
                    if world.is_fragile_material_at(*pos)
                        && target.y >= 0
                        && !structure.contains(&target)
                        && !world.cell_accepts_move_from(*pos, target)
                        && !world.is_fragile_material_at(target)
                    {
                        shatter.insert(*pos);
                    }
                }
            }
            StructureMove::Rotate {
                structure,
                pivot,
                clockwise,
                ..
            } => {
                for collision in stamp_collisions_for_rotation(world, structure, *pivot, *clockwise)
                {
                    if let Some(stamp) = world.material_stamps.get(&collision.face).copied() {
                        if crate::blocks::stamp_def(stamp).fragile {
                            shatter_stamps.insert(
                                collision.face,
                                (collision.target, crate::blocks::BlockKind::Stamp(stamp)),
                            );
                        }
                    }
                }
            }
        }
    }
    if shatter.is_empty() && shatter_stamps.is_empty() {
        return Vec::new();
    }

    let mut debris = Vec::new();
    let mut affected: HashMap<StructureId, HashSet<IVec3>> = HashMap::new();
    for pos in &shatter {
        if let Some(block) = world.blocks.get(pos).copied() {
            if block.kind.is_material() {
                debris.push((*pos, block.kind));
            }
        }
        if let Some(id) = structures.id_at(*pos) {
            affected.entry(id).or_default().insert(*pos);
        }
        world.remove(pos);
    }
    let shattered_stamps = !shatter_stamps.is_empty();
    for (face, (pos, kind)) in shatter_stamps {
        world.material_stamps.remove(&face);
        debris.push((pos, kind));
    }
    if shattered_stamps {
        world.topology_revision = world.topology_revision.wrapping_add(1);
    }
    for (id, removed) in affected {
        let Some(old) = structures.structure_positions(id).cloned() else {
            continue;
        };
        let new_positions: HashSet<IVec3> = old.difference(&removed).copied().collect();
        structures.replace_structure_positions(world, &old, new_positions);
    }
    for movement in moves.iter_mut() {
        match movement {
            StructureMove::Translate { structure, .. }
            | StructureMove::Rotate { structure, .. } => {
                for pos in &shatter {
                    structure.remove(pos);
                }
            }
        }
    }
    debris
}

/// 按序执行运动标签：失败则试下一个；种子判占用，成功后标记展开后的格子。
/// `hard_pusher_head_occupancy` 为本回合开始时已伸出的头；执行中随 Push 伸出/收回更新。
/// `moved`：本回合真实平移过的格子。
/// `gravity_held`：已有推杆动画的整坨，只抑重力。
/// `push_held`：本回合已平移或已开推动画的格子，后续非零 Push 不可再吃。
/// 抬升与重力的互斥在 merge 阶段完成（有抬升标签则丢掉重叠重力）。
pub(in crate::simulation) fn execute_structure_moves_with_pushers(
    world: &mut WorldBlocks,
    moves: Vec<StructureMove>,
    structures: &mut StructureState,
    history: &mut MovementHistory,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
    suction: &SuctionLinks,
) -> (
    HashMap<IVec3, BlockMotion>,
    HashMap<IVec3, PusherMotion>,
    HashMap<BlockId, (IVec3, bool)>,
) {
    let mut moved = HashSet::new();
    let mut gravity_held = HashSet::new();
    let mut push_held = HashSet::new();
    let mut animations = HashMap::new();
    let mut pusher_animations = HashMap::new();
    let mut extension_commits = HashMap::new();
    let mut executed = Vec::new();
    let mut heads = hard_pusher_head_occupancy.clone();
    let mut gravity_held_structures = HashSet::new();
    let mut block_positions: HashMap<BlockId, IVec3> = moves
        .iter()
        .filter_map(|movement| match movement {
            StructureMove::Translate { actors, .. } => Some(actors.iter()),
            StructureMove::Rotate { .. } => None,
        })
        .flatten()
        .map(|actor| (actor.id, actor.pos))
        .collect();
    // 本回合收回的头先逻辑腾出，便于对向「伸+收」同向共用中间格
    for movement in &moves {
        if let StructureMove::Translate { actors, .. } = movement {
            for actor in actors {
                if matches!(actor.animation, PusherAnimationKind::Retract) {
                    let actor_pos = block_positions.get(&actor.id).copied().unwrap_or(actor.pos);
                    if let Some(block) = world.blocks.get(&actor_pos) {
                        heads.remove(&(actor_pos + block.facing.forward_ivec3()));
                    }
                }
            }
        }
    }
    for movement in moves {
        match movement {
            StructureMove::Translate {
                structure_id,
                structure,
                offset,
                actors,
                mark,
                source,
                source_pos: _,
            } => {
                let is_gravity = matches!(mark, MovementMark::Vertical) && source.is_none();
                // 仅用种子结构判占用；展开在当前世界上做，避免预展开导致误跳过
                if is_gravity {
                    if structure
                        .iter()
                        .any(|pos| moved.contains(pos) || gravity_held.contains(pos))
                    {
                        continue;
                    }
                } else if offset != IVec3::ZERO
                    && structure
                        .iter()
                        .any(|pos| moved.contains(pos) || push_held.contains(pos))
                {
                    continue;
                }
                // 收回标签检查时忽略自己的头，否则粘头拉回会撞上尚未释放的头占位
                let mut heads_for_check = heads.clone();
                for actor in &actors {
                    if matches!(actor.animation, PusherAnimationKind::Retract) {
                        let actor_pos =
                            block_positions.get(&actor.id).copied().unwrap_or(actor.pos);
                        if let Some(block) = world.blocks.get(&actor_pos) {
                            heads_for_check.remove(&(actor_pos + block.facing.forward_ivec3()));
                        }
                    }
                }
                let seed = structure.clone();
                let Some(structure) = expanded_move_structure(
                    world,
                    &structure,
                    offset,
                    structures,
                    movement_expansion_mode(mark, source),
                    suction,
                ) else {
                    continue;
                };
                // 展开卷入的格子若本回合已真实平移过 / 重力 hold，本标签失败
                if is_gravity {
                    if structure
                        .iter()
                        .any(|pos| moved.contains(pos) || gravity_held.contains(pos))
                    {
                        continue;
                    }
                } else if offset != IVec3::ZERO
                    && structure
                        .iter()
                        .any(|pos| moved.contains(pos) || push_held.contains(pos))
                {
                    continue;
                }
                // 活塞头是实体：本回合已提交的头会挡住后续更低优先级移动（自带头随结构走，不挡自己）
                if offset != IVec3::ZERO
                    && (hard_pusher_head_blocks_move(world, &structure, offset, &heads_for_check)
                        || !can_move_own_extended_heads(
                            world,
                            &structure,
                            offset,
                            &heads_for_check,
                        ))
                {
                    continue;
                }
                if offset == IVec3::NEG_Y
                    && hard_pusher_head_blocked_below(world, &seed, &heads_for_check)
                {
                    continue;
                }
                let own_heads_before = if offset != IVec3::ZERO {
                    own_extended_heads(world, &structure, &heads)
                } else {
                    HashSet::new()
                };
                // 平移前筛推杆：仅伸出时，伸头终点若落在同标签其它成员落点上则丢掉（A/B 同推 {B,C}）。
                // 收回把物体拉进头格是预期，不能用同一规则滤掉收回动画。
                let actors: Vec<PusherActor> = actors
                    .into_iter()
                    .filter(|actor| {
                        if matches!(actor.animation, PusherAnimationKind::Retract) {
                            return block_positions.contains_key(&actor.id);
                        }
                        let Some(actor_pos) = block_positions.get(&actor.id).copied() else {
                            return false;
                        };
                        let Some(block) = world.blocks.get(&actor_pos) else {
                            return false;
                        };
                        let forward = block.facing.forward_ivec3();
                        let head_final = if structure.contains(&actor_pos) {
                            actor_pos + offset + forward
                        } else {
                            actor_pos + forward
                        };
                        !(offset != IVec3::ZERO && structure.contains(&(head_final - offset)))
                    })
                    .collect();
                if offset != IVec3::ZERO {
                    for pos in &structure {
                        if let Some(block) = world.blocks.get(pos) {
                            if let Some(actor_pos) = block_positions.get_mut(&block.id) {
                                *actor_pos = *pos + offset;
                            }
                        }
                    }
                    for pos in &structure {
                        if let Some(block) = world.blocks.get(pos) {
                            if block.kind == BlockKind::PusherHead {
                                continue;
                            }
                            animations.insert(
                                *pos + offset,
                                BlockMotion {
                                    block_id: block.id,
                                    from_pos: *pos,
                                    to_pos: *pos + offset,
                                    from_facing: block.facing,
                                    to_facing: block.facing,
                                    kind: BlockMotionKind::Move,
                                },
                            );
                        }
                    }
                    // 同回合稍后还会动这块：先把已挂上的推杆动画键挪到新格
                    for pos in &structure {
                        if let Some(motion) = pusher_animations.remove(pos) {
                            pusher_animations.insert(*pos + offset, motion);
                        }
                    }
                    moved.extend(structure.iter().copied());
                    push_held.extend(structure.iter().copied());
                    // 真实头并入集合后由 relocate 一并平移
                    let structure = with_pusher_heads(world, &structure);
                    structures.translate(world, &structure, offset);
                    for head in own_heads_before {
                        heads.remove(&head);
                        heads.insert(head + offset);
                    }
                    let target_structure: HashSet<IVec3> =
                        structure.iter().map(|pos| *pos + offset).collect();
                    moved.extend(target_structure.iter().copied());
                    push_held.extend(target_structure);
                }
                for actor in actors {
                    let Some(actor_pos) = block_positions.get(&actor.id).copied() else {
                        continue;
                    };
                    let Some(block) = world.blocks.get(&actor_pos).copied() else {
                        continue;
                    };
                    let (from_extension, to_extension) = match actor.animation {
                        PusherAnimationKind::Extend => (0.0, 1.0),
                        PusherAnimationKind::Retract => (1.0, 0.0),
                    };
                    pusher_animations.insert(
                        actor_pos,
                        PusherMotion {
                            from_extension,
                            to_extension,
                        },
                    );
                    extension_commits.insert(actor.id, (actor_pos, to_extension > 0.5));
                    let head = actor_pos + block.facing.forward_ivec3();
                    match actor.animation {
                        PusherAnimationKind::Extend => {
                            heads.insert(head);
                        }
                        PusherAnimationKind::Retract => {
                            heads.remove(&head);
                        }
                    }
                    // 推杆开启动画：整坨抑重力；push_held 只锁本体，
                    // 避免挡住同结构其它杆对另一侧子集的 BoundFront
                    if let Some(actor_structure_id) = structures.id_at(actor_pos) {
                        if gravity_held_structures.insert(actor_structure_id)
                            && let Some(actor_structure) =
                                structures.structure_positions(actor_structure_id)
                        {
                            gravity_held.extend(actor_structure.iter().copied());
                        }
                    }
                    push_held.insert(actor_pos);
                }
                if let Some(source) = source {
                    executed.push(ExecutedMovement {
                        structure_id,
                        source,
                    });
                }
            }
            StructureMove::Rotate {
                structure_id,
                structure,
                pivot,
                clockwise,
                source,
                source_pos,
            } => {
                let structure = with_factory_attachment_children(world, &structure);
                if structure.iter().any(|pos| moved.contains(pos)) {
                    continue;
                }
                if can_rotate_structure(world, &structure, pivot, clockwise) {
                    for pos in &structure {
                        if let Some(block) = world.blocks.get(pos) {
                            if let Some(actor_pos) = block_positions.get_mut(&block.id) {
                                *actor_pos = rotate_pos_y(*pos, pivot, clockwise);
                            }
                        }
                    }
                    let targets: Vec<IVec3> = structure
                        .iter()
                        .map(|pos| rotate_pos_y(*pos, pivot, clockwise))
                        .collect();
                    for pos in &structure {
                        if let Some(block) = world.blocks.get(pos) {
                            let target = rotate_pos_y(*pos, pivot, clockwise);
                            animations.insert(
                                target,
                                BlockMotion {
                                    block_id: block.id,
                                    from_pos: *pos,
                                    to_pos: target,
                                    from_facing: block.facing,
                                    to_facing: rotate_facing(block.facing, clockwise),
                                    kind: BlockMotionKind::Rotate { pivot, clockwise },
                                },
                            );
                        }
                    }
                    moved.extend(structure.iter().copied());
                    rotate_structure(world, structures, &structure, pivot, clockwise);
                    if let Some(rotator_pos) = source_pos {
                        if let Some(id) = world
                            .blocks
                            .get(&(rotator_pos + IVec3::Y))
                            .filter(|block| block.kind.is_material() || block.kind.is_factory())
                            .map(|block| block.id)
                        {
                            world.mark_rotator_arrival(rotator_pos, id);
                        }
                    }
                    let target_structure: HashSet<IVec3> = targets.iter().copied().collect();
                    if let Some(source) = source {
                        executed.push(ExecutedMovement {
                            structure_id,
                            source,
                        });
                    }
                    moved.extend(target_structure);
                }
            }
        }
    }
    history.record_executed(executed);
    (animations, pusher_animations, extension_commits)
}
