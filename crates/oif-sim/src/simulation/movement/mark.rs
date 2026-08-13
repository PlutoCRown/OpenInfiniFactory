pub(super) fn mark_structure_movement_phase(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    structures: &mut StructureState,
    pusher_state: &mut PusherState,
    suction: &SuctionLinks,
) -> (Vec<StructureMove>, ConveyorMarkDiag) {
    world.sync_rotator_arrivals();
    structures.clear_turn_marks();
    let mut movers: Vec<(u64, IVec3, MovementRule)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            // 仅运动设备收集规则，避免场景等方块走 dyn movement_rule
            let rule = match block.kind {
                BlockKind::Conveyor => {
                    world
                        .is_occupied(*pos + IVec3::Y)
                        .then_some(MovementRule::Translate {
                            source: IVec3::Y,
                            offset: block.facing.forward_ivec3(),
                        })
                }
                BlockKind::ReverseConveyor => {
                    world
                        .is_occupied(*pos + IVec3::NEG_Y)
                        .then_some(MovementRule::Translate {
                            source: IVec3::NEG_Y,
                            offset: -block.facing.forward_ivec3(),
                        })
                }
                BlockKind::Lifter
                | BlockKind::Rotator
                | BlockKind::CounterRotator
                | BlockKind::Pusher
                | BlockKind::Blocker => block.kind.movement_rule(block.facing),
                _ => None,
            }?;
            Some((block.id.0, *pos, rule))
        })
        .collect();
    // 按 BlockId 稳定裁决（与 held 冲突规则一致）
    movers.sort_by_key(|(id, _, _)| *id);
    // 同结构同位移只需一个代表传送带尝试
    let mut seen_forward: HashSet<(StructureId, IVec3)> = HashSet::new();
    let mut seen_reverse: HashSet<(StructureId, IVec3)> = HashSet::new();
    let mut unique_movers = Vec::with_capacity(movers.len());
    for (_id, pos, mover) in movers {
        match mover {
            MovementRule::Translate { source, offset } => {
                let target = pos + source;
                let same_structure = structures
                    .id_at(target)
                    .is_some_and(|_| structures.structure_contains(target, pos));
                let keep = if same_structure {
                    structures
                        .id_at(pos)
                        .is_some_and(|sid| seen_reverse.insert((sid, -offset)))
                } else if let Some(tid) = structures.id_at(target) {
                    let forward_new = seen_forward.insert((tid, offset));
                    let reverse_new = structures
                        .id_at(pos)
                        .is_some_and(|sid| seen_reverse.insert((sid, -offset)));
                    forward_new || reverse_new
                } else if world.is_occupied(target)
                    || PusherState::body_at_extended_head(world, target).is_some()
                {
                    true
                } else {
                    false
                };
                if keep {
                    unique_movers.push((pos, mover));
                }
            }
            _ => unique_movers.push((pos, mover)),
        }
    }
    let movers = unique_movers;
    let mut moves = Vec::new();
    let mut claimed_heads = PusherState::hard_head_occupancy(world);
    // 同结构同位移：can_translate 每回合只算一次；同向标记去重
    let mut translate_ok: HashMap<(StructureId, IVec3), bool> = HashMap::new();
    let mut emitted_translate: HashSet<(StructureId, IVec3)> = HashSet::new();
    // 抬升：同一结构本回合只打一条标签（避免上千抬升器重复克隆同一坨）
    let mut lifted_structures: HashSet<StructureId> = HashSet::new();
    let mut conveyor_diag = ConveyorMarkDiag::default();

    // 本回合要切换伸出状态的推杆（排序继承 movers）
    let mut actuating: Vec<(IVec3, IVec3, IVec3, bool)> = Vec::new();
    for (pos, mover) in &movers {
        let MovementRule::PoweredTranslate {
            source,
            offset,
            extend_when_powered,
        } = mover
        else {
            continue;
        };
        let powered = powered_devices.contains(pos);
        let desired_extended = if *extend_when_powered {
            powered
        } else {
            !powered
        };
        let current_extended = world
            .blocks
            .get(pos)
            .and_then(|block| pusher_state.entries.get(&block.id))
            .map(|entry| entry.extended)
            .unwrap_or(false);
        if desired_extended != current_extended {
            actuating.push((*pos, *source, *offset, desired_extended));
        }
    }
    let mut motion_held: HashSet<IVec3> = HashSet::new();
    let mut motion_tags: HashMap<IVec3, IVec3> = HashMap::new();
    let mut succeeded_deform: HashSet<(StructureId, u32)> = HashSet::new();
    // 本回合欲伸出 / 欲收回的推杆体（共轴组校验用）
    let mut actuating_extend: HashSet<BlockId> = HashSet::new();
    let mut actuating_retract: HashSet<BlockId> = HashSet::new();
    for (pos, _, _, desired_extended) in &actuating {
        let Some(id) = world.blocks.get(pos).map(|b| b.id) else {
            continue;
        };
        if *desired_extended {
            actuating_extend.insert(id);
        } else {
            actuating_retract.insert(id);
        }
    }
    // 先收回再伸出，使对向「伸+收」同向共用中间格时伸出能看到头已逻辑释放
    actuating.sort_by_key(|(pos, _, _, desired)| {
        let id = world.blocks.get(pos).map(|b| b.id.0).unwrap_or(u64::MAX);
        (*desired, id)
    });

    for (pos, mover) in movers {
        let source_id = world.blocks.get(&pos).map(|block| block.id);
        match mover {
            MovementRule::Translate { source, offset } => {
                conveyor_diag.attempts += 1;
                if let Some(movement) = mark_conveyor_movement(
                    world,
                    structures,
                    pos,
                    source,
                    offset,
                    suction,
                    &claimed_heads,
                    &mut translate_ok,
                    &emitted_translate,
                    &mut conveyor_diag,
                ) {
                    let key = (movement.structure_id(), translate_offset(&movement));
                    if emitted_translate.insert(key) {
                        conveyor_diag.emitted += 1;
                        if let Some(source_id) = source_id {
                            moves.push(movement.with_source(source_id, pos));
                        }
                    } else {
                        conveyor_diag.deduped += 1;
                    }
                }
            }
            MovementRule::Lift { range } => {
                if powered_devices.contains(&pos) {
                    continue;
                }
                for movement in mark_lift_structures(
                    world,
                    structures,
                    pos,
                    range,
                    suction,
                    &mut lifted_structures,
                ) {
                    if let Some(source_id) = source_id {
                        moves.push(movement.with_source(source_id, pos));
                    }
                }
            }
            MovementRule::Rotate { clockwise } => {
                if let Some(movement) = mark_rotate_structure(
                    world,
                    powered_devices,
                    structures,
                    pos,
                    clockwise,
                    suction,
                ) {
                    if let Some(source_id) = source_id {
                        moves.push(movement.with_source(source_id, pos));
                    }
                }
            }
            MovementRule::PoweredTranslate { .. } => {}
        }
    }

    for (pos, source, offset, desired_extended) in actuating {
        let mut ctx = SimMovementMarkCtx {
            world,
            structures,
            suction,
            claimed_heads: &mut claimed_heads,
            motion_held: &mut motion_held,
            motion_tags: &mut motion_tags,
            succeeded_deform: &mut succeeded_deform,
            actuating_extend: &actuating_extend,
            actuating_retract: &actuating_retract,
        };
        if let Some(movement) = mark_pusher_movement(
            &mut ctx,
            pusher_state,
            pos,
            source,
            offset,
            desired_extended,
        ) {
            let merged = match &movement {
                StructureMove::Translate {
                    offset: move_off,
                    actors,
                    ..
                } if *move_off == IVec3::ZERO && actors.len() == 1 => {
                    let actor = actors[0];
                    let mut found = false;
                    for prior in moves.iter_mut().rev() {
                        if let StructureMove::Translate {
                            structure,
                            offset: prior_off,
                            actors: prior_actors,
                            ..
                        } = prior
                        {
                            if *prior_off != IVec3::ZERO && structure.contains(&actor.pos) {
                                prior_actors.push(actor);
                                found = true;
                                break;
                            }
                        }
                    }
                    found
                }
                _ => false,
            };
            if !merged {
                moves.push(movement);
            }
        }
    }
    (moves, conveyor_diag)
}

