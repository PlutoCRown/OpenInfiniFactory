fn bpos_facing_toward(body: IVec3, source: IVec3, target: IVec3) -> bool {
    body + source == target
}

/// 材料 id → 坐标（连通洪水复用，避免每个种子重建）
fn material_id_to_pos(world: &WorldBlocks) -> HashMap<BlockId, IVec3> {
    world
        .blocks
        .iter()
        .filter(|(_, block)| block.kind.is_material() && !block.id.is_none())
        .map(|(pos, block)| (block.id, *pos))
        .collect()
}

/// 材料焊接邻接表（同轮多次洪水复用，避免逐节点扫描全部焊缝）
fn material_weld_neighbors(world: &WorldBlocks) -> HashMap<BlockId, Vec<BlockId>> {
    let mut neighbors: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
    for weld in &world.material_welds {
        neighbors.entry(weld.a).or_default().push(weld.b);
        neighbors.entry(weld.b).or_default().push(weld.a);
    }
    neighbors
}

/// 材料焊接连通（即时）
pub fn material_structure(world: &WorldBlocks, start: IVec3) -> HashSet<IVec3> {
    let id_to_pos = material_id_to_pos(world);
    let weld_neighbors = material_weld_neighbors(world);
    material_structure_from(world, start, &id_to_pos, &weld_neighbors)
}

fn material_structure_from(
    world: &WorldBlocks,
    start: IVec3,
    id_to_pos: &HashMap<BlockId, IVec3>,
    weld_neighbors: &HashMap<BlockId, Vec<BlockId>>,
) -> HashSet<IVec3> {
    let Some(start_id) = world
        .blocks
        .get(&start)
        .filter(|block| block.kind.is_material() && !block.id.is_none())
        .map(|block| block.id)
    else {
        return HashSet::new();
    };

    let mut structure = HashSet::new();
    let mut seen_ids = HashSet::from([start_id]);
    let mut queue = VecDeque::from([start_id]);
    structure.insert(start);

    while let Some(id) = queue.pop_front() {
        for &other_id in weld_neighbors.get(&id).into_iter().flatten() {
            if !seen_ids.insert(other_id) {
                continue;
            }
            let Some(&neighbor) = id_to_pos.get(&other_id) else {
                continue;
            };
            if !world.is_material_at(neighbor) {
                continue;
            }
            structure.insert(neighbor);
            queue.push_back(other_id);
        }
    }

    structure
}

pub fn query_factory_structure(world: &WorldBlocks, pos: IVec3) -> Option<HashSet<IVec3>> {
    world
        .is_factory_at(pos)
        .then(|| factory_structure(world, pos))
}

fn collect_gravity_support(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> Vec<GravitySupportContact> {
    structure
        .iter()
        .filter_map(|pos| {
            let below = *pos + IVec3::NEG_Y;
            (below.y >= 0
                && !structure.contains(&below)
                && (!world.can_move_into_yielding_fragile(below)
                    || hard_pusher_head_occupancy.contains(&below)))
            .then_some((*pos, IVec3::NEG_Y))
        })
        .collect()
}

fn factory_structure(world: &WorldBlocks, start: IVec3) -> HashSet<IVec3> {
    let allowed: HashSet<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| block.kind.is_factory().then_some(*pos))
        .collect();
    connected_factory_subset(world, &allowed, start, None)
}

/// 在 allowed 内工厂连通（可穿过真实活塞头）
fn connected_factory_subset(
    world: &WorldBlocks,
    allowed: &HashSet<IVec3>,
    start: IVec3,
    blocked_pusher_pos: Option<IVec3>,
) -> HashSet<IVec3> {
    let mut structure = HashSet::new();
    let mut queue = VecDeque::from([start]);
    structure.insert(start);

    while let Some(pos) = queue.pop_front() {
        for offset in signal_offsets() {
            let neighbor = pos + offset;
            if structure.contains(&neighbor) {
                continue;
            }
            if world
                .blocks
                .get(&neighbor)
                .is_some_and(|block| block.kind == BlockKind::PusherHead)
            {
                if is_blocked_pusher_edge(world, blocked_pusher_pos, pos, neighbor)
                    || is_blocked_factory_connection(world, pos, neighbor)
                    || is_blocked_factory_connection(world, neighbor, pos)
                {
                    continue;
                }
                for offset2 in signal_offsets() {
                    let beyond = neighbor + offset2;
                    if structure.contains(&beyond) || !allowed.contains(&beyond) {
                        continue;
                    }
                    if is_blocked_factory_connection(world, beyond, neighbor)
                        || is_blocked_factory_connection(world, neighbor, beyond)
                    {
                        continue;
                    }
                    structure.insert(beyond);
                    queue.push_back(beyond);
                }
                continue;
            }
            if !allowed.contains(&neighbor)
                || is_blocked_pusher_edge(world, blocked_pusher_pos, pos, neighbor)
                || is_blocked_factory_connection(world, pos, neighbor)
                || is_blocked_factory_connection(world, neighbor, pos)
            {
                continue;
            }
            structure.insert(neighbor);
            queue.push_back(neighbor);
        }
    }

    structure
}

fn is_blocked_pusher_edge(
    world: &WorldBlocks,
    pusher_pos: Option<IVec3>,
    from: IVec3,
    to: IVec3,
) -> bool {
    let Some(pusher_pos) = pusher_pos else {
        return false;
    };
    world.blocks.get(&pusher_pos).is_some_and(|block| {
        matches!(
            block.kind.movement_rule(block.facing),
            Some(MovementRule::PoweredTranslate { .. })
        ) && {
            let front = pusher_pos + block.facing.forward_ivec3();
            (from == pusher_pos && to == front) || (from == front && to == pusher_pos)
        }
    })
}

fn is_blocked_factory_connection(world: &WorldBlocks, from: IVec3, to: IVec3) -> bool {
    world
        .blocks
        .get(&from)
        .is_some_and(|block| block.kind.non_connection_face(block.facing) == Some(to - from))
}

/// 子集是否贴着场景格
pub fn touches_scene(world: &WorldBlocks, structure: &HashSet<IVec3>) -> bool {
    structure.iter().any(|pos| {
        signal_offsets().into_iter().any(|offset| {
            let neighbor = *pos + offset;
            world.is_scene_at(neighbor) && !is_blocked_factory_connection(world, *pos, neighbor)
        })
    })
}
