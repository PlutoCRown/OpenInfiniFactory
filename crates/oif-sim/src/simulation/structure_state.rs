use glam::IVec3;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::blocks::{AcceptorId, BlockId, BlockKind, MovementRule};
use crate::world::grid::WorldBlocks;

use super::signal_offsets;

/// 结构运行时 ID：开局/焊接重建时分配，成员移动时保持不变
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct StructureId(pub u64);

impl StructureId {
    pub const NONE: Self = Self(0);

    pub const fn is_none(self) -> bool {
        self.0 == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactoryActivity {
    Active,
    Inactive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructureFreedom {
    None,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructureKind {
    Material,
    Factory,
}

impl StructureFreedom {
    pub fn can_translate(self, _offset: IVec3) -> bool {
        self == Self::All
    }
}

/// 重力支撑接触：成员格 → 支撑方向
pub type GravitySupportContact = (IVec3, IVec3);

/// 共轴可变形子集：同节点集的正/反推动作必须一起伸缩
#[derive(Clone, Debug, Default)]
pub struct DeformGroup {
    pub actions: Vec<(BlockId, bool)>,
    pub nodes: Vec<BlockId>,
}

/// 单杆正/反推对应的格点两侧（由 DeformGroup 解析）
#[derive(Clone, Debug)]
pub struct DeformSides {
    pub separated: bool,
    pub actor_side: HashSet<IVec3>,
    pub target_side: HashSet<IVec3>,
    pub actor_anchored: bool,
    pub target_anchored: bool,
}

/// 单个连通结构（工厂或材料）
#[derive(Clone)]
pub struct Structure {
    pub id: StructureId,
    pub kind: StructureKind,
    pub positions: HashSet<IVec3>,
    pub activity: FactoryActivity,
    pub freedom: StructureFreedom,
    gravity_support: Vec<GravitySupportContact>,
    /// 推杆体 → 逻辑头 BlockId（放置/重建时分配，伸出前后稳定）
    pub head_of: HashMap<BlockId, BlockId>,
    /// 逻辑头 → 推杆体
    body_of_head: HashMap<BlockId, BlockId>,
    pub deform_groups: Vec<DeformGroup>,
    /// (体, 正推?) → 候选组下标，按 nodes.len() 升序（优先多杆共轴、少节点）
    action_to_groups: HashMap<(BlockId, bool), Vec<u32>>,
    /// 开局/编辑时贴场景的成员格（锚死快照，中途落地不改）
    scene_touching: HashSet<IVec3>,
}

impl Structure {
    /// 是否可被推动：无贴场景成员
    pub fn is_pushable(&self) -> bool {
        self.scene_touching.is_empty()
    }

    /// 子集是否含贴场景成员（锚死不可推）
    pub fn is_scene_anchored_subset(&self, positions: &HashSet<IVec3>) -> bool {
        positions.iter().any(|p| self.scene_touching.contains(p))
    }
}

/// 验收口结构运行时计数
#[derive(Clone, Debug)]
pub struct AcceptorStructure {
    pub id: AcceptorId,
    pub positions: HashSet<IVec3>,
    pub count: u32,
}

/// 世界结构表：工厂/材料连通、可变形子集、回合 held
#[derive(Default, Clone)]
pub struct StructureState {
    structures: HashMap<StructureId, Structure>,
    structure_by_pos: HashMap<IVec3, StructureId>,
    next_structure_id: u64,
    next_head_id: u64,
    acceptor_structures: Vec<AcceptorStructure>,
    /// 本回合已占用移动的方块（含逻辑头）；每回合清空
    pub held_blocks: HashSet<BlockId>,
    /// 本回合已占用整段平移的结构
    pub moving_structures: HashSet<StructureId>,
}

impl StructureState {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn clear_turn_marks(&mut self) {
        self.held_blocks.clear();
        self.moving_structures.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.structure_by_pos.is_empty()
    }

    fn alloc_id(&mut self) -> StructureId {
        self.next_structure_id += 1;
        StructureId(self.next_structure_id)
    }

    fn alloc_head_id(&mut self) -> BlockId {
        self.next_head_id = self.next_head_id.max(1);
        let id = BlockId(self.next_head_id);
        self.next_head_id += 1;
        id
    }

    /// 逻辑头 ID 不得与世界方块 ID 冲突
    fn sync_head_counter(&mut self, world: &WorldBlocks) {
        self.next_head_id = self.next_head_id.max(world.next_block_id).max(1);
    }

    /// 全量重建：工厂连通 + inactive + deform + 验收口 + 材料
    pub fn rebuild_for_simulation(&mut self, world: &WorldBlocks) {
        let next_head = self.next_head_id.max(world.next_block_id).max(1);
        *self = Self::default();
        self.next_head_id = next_head;
        self.append_factory_structures(world);
        self.apply_factory_inactive_propagation(world);
        self.rebuild_all_factory_deform(world);
        self.append_acceptor_structures(world);
        self.append_material_structures(world, &HashMap::new(), &HashMap::new());
    }

    /// 开局轻量：已有工厂缓存则只刷新材料/验收口；否则全量重建
    pub fn refresh_for_simulation_start(&mut self, world: &WorldBlocks) {
        self.clear_turn_marks();
        if !self
            .structures
            .values()
            .any(|structure| structure.kind == StructureKind::Factory)
        {
            self.rebuild_for_simulation(world);
            return;
        }
        self.acceptor_structures.clear();
        self.append_acceptor_structures(world);
        self.refresh_material_structures(world);
    }

    pub fn acceptor_structures(&self) -> &[AcceptorStructure] {
        &self.acceptor_structures
    }

    pub fn increment_acceptor_count(&mut self, index: usize) {
        if let Some(structure) = self.acceptor_structures.get_mut(index) {
            structure.count = structure.count.saturating_add(1);
        }
    }

    /// 调试：仅重建工厂连通与 deform
    pub fn rebuild_factory_for_debug(&mut self, world: &WorldBlocks) {
        self.retain_factory_only();
        self.append_factory_structures(world);
        self.apply_factory_inactive_propagation(world);
        self.rebuild_all_factory_deform(world);
    }

    pub fn refresh_material_structures(&mut self, world: &WorldBlocks) {
        let mut previous_ids: HashMap<Vec<u64>, StructureId> = HashMap::new();
        let mut previous_support: HashMap<StructureId, Vec<GravitySupportContact>> = HashMap::new();
        for (id, structure) in &self.structures {
            if structure.kind != StructureKind::Material {
                continue;
            }
            let mut members: Vec<u64> = structure
                .positions
                .iter()
                .filter_map(|pos| world.blocks.get(pos).map(|block| block.id.0))
                .collect();
            members.sort_unstable();
            previous_ids.insert(members, *id);
            previous_support.insert(*id, structure.gravity_support.clone());
        }
        self.retain_factory_only();
        self.append_material_structures(world, &previous_ids, &previous_support);
    }

    fn retain_factory_only(&mut self) {
        self.structures
            .retain(|_, structure| structure.kind == StructureKind::Factory);
        self.structure_by_pos.clear();
        for (id, structure) in &self.structures {
            for pos in &structure.positions {
                self.structure_by_pos.insert(*pos, *id);
            }
        }
    }

    fn append_factory_structures(&mut self, world: &WorldBlocks) {
        let starts: Vec<IVec3> = world
            .blocks
            .iter()
            .filter_map(|(pos, block)| block.kind.is_factory().then_some(*pos))
            .collect();
        self.append_connected_factory_structures(world, starts);
    }

    fn append_acceptor_structures(&mut self, world: &WorldBlocks) {
        for stored in &world.acceptor_structures {
            self.acceptor_structures.push(AcceptorStructure {
                id: stored.id,
                positions: stored.positions.iter().copied().collect(),
                count: 0,
            });
        }
    }

    fn append_material_structures(
        &mut self,
        world: &WorldBlocks,
        previous_ids: &HashMap<Vec<u64>, StructureId>,
        previous_support: &HashMap<StructureId, Vec<GravitySupportContact>>,
    ) {
        let mut handled = self
            .structure_by_pos
            .keys()
            .copied()
            .collect::<HashSet<_>>();
        let mut starts: Vec<IVec3> = world
            .blocks
            .iter()
            .filter_map(|(pos, block)| block.kind.is_material().then_some(*pos))
            .collect();
        starts.sort_by_key(|pos| (pos.x, pos.y, pos.z));

        for start in starts {
            if handled.contains(&start) || !world.is_material_at(start) {
                continue;
            }
            let positions = material_structure(world, start);
            let mut members: Vec<u64> = positions
                .iter()
                .filter_map(|pos| world.blocks.get(pos).map(|block| block.id.0))
                .collect();
            members.sort_unstable();
            let id = previous_ids
                .get(&members)
                .copied()
                .unwrap_or_else(|| self.alloc_id());
            let gravity_support = previous_support.get(&id).cloned().unwrap_or_default();
            for pos in &positions {
                handled.insert(*pos);
                self.structure_by_pos.insert(*pos, id);
            }
            self.structures.insert(
                id,
                Structure {
                    id,
                    kind: StructureKind::Material,
                    positions,
                    activity: FactoryActivity::Active,
                    freedom: StructureFreedom::All,
                    gravity_support,
                    head_of: HashMap::new(),
                    body_of_head: HashMap::new(),
                    deform_groups: Vec::new(),
                    action_to_groups: HashMap::new(),
                    scene_touching: HashSet::new(),
                },
            );
        }
    }

    fn append_connected_factory_structures(
        &mut self,
        world: &WorldBlocks,
        starts: impl IntoIterator<Item = IVec3>,
    ) {
        let mut handled: HashSet<IVec3> = self.structure_by_pos.keys().copied().collect();
        let mut starts: Vec<IVec3> = starts
            .into_iter()
            .filter(|pos| world.is_factory_at(*pos) && !handled.contains(pos))
            .collect();
        starts.sort_by_key(|pos| (pos.x, pos.y, pos.z));

        for start in starts {
            if handled.contains(&start) || !world.is_factory_at(start) {
                continue;
            }

            let positions = factory_structure(world, start);
            let id = self.alloc_id();
            for pos in &positions {
                handled.insert(*pos);
                self.structure_by_pos.insert(*pos, id);
            }
            self.structures.insert(
                id,
                Structure {
                    id,
                    kind: StructureKind::Factory,
                    positions,
                    activity: FactoryActivity::Active,
                    freedom: StructureFreedom::All,
                    gravity_support: Vec::new(),
                    head_of: HashMap::new(),
                    body_of_head: HashMap::new(),
                    deform_groups: Vec::new(),
                    action_to_groups: HashMap::new(),
                    scene_touching: HashSet::new(),
                },
            );
        }
    }

    fn apply_factory_inactive_propagation(&mut self, world: &WorldBlocks) {
        let factory_ids: Vec<StructureId> = self
            .structures
            .iter()
            .filter(|(_, structure)| structure.kind == StructureKind::Factory)
            .map(|(id, _)| *id)
            .collect();
        let scene_anchored: HashMap<StructureId, bool> = factory_ids
            .iter()
            .map(|id| {
                let anchored = self
                    .structures
                    .get(id)
                    .is_some_and(|structure| touches_scene(world, &structure.positions));
                (*id, anchored)
            })
            .collect();
        let mut inactive: HashMap<StructureId, bool> = scene_anchored.clone();
        let mut queue = VecDeque::new();
        for (id, anchored) in &scene_anchored {
            if *anchored {
                inactive.insert(*id, true);
                queue.push_back(*id);
            }
        }

        while let Some(id) = queue.pop_front() {
            let Some(structure) = self.structures.get(&id) else {
                continue;
            };
            if structure.kind != StructureKind::Factory {
                continue;
            }
            for pos in structure.positions.clone() {
                for offset in signal_offsets() {
                    let neighbor = pos + offset;
                    let Some(neighbor_id) = self.structure_by_pos.get(&neighbor).copied() else {
                        continue;
                    };
                    let Some(neighbor_structure) = self.structures.get(&neighbor_id) else {
                        continue;
                    };
                    if neighbor_structure.kind != StructureKind::Factory {
                        continue;
                    }
                    if is_blocked_factory_connection(world, pos, neighbor)
                        || is_blocked_factory_connection(world, neighbor, pos)
                    {
                        continue;
                    }
                    if !inactive.get(&neighbor_id).copied().unwrap_or(false) {
                        inactive.insert(neighbor_id, true);
                        queue.push_back(neighbor_id);
                    }
                }
            }
        }

        for id in factory_ids {
            let Some(structure) = self.structures.get_mut(&id) else {
                continue;
            };
            structure.activity = FactoryActivity::Active;
            structure.freedom = StructureFreedom::All;
            if inactive.get(&id).copied().unwrap_or(false) {
                structure.activity = FactoryActivity::Inactive;
                if scene_anchored.get(&id).copied().unwrap_or(false) {
                    structure.freedom = StructureFreedom::None;
                }
            }
        }
    }

    /// 编辑变更：局部合并/拆分工厂连通并重算受影响 deform
    pub fn apply_factory_edit(&mut self, world: &WorldBlocks, changed: &HashSet<IVec3>) {
        if changed.is_empty() {
            return;
        }

        let mut seed_positions: HashSet<IVec3> = HashSet::new();
        let mut stale_ids: HashSet<StructureId> = HashSet::new();

        for &pos in changed {
            if let Some(id) = self.structure_by_pos.get(&pos).copied() {
                if let Some(structure) = self.structures.get(&id) {
                    if structure.kind == StructureKind::Factory {
                        stale_ids.insert(id);
                        seed_positions.extend(structure.positions.iter().copied());
                    }
                }
            }
            for offset in signal_offsets() {
                let neighbor = pos + offset;
                if let Some(id) = self.structure_by_pos.get(&neighbor).copied() {
                    if let Some(structure) = self.structures.get(&id) {
                        if structure.kind == StructureKind::Factory {
                            stale_ids.insert(id);
                            seed_positions.extend(structure.positions.iter().copied());
                        }
                    }
                }
            }
            if world.is_factory_at(pos) {
                seed_positions.insert(pos);
            }
        }

        for id in &stale_ids {
            if let Some(structure) = self.structures.remove(id) {
                for pos in &structure.positions {
                    self.structure_by_pos.remove(pos);
                }
            }
        }

        let mut rebuild_seeds: Vec<IVec3> = seed_positions
            .into_iter()
            .filter(|pos| world.is_factory_at(*pos) && !self.structure_by_pos.contains_key(pos))
            .collect();
        rebuild_seeds.sort_by_key(|pos| (pos.x, pos.y, pos.z));

        let before_ids: HashSet<StructureId> = self.structures.keys().copied().collect();
        self.append_connected_factory_structures(world, rebuild_seeds);
        let new_ids: Vec<StructureId> = self
            .structures
            .keys()
            .copied()
            .filter(|id| !before_ids.contains(id))
            .collect();

        self.apply_factory_inactive_propagation(world);
        for id in new_ids {
            self.rebuild_deform_for(world, id);
        }
        // 邻接 inactive 可能波及旧结构的 freedom；仅刷新仍存在的旧工厂 deform 的 scene 相关不重算图
        // inactive 传播已写回 activity/freedom；deform 节点集不依赖 inactive
        self.refresh_material_structures(world);
    }

    fn rebuild_all_factory_deform(&mut self, world: &WorldBlocks) {
        let ids: Vec<StructureId> = self
            .structures
            .iter()
            .filter(|(_, s)| s.kind == StructureKind::Factory)
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            self.rebuild_deform_for(world, id);
        }
    }

    /// 为单个工厂结构分配逻辑头并构建共轴 DeformGroup
    fn rebuild_deform_for(&mut self, world: &WorldBlocks, id: StructureId) {
        self.sync_head_counter(world);
        let Some(structure) = self.structures.get(&id) else {
            return;
        };
        if structure.kind != StructureKind::Factory {
            return;
        }
        let positions = structure.positions.clone();
        let mut prev_heads = structure.head_of.clone();

        let scene_touching: HashSet<IVec3> = positions
            .iter()
            .copied()
            .filter(|pos| {
                signal_offsets().into_iter().any(|offset| {
                    let neighbor = *pos + offset;
                    world.is_scene_at(neighbor)
                        && !is_blocked_factory_connection(world, *pos, neighbor)
                })
            })
            .collect();

        // body → head；复用旧 head id
        let mut head_of: HashMap<BlockId, BlockId> = HashMap::new();
        let mut body_of_head: HashMap<BlockId, BlockId> = HashMap::new();
        let mut body_pos: HashMap<BlockId, IVec3> = HashMap::new();
        let mut facing_of: HashMap<BlockId, IVec3> = HashMap::new();

        for &pos in &positions {
            let Some(block) = world.blocks.get(&pos) else {
                continue;
            };
            let Some(MovementRule::PoweredTranslate { source, .. }) =
                block.kind.movement_rule(block.facing)
            else {
                continue;
            };
            let head_id = prev_heads
                .remove(&block.id)
                .unwrap_or_else(|| self.alloc_head_id());
            head_of.insert(block.id, head_id);
            body_of_head.insert(head_id, block.id);
            body_pos.insert(block.id, pos);
            facing_of.insert(block.id, source);
        }

        // 无向邻接 + 有向 body→head
        let mut undirected: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();
        let link_u = |adj: &mut HashMap<BlockId, HashSet<BlockId>>, a: BlockId, b: BlockId| {
            adj.entry(a).or_default().insert(b);
            adj.entry(b).or_default().insert(a);
        };

        let id_at: HashMap<IVec3, BlockId> = positions
            .iter()
            .filter_map(|pos| world.blocks.get(pos).map(|b| (*pos, b.id)))
            .collect();

        for (&body, &head) in &head_of {
            let Some(&bpos) = body_pos.get(&body) else {
                continue;
            };
            let Some(&forward) = facing_of.get(&body) else {
                continue;
            };
            // 体—头仅有向；头接到正面（贴脸对向则头—头，避免绕回本体）
            let mut front = bpos + forward;
            while world
                .blocks
                .get(&front)
                .is_some_and(|b| b.kind == BlockKind::PusherHead)
            {
                front += forward;
            }
            if let Some(front_block) = world.blocks.get(&front) {
                let face_to_face = matches!(
                    front_block.kind.movement_rule(front_block.facing),
                    Some(MovementRule::PoweredTranslate { source, .. })
                        if bpos_facing_toward(front, source, bpos)
                );
                if face_to_face {
                    if let Some(&other_head) = head_of.get(&front_block.id) {
                        link_u(&mut undirected, head, other_head);
                    }
                } else if let Some(&front_id) = id_at.get(&front) {
                    link_u(&mut undirected, head, front_id);
                }
            }
        }

        for &pos in &positions {
            let Some(&block_id) = id_at.get(&pos) else {
                continue;
            };
            let Some(block) = world.blocks.get(&pos) else {
                continue;
            };
            let forward_opt = matches!(
                block.kind.movement_rule(block.facing),
                Some(MovementRule::PoweredTranslate { .. })
            )
            .then(|| block.facing.forward_ivec3());

            for offset in signal_offsets() {
                if forward_opt == Some(offset) {
                    continue; // 正面只经头连
                }
                let neighbor = pos + offset;
                let Some(&neighbor_id) = id_at.get(&neighbor) else {
                    continue;
                };
                if is_blocked_factory_connection(world, pos, neighbor)
                    || is_blocked_factory_connection(world, neighbor, pos)
                {
                    continue;
                }
                // 邻块若是朝向本格的推杆，已有「邻体→头→本格」，不再加体—体边
                let neighbor_faces_here = world.blocks.get(&neighbor).is_some_and(|b| {
                    matches!(
                        b.kind.movement_rule(b.facing),
                        Some(MovementRule::PoweredTranslate { source, .. })
                            if bpos_facing_toward(neighbor, source, pos)
                    )
                });
                if neighbor_faces_here {
                    continue;
                }
                link_u(&mut undirected, block_id, neighbor_id);
            }
        }

        // 推杆轴向：东西 / 南北 / 上下（同轴才共组；横切枚举也只对同轴同伴）
        let axis_of = |fwd: IVec3| -> u8 {
            if fwd.x != 0 {
                0
            } else if fwd.z != 0 {
                1
            } else {
                2
            }
        };

        // 在给定切断集下传播；碰到 forbidden=成环。有向边仅当其体未切断时可走。
        let propagate_with_cuts = |start: BlockId,
                                   forbidden: BlockId,
                                   cuts: &HashSet<BlockId>,
                                   undirected_only: bool|
         -> Option<Vec<BlockId>> {
            let mut seen = HashSet::from([start]);
            let mut queue = VecDeque::from([start]);
            while let Some(node) = queue.pop_front() {
                if let Some(neighbors) = undirected.get(&node) {
                    for &next in neighbors {
                        if next == forbidden {
                            return None;
                        }
                        if seen.insert(next) {
                            queue.push_back(next);
                        }
                    }
                }
                if undirected_only {
                    continue;
                }
                // 体→头
                if let Some(&head_n) = head_of.get(&node) {
                    if !cuts.contains(&node) {
                        if head_n == forbidden {
                            return None;
                        }
                        if seen.insert(head_n) {
                            queue.push_back(head_n);
                        }
                    }
                }
                // 头→体
                if let Some(&body_n) = body_of_head.get(&node) {
                    if head_of.get(&body_n) == Some(&node) && !cuts.contains(&body_n) {
                        if body_n == forbidden {
                            return None;
                        }
                        if seen.insert(body_n) {
                            queue.push_back(body_n);
                        }
                    }
                }
            }
            let mut nodes: Vec<_> = seen.into_iter().collect();
            nodes.sort_by_key(|id| id.0);
            Some(nodes)
        };

        // (nodes_key, move_dir) → group；同节点且同推动方向才共组
        let mut groups_by_key: HashMap<(Vec<u64>, IVec3), DeformGroup> = HashMap::new();

        let mut bodies: Vec<BlockId> = head_of.keys().copied().collect();
        bodies.sort_by_key(|id| id.0);

        for &body in &bodies {
            let Some(&head) = head_of.get(&body) else {
                continue;
            };
            let Some(&b_fwd) = facing_of.get(&body) else {
                continue;
            };
            let axis = axis_of(b_fwd);

            // 同轴其它推杆：枚举「也切断」子集 → 每条成功传播都是该动作的候选
            let mut peers: Vec<BlockId> = bodies
                .iter()
                .copied()
                .filter(|p| *p != body && facing_of.get(p).is_some_and(|f| axis_of(*f) == axis))
                .collect();
            peers.sort_by_key(|id| id.0);
            if peers.len() > 12 {
                peers.truncate(12);
            }

            let mut candidate_sets: Vec<(bool, Vec<BlockId>)> = Vec::new();
            let peer_n = peers.len();
            let masks = 1u32 << peer_n;
            for mask in 0..masks {
                let mut cuts = HashSet::from([body]);
                for (i, peer) in peers.iter().enumerate() {
                    if mask & (1 << i) != 0 {
                        cuts.insert(*peer);
                    }
                }
                if let Some(nodes) = propagate_with_cuts(head, body, &cuts, false) {
                    if !nodes.is_empty() {
                        candidate_sets.push((true, nodes));
                    }
                }
                if let Some(nodes) = propagate_with_cuts(body, head, &cuts, false) {
                    if !nodes.is_empty() {
                        candidate_sets.push((false, nodes));
                    }
                }
            }

            // 全部同轴单切都成环 → 全切仅无向（对向环）
            let any_fwd = candidate_sets.iter().any(|(f, _)| *f);
            let any_rev = candidate_sets.iter().any(|(f, _)| !*f);
            if !any_fwd || !any_rev {
                let all_cuts: HashSet<BlockId> = head_of.keys().copied().collect();
                if !any_fwd {
                    if let Some(nodes) = propagate_with_cuts(head, body, &all_cuts, true) {
                        if !nodes.is_empty() {
                            candidate_sets.push((true, nodes));
                        }
                    }
                }
                if !any_rev {
                    if let Some(nodes) = propagate_with_cuts(body, head, &all_cuts, true) {
                        if !nodes.is_empty() {
                            candidate_sets.push((false, nodes));
                        }
                    }
                }
            }

            // 正反仍缺一侧则整杆丢弃
            let any_fwd = candidate_sets.iter().any(|(f, _)| *f);
            let any_rev = candidate_sets.iter().any(|(f, _)| !*f);
            if !any_fwd || !any_rev {
                continue;
            }

            for (forward, nodes) in candidate_sets {
                let move_dir = if forward { b_fwd } else { -b_fwd };
                let key = (nodes.iter().map(|id| id.0).collect::<Vec<_>>(), move_dir);
                if let Some(group) = groups_by_key.get_mut(&key) {
                    if !group.actions.contains(&(body, forward)) {
                        group.actions.push((body, forward));
                    }
                } else {
                    groups_by_key.insert(
                        key,
                        DeformGroup {
                            actions: vec![(body, forward)],
                            nodes,
                        },
                    );
                }
            }
        }

        let mut keys: Vec<_> = groups_by_key.keys().cloned().collect();
        keys.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.x.cmp(&b.1.x))
                .then_with(|| a.1.y.cmp(&b.1.y))
                .then_with(|| a.1.z.cmp(&b.1.z))
        });
        let mut deform_groups: Vec<DeformGroup> = Vec::new();
        let mut action_to_groups: HashMap<(BlockId, bool), Vec<u32>> = HashMap::new();
        for key in keys {
            let Some(mut group) = groups_by_key.remove(&key) else {
                continue;
            };
            group.actions.sort_by_key(|(id, fwd)| (id.0, !*fwd));
            let index = deform_groups.len() as u32;
            for action in &group.actions {
                action_to_groups.entry(*action).or_default().push(index);
            }
            deform_groups.push(group);
        }
        for indices in action_to_groups.values_mut() {
            indices.sort_by_key(|idx| deform_groups[*idx as usize].nodes.len());
            indices.dedup();
        }

        if let Some(structure) = self.structures.get_mut(&id) {
            structure.head_of = head_of;
            structure.body_of_head = body_of_head;
            structure.deform_groups = deform_groups;
            structure.action_to_groups = action_to_groups;
            structure.scene_touching = scene_touching;
        }
    }

