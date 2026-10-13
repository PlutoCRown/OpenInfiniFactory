use super::{
    BlockId, DeformSides, GridBounds, HashSet, IVec3, Structure, StructureId, StructureKind,
    StructureState, WorldBlocks, collect_gravity_support,
};

impl StructureState {
    pub fn structure_ids(&self) -> impl Iterator<Item = StructureId> + '_ {
        self.structures.keys().copied()
    }

    pub fn id_at(&self, pos: IVec3) -> Option<StructureId> {
        self.structure_by_pos.get(&pos).copied()
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
        let actor_anchored = seed.contains_anchored_members(&actor_side);
        let target_anchored = seed.contains_anchored_members(&target_side);
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
        // 场景或锚定结构的支撑可跨回合生效，未锚定的支撑结构下回合可能移动
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
                                s.kind == StructureKind::Factory && s.is_anchored()
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

    /// 记录成员下方第一个场景或锚定结构提供的稳定支撑
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
                || self
                    .structure(below)
                    .is_some_and(|s| s.kind == StructureKind::Factory && s.is_anchored());
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
            structure.anchored_members = structure
                .anchored_members
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

    pub fn kind_at(&self, pos: IVec3) -> Option<StructureKind> {
        Some(self.structure(pos)?.kind)
    }

    pub fn structure_contains(&self, pos: IVec3, candidate: IVec3) -> bool {
        self.structure(pos)
            .is_some_and(|structure| structure.positions.contains(&candidate))
    }

    /// 从低到高列出未锚定结构，重力支撑与碰撞由运动阶段判断
    pub fn gravity_structure_ids(&self) -> Vec<StructureId> {
        let mut ids: Vec<StructureId> = self
            .structures
            .iter()
            .filter(|(_, structure)| !structure.is_anchored())
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
}