/// 传送带标记诊断：回答「单次 can_translate 有多贵」
#[derive(Default, Clone, Debug)]
pub(super) struct ConveyorMarkDiag {
    pub attempts: u32,
    pub cache_hits: u32,
    pub can_translate_calls: u32,
    pub can_translate_ms: f64,
    pub emitted: u32,
    pub deduped: u32,
}

fn translate_offset(movement: &StructureMove) -> IVec3 {
    match movement {
        StructureMove::Translate { offset, .. } => *offset,
        StructureMove::Rotate { .. } => IVec3::ZERO,
    }
}

fn cached_can_translate(
    world: &WorldBlocks,
    structures: &StructureState,
    suction: &SuctionLinks,
    hard_pusher_heads: &HashSet<IVec3>,
    structure_id: StructureId,
    structure: &HashSet<IVec3>,
    offset: IVec3,
    cache: &mut HashMap<(StructureId, IVec3), bool>,
    diag: &mut ConveyorMarkDiag,
) -> bool {
    if let Some(&allowed) = cache.get(&(structure_id, offset)) {
        diag.cache_hits += 1;
        return allowed;
    }
    let started = std::time::Instant::now();
    let allowed = can_translate_structure(
        world,
        structure,
        offset,
        structures,
        suction,
        hard_pusher_heads,
    );
    diag.can_translate_calls += 1;
    diag.can_translate_ms += started.elapsed().as_secs_f64() * 1000.0;
    cache.insert((structure_id, offset), allowed);
    allowed
}