    pub fn structure_ids(&self) -> impl Iterator<Item = StructureId> + '_ {
        self.structures.keys().copied()
    }

    pub fn activity_at(&self, pos: IVec3) -> Option<FactoryActivity> {
        Some(self.structure(pos)?.activity)
    }

    pub fn id_at(&self, pos: IVec3) -> Option<StructureId> {
        self.structure_by_pos.get(&pos).copied()
    }

    pub fn pushable_structure_at(&self, pos: IVec3, offset: IVec3) -> Option<HashSet<IVec3>> {
        let structure = self.structure(pos)?;
        if !structure.is_pushable() || !structure.freedom.can_translate(offset) {
            return None;
        }
        Some(structure.positions.clone())
    }

    pub fn active_structure_at(&self, pos: IVec3, offset: IVec3) -> Option<HashSet<IVec3>> {
        self.pushable_structure_at(pos, offset)
    }

    /// 不同结构正推：整坨；同结构变形由 deform_sides 负责
    pub fn pusher_target_structure(
        &self,
        _world: &WorldBlocks,
        pusher_pos: IVec3,
        target_pos: IVec3,
        offset: IVec3,
    ) -> Option<HashSet<IVec3>> {
        let target = self.structure(target_pos)?;
        if target.kind != StructureKind::Factory {
            return None;
        }
        let actor_id = self.id_at(pusher_pos)?;
        if actor_id != target.id {
            if !target.is_pushable() || !target.freedom.can_translate(offset) {
                return None;
            }
            return Some(target.positions.clone());
        }
        None
    }

    /// 查询活塞某方向 deform 候选（按节点数升序）；返回结构与组下标列表
    pub fn deform_action_groups(
        &self,
        world: &WorldBlocks,
        pusher_pos: IVec3,
        forward: bool,
    ) -> Option<(&Structure, &[u32])> {
        let seed = self.structure(pusher_pos)?;
        if seed.kind != StructureKind::Factory {
            return None;
        }
        let block = world.blocks.get(&pusher_pos)?;
        let indices = seed.action_to_groups.get(&(block.id, forward))?;
        if indices.is_empty() {
            return None;
        }
        Some((seed, indices.as_slice()))
    }

    /// 首选（节点最少）deform 动作
    pub fn deform_action(
        &self,
        world: &WorldBlocks,
        pusher_pos: IVec3,
        forward: bool,
    ) -> Option<(&Structure, u32, &[BlockId])> {
        let (seed, indices) = self.deform_action_groups(world, pusher_pos, forward)?;
        let idx = *indices.first()?;
        let nodes = seed.deform_groups.get(idx as usize)?.nodes.as_slice();
        Some((seed, idx, nodes))
    }

    /// 由首选 DeformGroup 解析正/反推格点两侧
    pub fn deform_sides(&self, world: &WorldBlocks, pusher_pos: IVec3) -> Option<DeformSides> {
        let (seed, _, target_nodes) = self.deform_action(world, pusher_pos, true)?;
        let (_, _, actor_nodes) = self.deform_action(world, pusher_pos, false)?;

        let target_side = self.nodes_to_positions(world, seed, target_nodes);
        let actor_side = self.nodes_to_positions(world, seed, actor_nodes);
        let separated = !target_side.is_empty()
            && !actor_side.is_empty()
            && target_side.is_disjoint(&actor_side);
        let actor_anchored = actor_side.iter().any(|p| seed.scene_touching.contains(p));
        let target_anchored = target_side.iter().any(|p| seed.scene_touching.contains(p));
        Some(DeformSides {
            separated,
            actor_side,
            target_side,
            actor_anchored,
            target_anchored,
        })
    }

    /// 逻辑头以外的实体格；头占格由运动阶段 claimed_heads 处理
    pub fn nodes_to_positions(
        &self,
        world: &WorldBlocks,
        structure: &Structure,
        nodes: &[BlockId],
    ) -> HashSet<IVec3> {
        let mut out = HashSet::new();
        for &node in nodes {
            if structure.body_of_head.contains_key(&node) {
                continue;
            }
            if let Some(pos) = structure
                .positions
                .iter()
                .find(|p| world.blocks.get(p).is_some_and(|b| b.id == node))
            {
                out.insert(*pos);
            }
        }
        out
    }

    pub fn get(&self, id: StructureId) -> Option<&Structure> {
        self.structures.get(&id)
    }

    pub fn falling_structure_at(
        &self,
        pos: IVec3,
        offset: IVec3,
    ) -> Option<(StructureId, HashSet<IVec3>)> {
        let id = *self.structure_by_pos.get(&pos)?;
        let structure = self.structures.get(&id)?;
        if structure.activity != FactoryActivity::Active || !structure.freedom.can_translate(offset)
        {
            return None;
        }
        Some((id, structure.positions.clone()))
    }

    pub fn structure_id_at(&self, pos: IVec3) -> Option<StructureId> {
        self.structure_by_pos.get(&pos).copied()
    }

    pub fn structure_positions(&self, id: StructureId) -> Option<&HashSet<IVec3>> {
        self.structures
            .get(&id)
            .map(|structure| &structure.positions)
    }

    pub fn gravity_support_valid(
        &self,
        id: StructureId,
        world: &WorldBlocks,
        hard_pusher_head_occupancy: &HashSet<IVec3>,
    ) -> bool {
        let Some(structure) = self.structures.get(&id) else {
            return false;
        };
        let contacts = &structure.gravity_support;
        !contacts.is_empty()
            && contacts.iter().any(|(member, dir)| {
                structure.positions.contains(member) && {
                    let support = *member + *dir;
                    support.y >= 0
                        && !structure.positions.contains(&support)
                        && (!world.can_move_into_yielding_fragile(support)
                            || hard_pusher_head_occupancy.contains(&support))
                }
            })
    }

    pub fn record_gravity_support(
        &mut self,
        id: StructureId,
        world: &WorldBlocks,
        hard_pusher_head_occupancy: &HashSet<IVec3>,
    ) {
        let Some(structure) = self.structures.get_mut(&id) else {
            return;
        };
        structure.gravity_support =
            collect_gravity_support(world, &structure.positions, hard_pusher_head_occupancy);
    }

    pub fn clear_gravity_support(&mut self, id: StructureId) {
        if let Some(structure) = self.structures.get_mut(&id) {
            structure.gravity_support.clear();
        }
    }

    pub fn move_positions(&mut self, positions: &HashSet<IVec3>, offset: IVec3) {
        let mut changed_ids = HashSet::new();
        for pos in positions {
            if let Some(id) = self.structure_by_pos.get(pos).copied() {
                changed_ids.insert(id);
            }
        }
        for id in &changed_ids {
            let Some(structure) = self.structures.get(id) else {
                continue;
            };
            for pos in &structure.positions {
                self.structure_by_pos.remove(pos);
            }
        }
        for id in changed_ids {
            let Some(structure) = self.structures.get_mut(&id) else {
                continue;
            };
            structure.positions = structure
                .positions
                .iter()
                .map(|pos| {
                    if positions.contains(pos) {
                        *pos + offset
                    } else {
                        *pos
                    }
                })
                .collect();
            for (member, _dir) in &mut structure.gravity_support {
                if positions.contains(member) {
                    *member += offset;
                }
            }
            structure.scene_touching = structure
                .scene_touching
                .iter()
                .map(|pos| {
                    if positions.contains(pos) {
                        *pos + offset
                    } else {
                        *pos
                    }
                })
                .collect();
            for pos in &structure.positions {
                self.structure_by_pos.insert(*pos, id);
            }
        }
    }

    pub fn replace_structure_positions(
        &mut self,
        world: &WorldBlocks,
        old_positions: &HashSet<IVec3>,
        new_positions: HashSet<IVec3>,
    ) {
        let Some(&id) = old_positions
            .iter()
            .find_map(|pos| self.structure_by_pos.get(pos))
        else {
            return;
        };
        for pos in old_positions {
            self.structure_by_pos.remove(pos);
        }
        let Some(structure) = self.structures.get_mut(&id) else {
            return;
        };
        structure.positions = new_positions;
        structure.gravity_support.clear();
        for pos in &structure.positions {
            self.structure_by_pos.insert(*pos, id);
        }
        self.rebuild_deform_for(world, id);
    }

    pub fn movable_structure_at(&self, pos: IVec3) -> Option<HashSet<IVec3>> {
        let structure = self.structure(pos)?;
        if structure.activity != FactoryActivity::Active
            || structure.freedom == StructureFreedom::None
        {
            return None;
        }
        Some(structure.positions.clone())
    }

    pub fn freedom_at(&self, pos: IVec3) -> Option<StructureFreedom> {
        Some(self.structure(pos)?.freedom)
    }

    pub fn kind_at(&self, pos: IVec3) -> Option<StructureKind> {
        Some(self.structure(pos)?.kind)
    }

    pub fn structure_contains(&self, pos: IVec3, candidate: IVec3) -> bool {
        self.structure(pos)
            .is_some_and(|structure| structure.positions.contains(&candidate))
    }

    pub fn gravity_structure_ids(&self) -> Vec<StructureId> {
        let mut ids: Vec<StructureId> = self
            .structures
            .iter()
            .filter(|(_, structure)| structure.activity == FactoryActivity::Active)
            .map(|(id, _)| *id)
            .collect();
        ids.sort_by_key(|id| {
            self.structures
                .get(id)
                .and_then(|structure| structure.positions.iter().map(|pos| pos.y).min())
                .unwrap_or(0)
        });
        ids
    }

    fn structure(&self, pos: IVec3) -> Option<&Structure> {
        self.structure_by_pos
            .get(&pos)
            .and_then(|id| self.structures.get(id))
    }

    pub(super) fn structure_by_id(&self, id: StructureId) -> Option<&Structure> {
        self.structures.get(&id)
    }
}

