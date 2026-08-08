pub(super) fn mark_structure_movement_phase(
    world: &mut WorldBlocks,
    powered_devices: &HashSet<IVec3>,
    structures: &mut StructureState,
    pusher_state: &mut PusherState,
    suction: &SuctionLinks,
) -> Vec<StructureMove> {
    world.sync_rotator_arrivals();
    structures.clear_turn_marks();
    let mut movers: Vec<(IVec3, MovementRule)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            block
                .kind
                .movement_rule(block.facing)
                .map(|mover| (*pos, mover))
        })
        .collect();
    // 按 BlockId 稳定裁决（与 held 冲突规则一致）
    movers.sort_by_key(|(pos, _)| {
        world
            .blocks
            .get(pos)
            .map(|block| block.id.0)
            .unwrap_or(u64::MAX)
    });
    let mut moves = Vec::new();
    let mut claimed_heads = PusherState::hard_head_occupancy(world);

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

    for (pos, mover) in movers {
        let source_id = world.blocks.get(&pos).map(|block| block.id);
        match mover {
            MovementRule::Translate { source, offset } => {
                if let Some(movement) =
                    mark_conveyor_movement(world, structures, pos, source, offset, suction)
                {
                    if let Some(source_id) = source_id {
                        moves.push(movement.with_source(source_id, pos));
                    }
                }
            }
            MovementRule::Lift { range } => {
                if powered_devices.contains(&pos) {
                    continue;
                }
                for movement in mark_lift_structures(world, structures, pos, range, suction) {
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
        if let Some(movement) = mark_pusher_movement(
            world,
            structures,
            pusher_state,
            pos,
            source,
            offset,
            desired_extended,
            &mut claimed_heads,
            suction,
            &mut motion_held,
            &mut motion_tags,
            &mut succeeded_deform,
            &actuating_extend,
            &actuating_retract,
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
    moves
}

fn mark_conveyor_movement(
    world: &WorldBlocks,
    structures: &StructureState,
    pos: IVec3,
    source: IVec3,
    offset: IVec3,
    suction: &SuctionLinks,
) -> Option<StructureMove> {
    let heads = PusherState::hard_head_occupancy(world);
    let target = pos + source;
    if let Some(movement) = mark_structure_translate(
        world,
        structures,
        pos,
        target,
        offset,
        MovementMark::Conveyor,
        suction,
    ) {
        if can_translate_structure(
            world,
            movement.structure(),
            offset,
            structures,
            suction,
            &heads,
        ) {
            return Some(movement);
        }
    } else if !world.is_occupied(target)
        && PusherState::body_at_extended_head(world, target).is_none()
    {
        return None;
    }

    let structure = structures.linked_pushable_at(suction, pos, -offset)?;
    if !can_translate_structure(world, &structure, -offset, structures, suction, &heads) {
        return None;
    }
    Some(StructureMove::translate_marked(
        structures.id_at(pos)?,
        structure,
        -offset,
        MovementMark::Conveyor,
    ))
}

fn mark_pusher_movement(
    world: &WorldBlocks,
    structures: &mut StructureState,
    pusher_state: &mut PusherState,
    pos: IVec3,
    source: IVec3,
    offset: IVec3,
    desired_extended: bool,
    claimed_heads: &mut HashSet<IVec3>,
    suction: &SuctionLinks,
    motion_held: &mut HashSet<IVec3>,
    motion_tags: &mut HashMap<IVec3, IVec3>,
    succeeded_deform: &mut HashSet<(StructureId, u32)>,
    actuating_extend: &HashSet<BlockId>,
    actuating_retract: &HashSet<BlockId>,
) -> Option<StructureMove> {
    let id = world.blocks.get(&pos)?.id;
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
    let structure_id = structures.id_at(pos)?;

    // 体已 held：不可再发动其它 deform；仅当某动作组已成功时挂共轴动画
    if structures.held_blocks.contains(&id) || motion_held.contains(&pos) {
        if desired_extended {
            for forward in [true, false] {
                let Some((_, indices)) = structures.deform_action_groups(world, pos, forward)
                else {
                    continue;
                };
                if indices
                    .iter()
                    .any(|idx| succeeded_deform.contains(&(structure_id, *idx)))
                {
                    return try_deform_action(
                        world,
                        structures,
                        suction,
                        claimed_heads,
                        pos,
                        id,
                        structure_id,
                        forward,
                        if forward { offset } else { -offset },
                        animation,
                        forward,
                        motion_held,
                        motion_tags,
                        succeeded_deform,
                        actuating_extend,
                        actuating_retract,
                    );
                }
            }
        }
        return None;
    }

    if desired_extended {
        if let Some(movement) = try_deform_action(
            world,
            structures,
            suction,
            claimed_heads,
            pos,
            id,
            structure_id,
            true,
            offset,
            animation,
            true,
            motion_held,
            motion_tags,
            succeeded_deform,
            actuating_extend,
            actuating_retract,
        ) {
            return Some(movement);
        }
        // 正推无实体格且头前是外结构：整坨外推
        let forward_physical_empty =
            structures
                .deform_action(world, pos, true)
                .is_some_and(|(seed, _, nodes)| {
                    structures.nodes_to_positions(world, seed, nodes).is_empty()
                });
        if forward_physical_empty && !world.is_fragile_material_at(head) {
            if let Some(front_id) = world.blocks.get(&head).map(|b| b.id) {
                let external = structures
                    .id_at(head)
                    .is_some_and(|sid| sid != structure_id)
                    || PusherState::body_at_extended_head(world, head)
                        .is_some_and(|body| structures.id_at(body) != Some(structure_id));
                if external && !structures.held_blocks.contains(&front_id) {
                    let cargo_pos = PusherState::body_at_extended_head(world, head).unwrap_or(head);
                    if let Some(movement) = mark_structure_translate(
                        world,
                        structures,
                        pos,
                        cargo_pos,
                        offset,
                        MovementMark::Push,
                        suction,
                    ) {
                        let mut heads = claimed_heads.clone();
                        heads.remove(&head);
                        if can_translate_structure(
                            world,
                            movement.structure(),
                            offset,
                            structures,
                            suction,
                            &heads,
                        ) {
                            claimed_heads.insert(head);
                            apply_motion_tags(
                                movement.structure(),
                                offset,
                                motion_held,
                                motion_tags,
                            );
                            for &p in movement.structure() {
                                if let Some(b) = world.blocks.get(&p) {
                                    structures.held_blocks.insert(b.id);
                                }
                            }
                            if let Some((_, _, nodes)) = structures.deform_action(world, pos, true)
                            {
                                let node_ids: Vec<_> = nodes.to_vec();
                                structures.held_blocks.extend(node_ids);
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
        if PUSHER_REVERSE_ENABLED {
            // 正推失败后反推自身；仍走 Extend：到位后进入伸出并停住（避免每回合再退）
            return try_deform_action(
                world,
                structures,
                suction,
                claimed_heads,
                pos,
                id,
                structure_id,
                false,
                -offset,
                animation,
                false,
                motion_held,
                motion_tags,
                succeeded_deform,
                actuating_extend,
                actuating_retract,
            );
        }
        return None;
    }

    // 收回：先释放头占格，粘头则拉回正推节点集
    claimed_heads.remove(&head);
    if bound_front {
        if let Some(movement) = try_deform_action(
            world,
            structures,
            suction,
            claimed_heads,
            pos,
            id,
            structure_id,
            true,
            -offset,
            animation,
            false,
            motion_held,
            motion_tags,
            succeeded_deform,
            actuating_extend,
            actuating_retract,
        ) {
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
    world: &WorldBlocks,
    structures: &mut StructureState,
    suction: &SuctionLinks,
    claimed_heads: &mut HashSet<IVec3>,
    pos: IVec3,
    id: BlockId,
    structure_id: StructureId,
    forward: bool,
    move_offset: IVec3,
    animation: PusherAnimationKind,
    claim_head: bool,
    motion_held: &mut HashSet<IVec3>,
    motion_tags: &mut HashMap<IVec3, IVec3>,
    succeeded_deform: &mut HashSet<(StructureId, u32)>,
    actuating_extend: &HashSet<BlockId>,
    actuating_retract: &HashSet<BlockId>,
) -> Option<StructureMove> {
    let group_indices: Vec<u32> = {
        let (_, indices) = structures.deform_action_groups(world, pos, forward)?;
        indices.to_vec()
    };

    for group_idx in group_indices {
        let (nodes, actions) = {
            let structure = structures.get(structure_id)?;
            let group = structure.deform_groups.get(group_idx as usize)?;
            (group.nodes.clone(), group.actions.clone())
        };

        // 共轴组：actions 里列出的同伴本回合必须同样在伸/缩（与是否在 nodes 无关）
        let peers_ready = actions.iter().all(|(body, action_fwd)| {
            if *body == id || *action_fwd != forward {
                return true;
            }
            if forward {
                actuating_extend.contains(body)
            } else {
                actuating_retract.contains(body)
            }
        });
        if !peers_ready {
            continue;
        }

        if succeeded_deform.contains(&(structure_id, group_idx)) {
            if claim_head {
                let head = pos
                    + world
                        .blocks
                        .get(&pos)
                        .map(|b| b.facing.forward_ivec3())
                        .unwrap_or(IVec3::ZERO);
                // 世界尚未提交位移：头格上的货物可能已在本回合 motion_held
                if !world.is_fragile_material_at(head)
                    && world.is_occupied(head)
                    && !motion_held.contains(&head)
                {
                    continue;
                }
                if !claimed_heads.insert(head) {
                    return None;
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

        if nodes.iter().any(|n| structures.held_blocks.contains(n)) {
            continue;
        }

        let (subset, anchored) = {
            let structure = structures.get(structure_id)?;
            let subset = structures.nodes_to_positions(world, structure, &nodes);
            let anchored = structure.is_scene_anchored_subset(&subset);
            (subset, anchored)
        };
        if anchored || subset.iter().any(|p| motion_held.contains(p)) {
            continue;
        }

        let head = pos
            + world
                .blocks
                .get(&pos)
                .map(|b| b.facing.forward_ivec3())
                .unwrap_or(IVec3::ZERO);

        if subset.is_empty() {
            if move_offset != IVec3::ZERO && !claim_head {
                continue;
            }
            if claim_head {
                if !world.is_fragile_material_at(head) && world.is_occupied(head) {
                    continue;
                }
                if !claimed_heads.insert(head) {
                    return None;
                }
            }
            structures.held_blocks.extend(nodes.iter().copied());
            succeeded_deform.insert((structure_id, group_idx));
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

        let Some(expanded) = structures.linked_expand_pusher_subset(suction, &subset, move_offset)
        else {
            continue;
        };
        let mut heads_for_check = claimed_heads.clone();
        if let Some(body) = PusherState::body_at_extended_head(world, head) {
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
            world,
            &expanded,
            move_offset,
            structures,
            suction,
            &heads_for_check,
        ) {
            continue;
        }
        if claim_head {
            if let Some(body) = PusherState::body_at_extended_head(world, head) {
                if expanded.contains(&body) {
                    claimed_heads.remove(&head);
                }
            }
            if !claimed_heads.insert(head) {
                return None;
            }
        }

        apply_motion_tags(&expanded, move_offset, motion_held, motion_tags);
        structures.held_blocks.extend(nodes.iter().copied());
        for &p in &expanded {
            if let Some(b) = world.blocks.get(&p) {
                structures.held_blocks.insert(b.id);
            }
        }
        succeeded_deform.insert((structure_id, group_idx));
        structures.moving_structures.insert(structure_id);

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

/// 抬升器 range 内每个可动结构各打一条抬升标签（不并成一条，避免只抬底层）
fn mark_lift_structures(
    world: &WorldBlocks,
    structures: &StructureState,
    pos: IVec3,
    range: i32,
    suction: &SuctionLinks,
) -> Vec<StructureMove> {
    let mut moves = Vec::new();
    let mut seen_ids = HashSet::new();
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
        if !seen_ids.insert(id) {
            continue;
        }
        let eligible = world.is_material_at(seed)
            || structures
                .linked_pushable_at(suction, seed, IVec3::Y)
                .is_some();
        if !eligible {
            seen_ids.remove(&id);
            continue;
        }
        let Some(movement) = mark_structure_translate(
            world,
            structures,
            pos,
            seed,
            IVec3::Y,
            MovementMark::Vertical,
            suction,
        ) else {
            seen_ids.remove(&id);
            continue;
        };
        // 不在标记期用 can_translate 过滤：抬不动也要打标签，merge 才能压住重力，
        // 否则被挡住时下落→再抬→上下弹（65ff7b1）。执行阶段推不动则原地不动。
        for member in movement.structure() {
            if let Some(member_id) = structures.id_at(*member) {
                seen_ids.insert(member_id);
            }
        }
        moves.push(movement);
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