fn mark_conveyor_movement(
    world: &WorldBlocks,
    structures: &StructureState,
    pos: IVec3,
    source: IVec3,
    offset: IVec3,
    suction: &SuctionLinks,
    hard_pusher_heads: &HashSet<IVec3>,
    translate_ok: &mut HashMap<(StructureId, IVec3), bool>,
    emitted_translate: &HashSet<(StructureId, IVec3)>,
    diag: &mut ConveyorMarkDiag,
) -> Option<StructureMove> {
    let target = pos + source;
    let forward_seed = if structures.id_at(target).is_some() {
        Some(target)
    } else {
        PusherState::body_at_extended_head(world, target)
    };

    // 正向推货：先看 structure_id 缓存，避免上千次克隆整坨位置
    if let Some(seed) = forward_seed {
        if !structures.structure_contains(seed, pos) {
            if let Some(structure_id) = structures.id_at(seed) {
                let key = (structure_id, offset);
                if !emitted_translate.contains(&key) {
                    match translate_ok.get(&key).copied() {
                        Some(true) => {
                            return mark_structure_translate(
                                world,
                                structures,
                                pos,
                                target,
                                offset,
                                MovementMark::Conveyor,
                                suction,
                            );
                        }
                        Some(false) => {}
                        None => {
                            if let Some(movement) = mark_structure_translate(
                                world,
                                structures,
                                pos,
                                target,
                                offset,
                                MovementMark::Conveyor,
                                suction,
                            ) {
                                if cached_can_translate(
                                    world,
                                    structures,
                                    suction,
                                    hard_pusher_heads,
                                    movement.structure_id(),
                                    movement.structure(),
                                    offset,
                                    translate_ok,
                                    diag,
                                ) {
                                    return Some(movement);
                                }
                            }
                        }
                    }
                }
            }
        }
    } else if !world.is_occupied(target)
        && PusherState::body_at_extended_head(world, target).is_none()
    {
        return None;
    }

    // 反向：传送带自己被顶着走；同样先查缓存再克隆
    let structure_id = structures.id_at(pos)?;
    let reverse = -offset;
    let key = (structure_id, reverse);
    if emitted_translate.contains(&key) {
        return None;
    }
    match translate_ok.get(&key).copied() {
        Some(false) => return None,
        Some(true) => {
            let structure = structures.linked_pushable_at(suction, pos, reverse)?;
            return Some(StructureMove::translate_marked(
                structure_id,
                structure,
                reverse,
                MovementMark::Conveyor,
            ));
        }
        None => {
            let structure = structures.linked_pushable_at(suction, pos, reverse)?;
            if !cached_can_translate(
                world,
                structures,
                suction,
                hard_pusher_heads,
                structure_id,
                &structure,
                reverse,
                translate_ok,
                diag,
            ) {
                return None;
            }
            Some(StructureMove::translate_marked(
                structure_id,
                structure,
                reverse,
                MovementMark::Conveyor,
            ))
        }
    }
}

