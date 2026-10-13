use super::{
    BlockId, BlockKind, GravitySupportContact, HashMap, HashSet, IVec3, VecDeque, WorldBlocks,
    signal_offsets,
};

pub(super) fn bpos_facing_toward(body: IVec3, source: IVec3, target: IVec3) -> bool {
    body + source == target
}

/// 材料 id → 坐标（连通洪水复用，避免每个种子重建）
pub(super) fn material_id_to_pos(world: &WorldBlocks) -> HashMap<BlockId, IVec3> {
    world
        .blocks
        .iter()
        .filter(|(_, block)| block.kind.is_material() && !block.id.is_none())
        .map(|(pos, block)| (block.id, *pos))
        .collect()
}

/// 材料焊接邻接表（同轮多次洪水复用，避免逐节点扫描全部焊缝）
pub(super) fn material_weld_neighbors(world: &WorldBlocks) -> HashMap<BlockId, Vec<BlockId>> {
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

pub(super) fn material_structure_from(
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

pub(super) fn collect_gravity_support(
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

/// 收集工厂连通成员，真实活塞头可作为连接桥
pub(super) fn factory_structure(world: &WorldBlocks, start: IVec3) -> HashSet<IVec3> {
    let allowed: HashSet<IVec3> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| block.kind.is_factory().then_some(*pos))
        .collect();
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
                if !factory_faces_connect(world, pos, neighbor) {
                    continue;
                }
                for offset2 in signal_offsets() {
                    let beyond = neighbor + offset2;
                    if structure.contains(&beyond) || !allowed.contains(&beyond) {
                        continue;
                    }
                    if !factory_faces_connect(world, neighbor, beyond) {
                        continue;
                    }
                    structure.insert(beyond);
                    queue.push_back(beyond);
                }
                continue;
            }
            if !allowed.contains(&neighbor) || !factory_faces_connect(world, pos, neighbor) {
                continue;
            }
            structure.insert(neighbor);
            queue.push_back(neighbor);
        }
    }

    structure
}

/// 工厂与场景的连接要求相邻两面均可连接，活塞头沿用连通桥语义
pub(super) fn factory_faces_connect(world: &WorldBlocks, from: IVec3, to: IVec3) -> bool {
    let Some(a) = world.blocks.get(&from) else {
        return false;
    };
    let Some(b) = world.blocks.get(&to) else {
        return false;
    };
    let normal = to - from;
    normal.abs().element_sum() == 1
        && (a.kind == BlockKind::PusherHead || a.kind.face_attachable(a.facing, normal))
        && (b.kind == BlockKind::PusherHead || b.kind.face_attachable(b.facing, -normal))
}