fn bpos_facing_toward(body: IVec3, source: IVec3, target: IVec3) -> bool {
    body + source == target
}

/// 材料焊接连通（即时）
pub fn material_structure(world: &WorldBlocks, start: IVec3) -> HashSet<IVec3> {
    let Some(start_id) = world
        .blocks
        .get(&start)
        .filter(|block| block.kind.is_material() && !block.id.is_none())
        .map(|block| block.id)
    else {
        return HashSet::new();
    };
    let id_to_pos: HashMap<BlockId, IVec3> = world
        .blocks
        .iter()
        .filter(|(_, block)| block.kind.is_material() && !block.id.is_none())
        .map(|(pos, block)| (block.id, *pos))
        .collect();

    let mut structure = HashSet::new();
    let mut seen_ids = HashSet::from([start_id]);
    let mut queue = VecDeque::from([start_id]);
    structure.insert(start);

    while let Some(id) = queue.pop_front() {
        for weld in &world.material_welds {
            let Some(other_id) = weld.other(id) else {
                continue;
            };
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
        for (child_id, att) in &world.material_attachments {
            let other_id = if *child_id == id {
                att.parent
            } else if att.parent == id {
                *child_id
            } else {
                continue;
            };
            if !seen_ids.insert(other_id) {
                continue;
            }
            let Some(&neighbor) = id_to_pos.get(&other_id) else {
                continue;
            };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::BlockData;
    use crate::world::Facing;

    fn place(world: &mut WorldBlocks, pos: IVec3, kind: BlockKind, facing: Facing) -> BlockId {
        world.insert(pos, BlockData::new(kind, facing));
        world.blocks.get(&pos).unwrap().id
    }

    fn preferred_nodes(structure: &Structure, body: BlockId, forward: bool) -> &[BlockId] {
        let idx = structure.action_to_groups.get(&(body, forward)).unwrap()[0];
        structure.deform_groups[idx as usize].nodes.as_slice()
    }

    fn candidate_node_sets(
        structure: &Structure,
        body: BlockId,
        forward: bool,
    ) -> Vec<Vec<BlockId>> {
        structure
            .action_to_groups
            .get(&(body, forward))
            .unwrap()
            .iter()
            .map(|idx| structure.deform_groups[*idx as usize].nodes.clone())
            .collect()
    }

    /// 图 A：三平行推杆 → 两组共轴 deform（体列成环丢掉独立 fork）
    #[test]
    fn deform_figure_a_three_parallel() {
        let mut world = WorldBlocks::default();
        let b1 = place(
            &mut world,
            IVec3::new(0, 0, 0),
            BlockKind::Pusher,
            Facing::East,
        );
        let _f1 = place(
            &mut world,
            IVec3::new(1, 0, 0),
            BlockKind::Platform,
            Facing::North,
        );
        let b2 = place(
            &mut world,
            IVec3::new(0, 0, 1),
            BlockKind::Pusher,
            Facing::East,
        );
        let _f2 = place(
            &mut world,
            IVec3::new(1, 0, 1),
            BlockKind::Platform,
            Facing::North,
        );
        let b3 = place(
            &mut world,
            IVec3::new(0, 0, 2),
            BlockKind::Pusher,
            Facing::East,
        );
        let _f3 = place(
            &mut world,
            IVec3::new(1, 0, 2),
            BlockKind::Platform,
            Facing::North,
        );

        let mut state = StructureState::default();
        state.rebuild_for_simulation(&world);
        let sid = state.structure_id_at(IVec3::new(0, 0, 0)).unwrap();
        let structure = state.get(sid).unwrap();
        assert_eq!(structure.deform_groups.len(), 2);

        let fwd = structure.action_to_groups.get(&(b1, true)).unwrap().clone();
        let rev = structure
            .action_to_groups
            .get(&(b1, false))
            .unwrap()
            .clone();
        assert_eq!(fwd.len(), 1);
        assert_eq!(rev.len(), 1);
        assert_eq!(structure.action_to_groups.get(&(b2, true)), Some(&fwd));
        assert_eq!(structure.action_to_groups.get(&(b3, true)), Some(&fwd));
        assert_eq!(structure.action_to_groups.get(&(b2, false)), Some(&rev));
        assert_eq!(structure.action_to_groups.get(&(b3, false)), Some(&rev));

        let fwd_nodes = &structure.deform_groups[fwd[0] as usize].nodes;
        let rev_nodes = &structure.deform_groups[rev[0] as usize].nodes;
        assert_eq!(fwd_nodes.len(), 6); // 3 heads + 3 fronts
        assert_eq!(rev_nodes.len(), 3); // 3 bodies
        assert!(rev_nodes.contains(&b1) && rev_nodes.contains(&b2) && rev_nodes.contains(&b3));
    }

    /// 图 B：折角三杆；fork 可产生多候选，须含文档中的最大成功集
    #[test]
    fn deform_figure_b_bent() {
        let mut world = WorldBlocks::default();
        let b1 = place(
            &mut world,
            IVec3::new(0, 0, 0),
            BlockKind::Pusher,
            Facing::East,
        );
        let b2 = place(
            &mut world,
            IVec3::new(1, 0, 0),
            BlockKind::Pusher,
            Facing::South,
        );
        let b3 = place(
            &mut world,
            IVec3::new(1, 0, 1),
            BlockKind::Pusher,
            Facing::West,
        );

        let mut state = StructureState::default();
        state.rebuild_for_simulation(&world);
        let sid = state.structure_id_at(IVec3::new(0, 0, 0)).unwrap();
        let structure = state.get(sid).unwrap();

        let h1 = *structure.head_of.get(&b1).unwrap();
        let h2 = *structure.head_of.get(&b2).unwrap();
        let h3 = *structure.head_of.get(&b3).unwrap();

        let fwd1_sets = candidate_node_sets(structure, b1, true);
        assert!(
            fwd1_sets.iter().any(|n| {
                n.contains(&h1)
                    && n.contains(&b2)
                    && n.contains(&h2)
                    && n.contains(&b3)
                    && n.contains(&h3)
                    && !n.contains(&b1)
            }),
            "missing full (1,true) set: {fwd1_sets:?}"
        );
        assert_eq!(preferred_nodes(structure, b1, false), &[b1]);

        let fwd3 = candidate_node_sets(structure, b2, true);
        assert!(fwd3.iter().any(|n| {
            n.contains(&h2) && n.contains(&b3) && n.contains(&h3) && !n.contains(&b2)
        }));
        let rev3 = candidate_node_sets(structure, b2, false);
        assert!(
            rev3.iter()
                .any(|n| n.contains(&b2) && n.contains(&b1) && n.contains(&h1))
        );

        let fwd5 = preferred_nodes(structure, b3, true);
        assert!(fwd5.contains(&h3) && !fwd5.contains(&b3));
        let rev5 = candidate_node_sets(structure, b3, false);
        assert!(rev5.iter().any(|n| {
            n.contains(&b3)
                && n.contains(&h2)
                && n.contains(&b2)
                && n.contains(&h1)
                && n.contains(&b1)
        }));
    }

    /// 贴脸对向：独立正推含对面体+背面；共轴小集可只有两头
    #[test]
    fn deform_face_to_face_includes_back_cargo() {
        let mut world = WorldBlocks::default();
        let south = place(
            &mut world,
            IVec3::new(0, 0, 0),
            BlockKind::Blocker,
            Facing::South,
        );
        let north = place(
            &mut world,
            IVec3::new(0, 0, 1),
            BlockKind::Blocker,
            Facing::North,
        );
        let cargo = place(
            &mut world,
            IVec3::new(0, 0, 2),
            BlockKind::Wire,
            Facing::North,
        );

        let mut state = StructureState::default();
        state.rebuild_for_simulation(&world);
        let sid = state.structure_id_at(IVec3::new(0, 0, 1)).unwrap();
        let structure = state.get(sid).unwrap();
        let h_south = *structure.head_of.get(&south).unwrap();
        let h_north = *structure.head_of.get(&north).unwrap();

        let south_fwd = candidate_node_sets(structure, south, true);
        assert!(
            south_fwd.iter().any(|n| {
                n.contains(&h_south)
                    && n.contains(&h_north)
                    && n.contains(&north)
                    && n.contains(&cargo)
                    && !n.contains(&south)
            }),
            "missing independent south+: {south_fwd:?}"
        );
        assert!(
            south_fwd.iter().any(|n| {
                n.contains(&h_south)
                    && n.contains(&h_north)
                    && !n.contains(&north)
                    && !n.contains(&cargo)
            }),
            "missing coaxial heads-only south+: {south_fwd:?}"
        );

        let north_fwd = candidate_node_sets(structure, north, true);
        assert!(north_fwd.iter().any(|n| {
            n.contains(&h_south)
                && n.contains(&h_north)
                && n.contains(&south)
                && !n.contains(&north)
        }));

        assert_eq!(preferred_nodes(structure, south, false), &[south]);
        let rev_north = preferred_nodes(structure, north, false);
        assert!(rev_north.contains(&north) && rev_north.contains(&cargo));
    }

    /// 平行双杆正面列相连、体不相邻：共轴小集 + 各自独立拖对面体
    #[test]

    fn deform_parallel_gap_fork_independent() {
        let mut world = WorldBlocks::default();
        let a = place(
            &mut world,
            IVec3::new(-1, 2, -6),
            BlockKind::Pusher,
            Facing::East,
        );
        let b = place(
            &mut world,
            IVec3::new(-1, 2, -4),
            BlockKind::Blocker,
            Facing::East,
        );
        let p0 = place(
            &mut world,
            IVec3::new(0, 2, -6),
            BlockKind::Platform,
            Facing::North,
        );
        let p1 = place(
            &mut world,
            IVec3::new(0, 2, -5),
            BlockKind::Platform,
            Facing::North,
        );
        let p2 = place(
            &mut world,
            IVec3::new(0, 2, -4),
            BlockKind::Platform,
            Facing::North,
        );

        let mut state = StructureState::default();
        state.rebuild_for_simulation(&world);
        let structure = state
            .get(state.structure_id_at(IVec3::new(-1, 2, -4)).unwrap())
            .unwrap();
        let ha = *structure.head_of.get(&a).unwrap();
        let hb = *structure.head_of.get(&b).unwrap();

        let a_fwd = candidate_node_sets(structure, a, true);
        let b_fwd = candidate_node_sets(structure, b, true);
        assert!(
            a_fwd.len() >= 2 && b_fwd.len() >= 2,
            "a={a_fwd:?} b={b_fwd:?}"
        );

        let coaxial = [p0, p1, p2, ha, hb];
        assert!(
            a_fwd.iter().any(|n| {
                n.len() == 5
                    && coaxial.iter().all(|id| n.contains(id))
                    && !n.contains(&a)
                    && !n.contains(&b)
            }),
            "missing coaxial: {a_fwd:?}"
        );
        assert!(
            a_fwd.iter().any(|n| {
                n.len() == 6
                    && coaxial.iter().all(|id| n.contains(id))
                    && n.contains(&b)
                    && !n.contains(&a)
            }),
            "missing a independent: {a_fwd:?}"
        );
        assert!(
            b_fwd.iter().any(|n| {
                n.len() == 6
                    && coaxial.iter().all(|id| n.contains(id))
                    && n.contains(&a)
                    && !n.contains(&b)
            }),
            "missing b independent: {b_fwd:?}"
        );

        // 首选应是共轴小集，且两杆共享同一组
        let pref_a = structure.action_to_groups.get(&(a, true)).unwrap()[0];
        let pref_b = structure.action_to_groups.get(&(b, true)).unwrap()[0];
        assert_eq!(pref_a, pref_b);
        assert_eq!(structure.deform_groups[pref_a as usize].nodes.len(), 5);
    }

    /// 单活塞 2×2 方环：正反单切/全切均成环 → 不写任何 DeformGroup
    #[test]
    fn deform_single_pusher_square_loop_discards() {
        let mut world = WorldBlocks::default();
        place(
            &mut world,
            IVec3::new(13, 3, 12),
            BlockKind::Platform,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(14, 3, 12),
            BlockKind::Platform,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(14, 4, 12),
            BlockKind::Platform,
            Facing::North,
        );
        let body = place(
            &mut world,
            IVec3::new(13, 4, 12),
            BlockKind::Blocker,
            Facing::East,
        );

        let mut state = StructureState::default();
        state.rebuild_for_simulation(&world);
        let structure = state
            .get(state.structure_id_at(IVec3::new(13, 4, 12)).unwrap())
            .unwrap();
        assert!(
            structure.deform_groups.is_empty(),
            "cyclic single pusher must discard all deform actions; got {:?}",
            structure.deform_groups
        );
        assert!(structure.action_to_groups.is_empty());
        assert!(structure.action_to_groups.get(&(body, true)).is_none());
        assert!(structure.action_to_groups.get(&(body, false)).is_none());
        assert!(state.deform_sides(&world, IVec3::new(13, 4, 12)).is_none());
    }

    /// 对向环：单切成环 → 全切；西正推与东反推共轴
    #[test]
    fn deform_opposing_ring_coaxial_after_all_cut() {
        let mut world = WorldBlocks::default();
        let west = IVec3::new(13, 2, 16);
        let east = IVec3::new(13, 4, 16);
        for (x, y, z) in [
            (12, 2, 16),
            (12, 3, 16),
            (12, 4, 16),
            (14, 2, 16),
            (14, 3, 16),
            (14, 4, 16),
        ] {
            place(
                &mut world,
                IVec3::new(x, y, z),
                BlockKind::Platform,
                Facing::North,
            );
        }
        let west_id = place(&mut world, west, BlockKind::Blocker, Facing::West);
        let east_id = place(&mut world, east, BlockKind::Blocker, Facing::East);

        let mut state = StructureState::default();
        state.rebuild_for_simulation(&world);
        let sides_w = state.deform_sides(&world, west).unwrap();
        let sides_e = state.deform_sides(&world, east).unwrap();
        assert!(sides_w.separated, "west must separate after all-cut");
        assert!(sides_e.separated, "east must separate after all-cut");
        assert_eq!(
            sides_w.target_side, sides_e.actor_side,
            "west forward == east reverse"
        );
        assert_eq!(
            sides_w.actor_side, sides_e.target_side,
            "west reverse == east forward"
        );

        let structure = state.get(state.structure_id_at(west).unwrap()).unwrap();
        let w_fwd = structure.action_to_groups.get(&(west_id, true)).unwrap()[0];
        let e_rev = structure.action_to_groups.get(&(east_id, false)).unwrap()[0];
        assert_eq!(w_fwd, e_rev);
    }

    /// 伸出真实头后工厂连通仍穿过头
    #[test]
    fn extended_pusher_head_keeps_front_connected() {
        let mut world = WorldBlocks::default();
        let body = IVec3::new(0, 1, 0);
        let head = IVec3::new(1, 1, 0);
        let front = IVec3::new(2, 1, 0);
        world.insert(body, BlockData::new(BlockKind::Blocker, Facing::East));
        world.insert(head, BlockData::new(BlockKind::PusherHead, Facing::East));
        world.insert(front, BlockData::new(BlockKind::Platform, Facing::North));

        let members = factory_structure(&world, body);
        assert!(members.contains(&front));
        assert!(!members.contains(&head));

        let mut state = StructureState::default();
        state.rebuild_for_simulation(&world);
        let sides = state.deform_sides(&world, body).expect("sides");
        assert!(sides.separated);
        assert!(sides.target_side.contains(&front));
        assert!(!sides.target_side.contains(&body));
    }
}