fn mark_pusher_movement(
    ctx: &mut SimMovementMarkCtx<'_>,
    pusher_state: &mut PusherState,
    pos: IVec3,
    source: IVec3,
    offset: IVec3,
    desired_extended: bool,
) -> Option<StructureMove> {
    let id = ctx.world.blocks.get(&pos)?.id;
    let (current_extended, bound_front) = {
        let entry = pusher_state
            .entries
            .entry(id)
            .or_insert_with(|| PusherStateEntry {
                extended: false,
                bound_front: false,
            });
        (entry.extended, entry.bound_front)
    };
    if desired_extended == current_extended {
        return None;
    }
    let animation = if desired_extended {
        PusherAnimationKind::Extend
    } else {
        PusherAnimationKind::Retract
    };

    let head = pos + source;
    let structure_id = ctx.structures.id_at(pos)?;
    let lazy_anchored = ctx.structures.get(structure_id).is_some_and(|structure| {
        structure.activity == FactoryActivity::Inactive && structure.head_of.is_empty()
    });

    // 体已 held：不可再发动其它 deform；仅当某动作组已成功时挂共轴动画
    if ctx.structures.held_blocks.contains(&id) || ctx.motion_held.contains(&pos) {
        if desired_extended {
            for forward in [true, false] {
                let Some((_, indices)) =
                    ctx.structures.deform_action_groups(ctx.world, pos, forward)
                else {
                    continue;
                };
                if indices
                    .iter()
                    .any(|idx| ctx.succeeded_deform.contains(&(structure_id, *idx)))
                {
                    return try_deform_action(
                        ctx,
                        pos,
                        id,
                        structure_id,
                        forward,
                        if forward { offset } else { -offset },
                        animation,
                        forward,
                    );
                }
            }
        }
        return None;
    }

    if desired_extended {
        if lazy_anchored
            && (ctx.world.is_fragile_material_at(head) || !ctx.world.is_occupied(head))
        {
            if !ctx.claimed_heads.insert(head) {
                return None;
            }
            return Some(
                StructureMove::translate_by_pusher_actor(
                    structure_id,
                    HashSet::from([pos]),
                    IVec3::ZERO,
                    PusherActor { id, pos, animation },
                    MovementMark::Push,
                )
                .with_source(id, pos),
            );
        }
        if let Some(movement) =
            try_deform_action(ctx, pos, id, structure_id, true, offset, animation, true)
        {
            return Some(movement);
        }
        // 正推无实体格且头前是外结构：整坨外推
        let forward_physical_empty = ctx
            .structures
            .deform_action(ctx.world, pos, true)
            .is_some_and(|(seed, _, nodes)| {
                ctx.structures
                    .nodes_to_positions(ctx.world, seed, nodes)
                    .is_empty()
            })
            || lazy_anchored;
        if forward_physical_empty && !ctx.world.is_fragile_material_at(head) {
            if let Some(front_id) = ctx.world.blocks.get(&head).map(|b| b.id) {
                let external = ctx
                    .structures
                    .id_at(head)
                    .is_some_and(|sid| sid != structure_id)
                    || PusherState::body_at_extended_head(ctx.world, head)
                        .is_some_and(|body| ctx.structures.id_at(body) != Some(structure_id));
                if external && !ctx.structures.held_blocks.contains(&front_id) {
                    let cargo_pos =
                        PusherState::body_at_extended_head(ctx.world, head).unwrap_or(head);
                    if let Some(movement) = mark_structure_translate(
                        ctx.world,
                        ctx.structures,
                        pos,
                        cargo_pos,
                        offset,
                        MovementMark::Push,
                        ctx.suction,
                    ) {
                        let mut heads = ctx.claimed_heads.clone();
                        heads.remove(&head);
                        if can_translate_structure(
                            ctx.world,
                            movement.structure(),
                            offset,
                            ctx.structures,
                            ctx.suction,
                            &heads,
                        ) {
                            ctx.claimed_heads.insert(head);
                            apply_motion_tags(
                                movement.structure(),
                                offset,
                                ctx.motion_held,
                                ctx.motion_tags,
                            );
                            for &p in movement.structure() {
                                if let Some(b) = ctx.world.blocks.get(&p) {
                                    ctx.structures.held_blocks.insert(b.id);
                                }
                            }
                            if let Some((_, _, nodes)) =
                                ctx.structures.deform_action(ctx.world, pos, true)
                            {
                                let node_ids: Vec<_> = nodes.to_vec();
                                ctx.structures.held_blocks.extend(node_ids);
                            }
                            return Some(
                                movement
                                    .with_pusher_actor(id, pos, MovementMark::Push, animation)
                                    .with_source(id, pos),
                            );
                        }
                    }
                }
            }
        }
        if lazy_anchored {
            return None;
        }
        if PUSHER_REVERSE_ENABLED {
            // 正推失败后反推自身；仍走 Extend：到位后进入伸出并停住（避免每回合再退）
            return try_deform_action(ctx, pos, id, structure_id, false, -offset, animation, false);
        }
        return None;
    }

    // 收回：先释放头占格，粘头则拉回正推节点集
    ctx.claimed_heads.remove(&head);
    if bound_front {
        if let Some(movement) =
            try_deform_action(ctx, pos, id, structure_id, true, -offset, animation, false)
        {
            return Some(movement);
        }
    }
    Some(
        StructureMove::translate_by_pusher_actor(
            structure_id,
            HashSet::from([pos]),
            IVec3::ZERO,
            PusherActor { id, pos, animation },
            MovementMark::Push,
        )
        .with_source(id, pos),
    )
}

