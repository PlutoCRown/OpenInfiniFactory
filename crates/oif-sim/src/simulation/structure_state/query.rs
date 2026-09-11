use super::{
    BlockId, DeformSides, FactoryActivity, GridBounds, HashSet, IVec3, Structure, StructureFreedom,
    StructureId, StructureKind, StructureState, WorldBlocks, collect_gravity_support,
};

impl StructureState {
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
        self.pushable_structure_positions_at(pos, offset).cloned()
    }

    /// 查询可推动结构的位置引用，供只读热点避免复制整组坐标
    pub fn pushable_structure_positions_at(
        &self,
        pos: IVec3,
        offset: IVec3,
    ) -> Option<&HashSet<IVec3>> {
        let structure = self.structure(pos)?;
        if !structure.is_pushable() || !structure.freedom.can_translate(offset) {
            return None;
        }
        Some(&structure.positions)
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
        // 仅场景 / Inactive 支撑可跨回合生效；撑在 Active 上时对方下回合可能自己动
        !contacts.is_empty()
            && contacts.iter().any(|(member, dir)| {
                structure.positions.contains(member) && {
                    let support = *member + *dir;
                    support.y >= 0
                        && !structure.positions.contains(&support)
                        && (!world.can_move_into_yielding_fragile(support)
                            || hard_pusher_head_occupancy.contains(&support))
                        && (world.is_scene_at(support)
                            || self.structure(support).is_some_and(|s| {
                                s.kind == StructureKind::Factory
                                    && s.activity == FactoryActivity::Inactive
                            }))
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

    /// 扫描结构成员下方，找到第一个场景 / Inactive 稳定支撑即记下并返回 true
    pub fn try_record_stable_gravity_support(
        &mut self,
        id: StructureId,
        world: &WorldBlocks,
        hard_pusher_head_occupancy: &HashSet<IVec3>,
    ) -> bool {
        let positions: Vec<IVec3> = {
            let Some(structure) = self.structures.get(&id) else {
                return false;
            };
            structure.positions.iter().copied().collect()
        };
        let mut contact = None;
        for pos in positions {
            let below = pos + IVec3::NEG_Y;
            if below.y < 0 {
                continue;
            }
            if self.structure_by_pos.get(&below) == Some(&id) {
                continue;
            }
            if world.can_move_into_yielding_fragile(below)
                && !hard_pusher_head_occupancy.contains(&below)
            {
                continue;
            }
            let stable = world.is_scene_at(below)
                || self.structure(below).is_some_and(|s| {
                    s.kind == StructureKind::Factory && s.activity == FactoryActivity::Inactive
                });
            if stable {
                contact = Some((pos, IVec3::NEG_Y));
                break;
            }
        }
        let Some(contact) = contact else {
            return false;
        };
        if let Some(structure) = self.structures.get_mut(&id) {
            structure.gravity_support = vec![contact];
        }
        true
    }

    pub fn clear_gravity_support(&mut self, id: StructureId) {
        if let Some(structure) = self.structures.get_mut(&id) {
            structure.gravity_support.clear();
        }
    }

    /// 原子提交平移：世界占格、结构位置、支撑和包围盒由同一入口维护
    pub fn translate(
        &mut self,
        world: &mut WorldBlocks,
        positions: &HashSet<IVec3>,
        offset: IVec3,
    ) {
        let was_synced = self
            .material_topology
            .as_ref()
            .is_some_and(|revision| std::sync::Arc::ptr_eq(revision, &world.material_topology));
        let moves = positions
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
            structure.bounds = GridBounds::from_positions(&structure.positions);
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
        if was_synced {
            self.material_topology = Some(world.material_topology.clone());
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
        structure.bounds = GridBounds::from_positions(&new_positions);
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

    pub(in crate::simulation) fn structure_by_id(&self, id: StructureId) -> Option<&Structure> {
        self.structures.get(&id)
    }
}