/// 一次 deform：按节点数升序试候选；共轴已成功则只挂动画；节点撞 held 则试下一条
fn try_deform_action(
    ctx: &mut SimMovementMarkCtx<'_>,
    pos: IVec3,
    id: BlockId,
    structure_id: StructureId,
    forward: bool,
    move_offset: IVec3,
    animation: PusherAnimationKind,
    claim_head: bool,
) -> Option<StructureMove> {
    let group_indices: Vec<u32> = {
        let (_, indices) = ctx
            .structures
            .deform_action_groups(ctx.world, pos, forward)?;
        indices.to_vec()
    };

    for group_idx in group_indices {
        if ctx.succeeded_deform.contains(&(structure_id, group_idx)) {
            if claim_head {
                let head = pos
                    + ctx
                        .world
                        .blocks
                        .get(&pos)
                        .map(|b| b.facing.forward_ivec3())
                        .unwrap_or(IVec3::ZERO);
                // 对向杆本回合收回：头格虽仍在世界中，但将同向腾出，允许伸入
                let head_vacating = PusherState::body_at_extended_head(ctx.world, head)
                    .and_then(|body| ctx.world.blocks.get(&body).map(|b| b.id))
                    .is_some_and(|bid| ctx.actuating_retract.contains(&bid));
                if !ctx.world.is_fragile_material_at(head)
                    && ctx.world.is_occupied(head)
                    && !ctx.motion_held.contains(&head)
                    && !head_vacating
                {
                    continue;
                }
                if !head_vacating && !ctx.claimed_heads.insert(head) {
                    return None;
                }
                if head_vacating {
                    ctx.claimed_heads.insert(head);
                }
            }
            return Some(
                StructureMove::translate_by_pusher_actor(
                    structure_id,
                    HashSet::from([pos]),
                    IVec3::ZERO,
                    PusherActor { id, pos, animation },
                    MovementMark::Push,
                )
                .with_source(id, pos),
            );
        }

        let nodes = {
            let structure = ctx.structures.get(structure_id)?;
            let group = structure.deform_groups.get(group_idx as usize)?;
            // 共轴组：actions 里列出的同伴本回合必须同样在伸/缩（与是否在 nodes 无关）。
            // 同组已成功后的其余 actor 走上方快速路径，不再重复扫描整组同伴。
            let peers_ready = group.actions.iter().all(|(body, action_fwd)| {
                if *body == id || *action_fwd != forward {
                    return true;
                }
                if forward {
                    ctx.actuating_extend.contains(body)
                } else {
                    ctx.actuating_retract.contains(body)
                }
            });
            if !peers_ready {
                continue;
            }
            group.nodes.clone()
        };

        if nodes.iter().any(|n| ctx.structures.held_blocks.contains(n)) {
            continue;
        }

        let (subset, anchored) = {
            let structure = ctx.structures.get(structure_id)?;
            let subset = ctx
                .structures
                .nodes_to_positions(ctx.world, structure, &nodes);
            let anchored = structure.is_scene_anchored_subset(&subset);
            (subset, anchored)
        };
        if anchored || subset.iter().any(|p| ctx.motion_held.contains(p)) {
            continue;
        }

        let head = pos
            + ctx
                .world
                .blocks
                .get(&pos)
                .map(|b| b.facing.forward_ivec3())
                .unwrap_or(IVec3::ZERO);

        if subset.is_empty() {
            if move_offset != IVec3::ZERO && !claim_head {
                continue;
            }
            if claim_head {
                let head_vacating = PusherState::body_at_extended_head(ctx.world, head)
                    .and_then(|body| ctx.world.blocks.get(&body).map(|b| b.id))
                    .is_some_and(|bid| ctx.actuating_retract.contains(&bid));
                if !ctx.world.is_fragile_material_at(head)
                    && ctx.world.is_occupied(head)
                    && !head_vacating
                {
                    continue;
                }
                if !head_vacating && !ctx.claimed_heads.insert(head) {
                    return None;
                }
                if head_vacating {
                    ctx.claimed_heads.insert(head);
                }
            }
            ctx.structures.held_blocks.extend(nodes.iter().copied());
            ctx.succeeded_deform.insert((structure_id, group_idx));
            return Some(
                StructureMove::translate_by_pusher_actor(
                    structure_id,
                    HashSet::from([pos]),
                    IVec3::ZERO,
                    PusherActor { id, pos, animation },
                    MovementMark::Push,
                )
                .with_source(id, pos),
            );
        }

        let Some(expanded) =
            ctx.structures
                .linked_expand_pusher_subset(ctx.suction, &subset, move_offset)
        else {
            continue;
        };
        let mut heads_for_check = ctx.claimed_heads.clone();
        if let Some(body) = PusherState::body_at_extended_head(ctx.world, head) {
            if expanded.contains(&body) {
                heads_for_check.remove(&head);
            }
        }
        if claim_head && expanded.contains(&head) {
            heads_for_check.remove(&head);
        }
        if !claim_head {
            heads_for_check.remove(&head);
        }
        if !can_translate_structure(
            ctx.world,
            &expanded,
            move_offset,
            ctx.structures,
            ctx.suction,
            &heads_for_check,
        ) {
            continue;
        }
        if claim_head {
            if let Some(body) = PusherState::body_at_extended_head(ctx.world, head) {
                if expanded.contains(&body) {
                    ctx.claimed_heads.remove(&head);
                }
            }
            if !ctx.claimed_heads.insert(head) {
                return None;
            }
        }

        apply_motion_tags(&expanded, move_offset, ctx.motion_held, ctx.motion_tags);
        ctx.structures.held_blocks.extend(nodes.iter().copied());
        for &p in &expanded {
            if let Some(b) = ctx.world.blocks.get(&p) {
                ctx.structures.held_blocks.insert(b.id);
            }
        }
        ctx.succeeded_deform.insert((structure_id, group_idx));
        ctx.structures.moving_structures.insert(structure_id);

        return Some(
            StructureMove::translate_by_pusher_actor(
                structure_id,
                expanded,
                move_offset,
                PusherActor { id, pos, animation },
                MovementMark::Push,
            )
            .with_source(id, pos),
        );
    }
    None
}

fn apply_motion_tags(
    structure: &HashSet<IVec3>,
    offset: IVec3,
    motion_held: &mut HashSet<IVec3>,
    motion_tags: &mut HashMap<IVec3, IVec3>,
) {
    for &pos in structure {
        motion_held.insert(pos);
        motion_tags.insert(pos, offset);
    }
}

trait StructureMoveActorExt {
    fn with_pusher_actor(
        self,
        actor_id: BlockId,
        actor: IVec3,
        mark: MovementMark,
        animation: PusherAnimationKind,
    ) -> StructureMove;
}

impl StructureMoveActorExt for StructureMove {
    fn with_pusher_actor(
        self,
        actor_id: BlockId,
        actor: IVec3,
        mark: MovementMark,
        animation: PusherAnimationKind,
    ) -> StructureMove {
        match self {
            StructureMove::Translate {
                structure_id,
                structure,
                offset,
                mut actors,
                source,
                source_pos,
                ..
            } => {
                actors.push(PusherActor {
                    id: actor_id,
                    pos: actor,
                    animation,
                });
                StructureMove::Translate {
                    structure_id,
                    structure,
                    offset,
                    actors,
                    mark,
                    source: None,
                    source_pos: None,
                }
                .with_optional_source(source, source_pos)
            }
            movement => movement,
        }
    }
}

trait StructureMoveSourceExt {
    fn with_optional_source(
        self,
        source: Option<crate::blocks::BlockId>,
        source_pos: Option<IVec3>,
    ) -> StructureMove;
}

impl StructureMoveSourceExt for StructureMove {
    fn with_optional_source(
        self,
        source: Option<crate::blocks::BlockId>,
        source_pos: Option<IVec3>,
    ) -> StructureMove {
        match (source, source_pos) {
            (Some(source), Some(source_pos)) => self.with_source(source, source_pos),
            _ => self,
        }
    }
}

fn mark_structure_translate(
    world: &WorldBlocks,
    structures: &StructureState,
    actor: IVec3,
    mut source: IVec3,
    offset: IVec3,
    mark: MovementMark,
    suction: &SuctionLinks,
) -> Option<StructureMove> {
    // 推到已伸出的头：视为推动该推杆整坨（头+体占两格）
    if structures.id_at(source).is_none() {
        source = PusherState::body_at_extended_head(world, source)?;
    }
    if world.is_material_at(source) {
        let structure_id = structures.id_at(source)?;
        return structures
            .linked_pushable_at(suction, source, offset)
            .map(|structure| {
                StructureMove::translate_marked(structure_id, structure, offset, mark)
            });
    }

    let structure_id = structures.id_at(source)?;
    let structure = if matches!(mark, MovementMark::Push)
        && world.blocks.get(&actor).is_some_and(|block| {
            matches!(
                block.kind.movement_rule(block.facing),
                Some(MovementRule::PoweredTranslate { .. })
            )
        }) {
        // 活塞子集后再经吸盘扩展（子集不膨胀为整结构）
        let subset = structures.pusher_target_structure(world, actor, source, offset)?;
        structures.linked_expand_pusher_subset(suction, &subset, offset)?
    } else {
        if structures.structure_contains(source, actor) {
            return None;
        }
        structures.linked_pushable_at(suction, source, offset)?
    };
    Some(StructureMove::translate_marked(
        structure_id,
        structure,
        offset,
        mark,
    ))
}

/// 抬升器 range 内每个活动结构各打一条抬升标签（跨抬升器去重，避免同结构被打成百上千遍）
fn mark_lift_structures(
    world: &WorldBlocks,
    structures: &StructureState,
    pos: IVec3,
    range: i32,
    suction: &SuctionLinks,
    lifted_structures: &mut HashSet<StructureId>,
) -> Vec<StructureMove> {
    let mut moves = Vec::new();
    for height in 1..=range {
        let candidate = pos + IVec3::Y * height;
        let seed = structures
            .id_at(candidate)
            .map(|_| candidate)
            .or_else(|| PusherState::body_at_extended_head(world, candidate));
        let Some(seed) = seed else {
            continue;
        };
        let Some(id) = structures.id_at(seed) else {
            continue;
        };
        if lifted_structures.contains(&id) {
            continue;
        }
        // 抬升器自身所在结构不抬
        if structures
            .get(id)
            .is_some_and(|structure| structure.positions.contains(&pos))
        {
            continue;
        }
        // 吸盘分量内须全是 Active 且可上移（工厂 / 材料均可）
        let component = suction.component_ids(structures, [id]);
        let mut structure = HashSet::new();
        let mut liftable = !component.is_empty();
        for cid in &component {
            if lifted_structures.contains(cid) {
                liftable = false;
                break;
            }
            let Some(meta) = structures.get(*cid) else {
                liftable = false;
                break;
            };
            if meta.activity != FactoryActivity::Active || !meta.freedom.can_translate(IVec3::Y) {
                liftable = false;
                break;
            }
            structure.extend(meta.positions.iter().copied());
        }
        if !liftable || structure.is_empty() || structure.contains(&pos) {
            continue;
        }
        // 不在标记期用 can_translate 过滤：抬不动也要打标签，merge 才能压住重力，
        // 否则被挡住时下落→再抬→上下弹（65ff7b1）。执行阶段推不动则原地不动。
        for cid in &component {
            lifted_structures.insert(*cid);
        }
        moves.push(StructureMove::translate_marked(
            id,
            structure,
            IVec3::Y,
            MovementMark::Vertical,
        ));
    }
    moves
}

fn mark_rotate_structure(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    structures: &StructureState,
    pos: IVec3,
    clockwise: bool,
    suction: &SuctionLinks,
) -> Option<StructureMove> {
    // 通电清锁，同拍可再转工作面上同一块
    if powered_devices.contains(&pos) {
        world.rotator_arrivals.remove(&pos);
    }
    let source = pos + IVec3::Y;
    let block = world.blocks.get(&source)?;
    if !(block.kind.is_material() || block.kind.is_factory()) {
        return None;
    }
    if world.is_rotator_arrival(pos, block.id) {
        return None;
    }
    let structure_id = structures.id_at(source)?;
    // 材料被吸盘粘到工厂时不转；纯工厂结构可以转
    if structures.kind_at(source) == Some(StructureKind::Material)
        && structures.linked_contains_factory(suction, source)
    {
        return None;
    }
    let structure = structures.linked_pushable_at(suction, source, IVec3::ZERO)?;
    Some(StructureMove::rotate(
        structure_id,
        structure,
        pos,
        clockwise,
    ))
}
